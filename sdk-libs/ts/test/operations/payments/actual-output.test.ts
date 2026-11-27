import { AccountRole, address } from "@solana/kit";
import { describe, expect, it } from "vitest";

import { initializePoseidon } from "@heliuslabs/spr";
import {
  SHIELDED_POOL_PROGRAM_ID,
  SPL_TOKEN_2022_PROGRAM_ID,
  treeIdField,
  UTXO_DOMAIN,
  type Bytes32,
} from "@heliuslabs/spr/interface";
import { ShieldedKeypair } from "@heliuslabs/spr/keypair";
import { SOL_MINT } from "@heliuslabs/spr/transaction";
import {
  addressBytes,
  hashBytes,
  poseidon,
  SYSTEM_PROGRAM,
  splInterfaceWithBump,
  ZERO_32,
} from "@heliuslabs/spr/internal";

import {
  ACTUAL_OUTPUT_MINIMUM_OFFSET,
  actualOutputReceiptAddress,
  buildSettleActualOutputInstruction,
  prepareActualOutputDescriptor,
  type ActualOutputDescriptor,
} from "../../../src/operations/payments/actual-output.js";
import { ringPolicyPda } from "../../../src/protocol/pda/index.js";
import { MEMBER_REGISTRY_EMPTY_ROOT } from "../../../src/protocol/membership/member-opening.js";
import { ringIdField } from "../../../src/crypto/derivation/ring-stream.js";
import { RingTransactData } from "../../../src/protocol/transaction/transact.js";
import { LocalRingKeys } from "../../../src/state/ring-keys.js";
import {
  ownedAccount,
  ringPolicyData,
  storageRegistryAccount,
} from "../../helpers/ring-accounts.js";
import { PROOF_SERVICE, ringFixture, small, TREE_ID } from "../../helpers/ring-pipeline.js";
import { BLOCKHASH } from "../../../../../zolana/sdk-libs/ts/test/helpers/clients.js";

