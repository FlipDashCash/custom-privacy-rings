/** Exact SDK proof requests, test tree and transport only. No proof bytes returned. */
import { getAddressEncoder } from "@solana/kit";
import { describe, expect, it, vi } from "vitest";

import { SprClient } from "@heliuslabs/spr/client";
import { splAssetRegistryAddress } from "@heliuslabs/spr/internal";
import {
  SHIELDED_POOL_PROGRAM_ID,
  SPL_TOKEN_2022_PROGRAM_ID,
  StateDiscriminator,
  type Bytes32,
} from "@heliuslabs/spr/interface";
import { ShieldedKeypair } from "@heliuslabs/spr/keypair";
import { SOL_MINT, type SppProofInputs } from "@heliuslabs/spr/transaction";

import { RingAddress } from "../../../src/crypto/keys/ring-address.js";
import {
  PreparedRingDeposit,
  type RingDepositClient,
} from "../../../src/operations/payments/deposit.js";
import { BLOCKHASH } from "../../../../../zolana/sdk-libs/ts/test/helpers/clients.js";
import { randomAddress, ringFixture, TREE_ID } from "../../helpers/ring-pipeline.js";

describe("funded indexed Custom Ring composition", () => {
  it.each([
    { mode: "confidential", spl: false },
    { mode: "anonymous", spl: false },
    { mode: "confidential", spl: true },
    { mode: "anonymous", spl: true },
  ] as const)(
    "$mode spl=$spl builds both real requests, never succeeds without proofs",
    async ({ mode, spl }) => {
      const fixture = await ringFixture({ mode });
      const recipient = ShieldedKeypair.generate().shieldedAddress();
      const recipientTerm =
        mode === "confidential"
          ? ({ kind: "confidential", address: recipient } as const)
          : ({
              kind: "anonymousPayment",
              to: new RingAddress({
                recipient,
                ringProgramId: fixture.ringProgramId,
                listSalt: new Uint8Array(32).fill(1) as Bytes32,
              }),
            } as const);
      fixture.enroll(recipient);
      const depositor = randomAddress();
      const amount = 25n;
      const asset = spl ? randomAddress() : SOL_MINT;
      const source = randomAddress();
      const mintRegistry = spl ? await splAssetRegistryAddress(asset) : undefined;
      const registryData = new Uint8Array(48);
      if (spl) {
        registryData[0] = StateDiscriminator.splAssetRegistry;
        registryData.set(getAddressEncoder().encode(asset), 8);
        new DataView(registryData.buffer).setBigUint64(40, 2n, true);
      }
      const requests: { url: string; body: unknown }[] = [];
      const fetch: typeof globalThis.fetch = async (url, init) => {
        requests.push({
          url: url instanceof Request ? url.url : String(url),
          body: JSON.parse(String(init?.body)) as unknown,
        });
        return new Response(JSON.stringify({ error: "offline test: no prover configured" }), {
          status: 400,
          headers: { "content-type": "application/json" },
        });
      };
      const spp = new SprClient({ proofDataSource: "prover", treeId: TREE_ID, fetch });
      let staged: SppProofInputs | undefined;
      const proveRing = vi.fn(async (_request: Parameters<RingDepositClient["proveRing"]>[0]) => {
        throw new Error("offline test: no ring prover configured");
      });
      const client: RingDepositClient = {
        ...fixture.client,
        proofDataSource: "prover",
        proofService: spp.proofService,
        getLatestBlockhash: async () => BLOCKHASH,
        getAccount: (id) =>
          id === mintRegistry
            ? Promise.resolve({ owner: SHIELDED_POOL_PROGRAM_ID, data: registryData, lamports: 1n })
            : fixture.client.getAccount(id),
        proveRing,
        proveRingTransact: (...args) => {
          staged = args[0];
          return spp.proveRingTransact(...args);
        },
      };
      const params = {
        client,
        ringProgramId: fixture.ringProgramId,
        feePayer: fixture.authority,
        depositor,
        recipient: recipientTerm,
        amount,
        ...(spl ? { asset, splTokenAccount: source } : {}),
      };
      if (spl) {
        await expect(
          PreparedRingDeposit.prepare({
            ...params,
            splTokenProgram: SPL_TOKEN_2022_PROGRAM_ID,
          }),
        ).rejects.toMatchObject({ code: "RING_BUILD_DEPOSIT" });
      }
      const prepared = await PreparedRingDeposit.prepare(params);
      expect(requests).toEqual([]);
      const review = vi.fn();
      await expect(
        prepared.proveForComposition({ ...client, proofDataSource: "client" }, review),
      ).rejects.toMatchObject({ code: "RING_DEPOSIT_PROOF_DATA_SOURCE" });
      await expect(prepared.proveForComposition(client, review)).rejects.toBeInstanceOf(Error);
      await vi.waitFor(() => expect(requests).toHaveLength(1));
      expect(proveRing).toHaveBeenCalledTimes(1);
      expect(review).not.toHaveBeenCalled();
      await expect(prepared.proveForComposition(client, review)).rejects.toMatchObject({
        code: "RING_DEPOSIT_FRAME_REUSED",
      });
      expect(requests[0]?.url).toMatch(/\/prove\/transfer_ring_1_2\/indexed$/u);
      expect(requests[0]?.body).toMatchObject({
        circuitType: "transfer-ring",
        trees: [{ id: TREE_ID, tree: fixture.client.tree }],
        inputs: [{ treeSlot: 0, nullifier: null, commitment: null }],
      });
      if (staged === undefined) throw new Error("pool proof inputs were not prepared");
      expect(staged.payer).toBe(fixture.authority);
      expect(staged.externalData.interfaceTransfers).toEqual([
        spl
          ? expect.objectContaining({
              kind: "spl",
              mint: asset,
              isDeposit: true,
              amount,
              tokenAccount: source,
            })
          : { kind: "sol", isDeposit: true, amount, userSolAccount: depositor },
      ]);
      const paid = staged.outputs.filter((output) => !output.isDummy() && output.amount > 0n);
      expect(paid).toHaveLength(1);
      expect(paid[0]).toMatchObject({ asset, amount, ringProgramId: fixture.ringProgramId });
      if (mode === "confidential")
        expect(paid[0]?.ownerAddress?.signingPublicKey.toBytes()).toEqual(
          recipient.signingPublicKey.toBytes(),
        );
    },
  );
});