const authority = address("9EwHno8C1T1vVGjasGnDH1GubiEu8qbgLX9qDjBshFhz");
const sponsor = address("9vyTbYGyh3cwxkAQpjjFQGXmdJP6p9B6YcQ5pNuXPNbh");
const source = address("Cura111111111111111111111111111111111111111");
const descriptor: ActualOutputDescriptor = {
  ringProgramId: address("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
  asset: address("Mint111111111111111111111111111111111111111"),
  outputTree: address("SysvarC1ock11111111111111111111111111111111"),
  registries: [address("SysvarRent111111111111111111111111111111111")],
  slot: 1,
  ownerUtxoHash: new Uint8Array(32).fill(7) as Bytes32,
  privateTxBlinding: new Uint8Array(32).fill(9) as Bytes32,
  zeroChangeOwnerUtxoHash: new Uint8Array(32).fill(8) as Bytes32,
  certificate: Uint8Array.of(4, 5, 6),
};

describe("actual-output settlement instructions", () => {
  it.each([false, true])(
    "binds stock deposit slots and verified roots (own records: %s)",
    async (records) => {
      await initializePoseidon();
      const ring = await ringFixture({ mode: "anonymous" });
      const ringProgramId = ring.ringProgramId;
      const [policy, bump] = await ringPolicyPda(ringProgramId);
      ring.accounts.set(
        policy,
        ownedAccount(
          ringProgramId,
          ringPolicyData({
            bump,
            roles: { admin: ring.authority },
            auditorPublicKey: ring.auditor.publicKey().toBytes(),
            mode: "anonymous",
            tracking: ring.trackingRegistry,
            ...(records ? { records: ring.ownRegistry } : {}),
          }),
        ),
      );
      const keys = LocalRingKeys.fromKeypair(ShieldedKeypair.generate());
      try {
        ring.enroll(keys.address());
        const root = { hash: MEMBER_REGISTRY_EMPTY_ROOT, revision: 0n, expiresAt: 0n };
        const recordContext = {
          ringId: ringIdField(ringProgramId),
          trackingRegistry: ring.trackingRegistry,
        };
        if (records)
          ring.accounts.set(
            ring.ownRegistry,
            storageRegistryAccount([root], ring.authority, recordContext),
          );
        const params = {
          client: {
            ...ring.client,
            proofDataSource: "prover" as const,
            proofService: PROOF_SERVICE,
            getLatestBlockhash: async () => BLOCKHASH,
            getStorageMemberOpenings: async (
              request: Parameters<typeof ring.client.getStorageMemberOpenings>[0],
            ) => {
              const reply = await ring.client.getStorageMemberOpenings(request);
              return { ...reply, root: { ...reply.root, revision: 0n } };
            },
            getStorageRecordPool: async (
              request: Parameters<typeof ring.client.getStorageRecordPool>[0],
            ) => {
              const reply = await ring.client.getStorageRecordPool(request);
              return { ...reply, root: { ...reply.root, revision: 0n } };
            },
          },
          ringProgramId,
          feePayer: ring.authority,
          recipient: {
            kind: "anonymous" as const,
            keys,
            listSalt: await keys.listSalt(ringProgramId),
          },
        };
        const paid = await prepareActualOutputDescriptor(params);
        const frame = RingTransactData.parse(paid.certificate.subarray(1), "anonymous");
        const request = ring.ringRequests[0];
        expect(request?.kind === "transfer" && request.ownRoot).toEqual(
          records ? MEMBER_REGISTRY_EMPTY_ROOT : ZERO_32,
        );
        expect(frame.ownRevision).toBe(records ? 0n : undefined);
        expect(paid.registries).toEqual(
          records ? [ring.ownRegistry, ring.trackingRegistry] : [ring.trackingRegistry],
        );
        const ringHash = poseidon([ZERO_32, hashBytes(addressBytes(ringProgramId))]);
        const solAsset = hashBytes(addressBytes(SOL_MINT));
        const outputHash = (amount: number, owner: Bytes32) =>
          poseidon([
            small(UTXO_DOMAIN),
            treeIdField(TREE_ID),
            solAsset,
            small(amount),
            ZERO_32,
            ringHash,
            owner,
          ]);
        const change = outputHash(0, paid.zeroChangeOwnerUtxoHash);
        const output = outputHash(1, paid.ownerUtxoHash);
        expect(paid.slot).toBe(1);
        expect(frame.outputs.map((slot) => slot.hash)).toEqual([change, output]);
        expect(frame.privateTxHash).toEqual(
          poseidon([ZERO_32, poseidon([change, output]), ZERO_32, paid.privateTxBlinding]),
        );
        if (records) {
          for (const newerRegistry of [
            storageRegistryAccount(
              [{ ...root, revision: 1n }, root],
              ring.authority,
              recordContext,
            ),
            storageRegistryAccount([root], ring.authority, { ...recordContext, nextIndex: 2n }),
            storageRegistryAccount([root], ring.authority, { ...recordContext, ringId: ZERO_32 }),
          ]) {
            ring.accounts.set(ring.ownRegistry, newerRegistry);
            await expect(prepareActualOutputDescriptor(params)).rejects.toMatchObject({
              code: "RING_BUILD_DEPOSIT",
            });
          }
          expect(ring.ringRequests).toHaveLength(1);
        }
      } finally {
        keys.destroy();
      }
    },
  );

  it("encodes the exact SPL amount and separates the operation authority from the receipt payer", async () => {
    const minimumOutput = (1n << 53n) + 3n;
    const [splInterface, bump] = await splInterfaceWithBump(descriptor.asset);
    const instruction = await buildSettleActualOutputInstruction({
      descriptor,
      authority,
      receiptPayer: sponsor,
      minimumOutput,
      sourceTokenAccount: source,
      tokenProgram: SPL_TOKEN_2022_PROGRAM_ID,
    });
    const data = Uint8Array.from(instruction.data ?? []);
    const accounts = instruction.accounts ?? [];
    expect([...data.subarray(0, 3)]).toEqual([67, 1, 1]);
    expect(new DataView(data.buffer).getBigUint64(ACTUAL_OUTPUT_MINIMUM_OFFSET, true)).toBe(
      minimumOutput,
    );
    expect(data.subarray(115, 147)).toEqual(descriptor.zeroChangeOwnerUtxoHash);
    expect([...data.subarray(147)]).toEqual([1, bump, 3, 0, 4, 5, 6]);
    expect(accounts).toHaveLength(13);
    expect(accounts[2]?.address).toBe(
      await actualOutputReceiptAddress(descriptor.ringProgramId, descriptor.ownerUtxoHash),
    );
    expect(accounts[3]).toEqual({ address: sponsor, role: AccountRole.WRITABLE_SIGNER });
    expect(accounts[4]?.address).toBe(SYSTEM_PROGRAM);
    expect(accounts[5]?.address).toBe(descriptor.outputTree);
    expect(accounts[6]).toEqual({ address: authority, role: AccountRole.WRITABLE_SIGNER });
    expect(accounts.slice(8).map((account) => account.address)).toEqual([
      SHIELDED_POOL_PROGRAM_ID,
      SPL_TOKEN_2022_PROGRAM_ID,
      descriptor.asset,
      source,
      splInterface,
    ]);
  });

  it("rejects contradictory token and reserve parameters", async () => {
    const token = {
      descriptor,
      authority,
      receiptPayer: sponsor,
      minimumOutput: 1n,
      sourceTokenAccount: source,
      tokenProgram: SPL_TOKEN_2022_PROGRAM_ID,
    };
    for (const invalid of [
      { ...token, keepLamports: 1n },
      { ...token, minimumOutput: 0n },
      { ...token, tokenProgram: SYSTEM_PROGRAM },
      { ...token, descriptor: { ...descriptor, asset: SOL_MINT } },
      { ...token, descriptor: { ...descriptor, registries: [] } },
    ]) {
      await expect(buildSettleActualOutputInstruction(invalid)).rejects.toMatchObject({
        code: "RING_BUILD_DEPOSIT",
        details: { reason: "invalid actual-output descriptor or settlement" },
      });
    }
  });
});
