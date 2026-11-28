// Opt in with a stock Localnet fixture and independently replayed providers. See measured-output.md.
import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { performance } from "node:perf_hooks";
import { promisify } from "node:util";

import { LocalKeys, Wallet } from "@heliuslabs/spr";
import {
  associatedTokenAddress,
  compileUnsignedTransaction,
  createAssociatedTokenAccountInstruction,
  createSplInterfaceInstruction,
  decodeRingDepositOutput,
  readOutputFrame,
  SOL_INTERFACE,
  splInterfaceWithBump,
} from "@heliuslabs/spr/internal";
import {
  decodeTreeFees,
  SHIELDED_POOL_PROGRAM_ID,
  SPL_TOKEN_2022_PROGRAM_ID,
  SPL_TOKEN_PROGRAM_ID,
  TRANSACTION_SIZE_LIMIT,
} from "@heliuslabs/spr/interface";
import { X25519Keypair } from "@heliuslabs/spr/keypair";
import { SOL_MINT } from "@heliuslabs/spr/transaction";
import { getTransferSolInstruction } from "@solana-program/system";
import { getTransferCheckedInstruction } from "@solana-program/token";
import {
  address,
  createKeyPairSignerFromBytes,
  generateKeyPairSigner,
  getBase64EncodedWireTransaction,
  signTransactionWithSigners,
  type Address,
  type Instruction,
  type Signature,
  type TransactionPartialSigner,
} from "@solana/kit";
import { describe, expect, it, vi } from "vitest";

import {
  AnonymousWallet,
  buildSettleActualOutputInstruction,
  createRingWithdrawalSubmission,
  fetchRingPolicy,
  messageSignerSpendSigner,
  openActualOutputForAuditor,
  prepareActualOutputDescriptor,
  RingRpc,
  syncRingWallet,
  ViewingHistory,
} from "../../src/ring/index.js";
import { actualOutputReservationAddresses } from "../../src/operations/payments/actual-output-reservation.js";
import { createCustomRingClient, type CustomRingClient } from "../../src/rpc/client.js";
import { requiredEnv, type Actor } from "./live-helpers.js";
import { airdrop, enroll, freshActor, ringKeys, settle } from "./ring-live-helpers.js";

const CU_LIMIT = 1_400_000;
const FIXTURE_MINIMUM = 1_000n;
const RECORD_TAG = new TextEncoder().encode("CRADv001");
const RING_SETTLEMENT_ERROR = { InstructionError: [1n, { Custom: 8242n }] };
const SPP_SETTLEMENT_ERROR = { InstructionError: [1n, { Custom: 7010n }] };
const SPP_RESERVATION_EXISTS = { InstructionError: [1n, { Custom: 7043n }] };

type Endpoints = Readonly<{ rpcUrl: string; indexerUrl: string; proverUrl: string }>;
type Asset = Readonly<{ name: string; mint: Address; tokenProgram?: Address }>;
interface Fixture extends Endpoints {
  readonly genesisHash: string;
  readonly ringProgram: string;
  readonly tree: string;
  readonly ringRpcUrl: string;
  readonly closedServices: readonly string[];
  readonly payerFile: string;
  readonly assetFile: string;
  readonly custodyFile: string;
  readonly kmsUrl: string;
  readonly switchCommand: string;
  readonly independentNetworkFile: string;
}

function loopback(value: string): string {
  const url = new URL(value);
  expect(url.origin).toBe(value);
  expect(url.protocol).toBe("http:");
  expect(["127.0.0.1", "localhost"]).toContain(url.hostname);
  expect(url.port).not.toBe("");
  return value;
}

async function readJson<T>(path: string): Promise<T> {
  return JSON.parse(await readFile(path, "utf8")) as T;
}

function metric(name: string, values: Readonly<Record<string, unknown>>) {
  process.stdout.write(`${JSON.stringify({ measured: name, ...values })}\n`);
}

async function measuredClient(
  fixture: Fixture,
  endpoints: Endpoints,
  originReads?: { count: number },
) {
  for (const endpoint of Object.values(endpoints)) loopback(endpoint);
  const client = await createCustomRingClient({
    solanaRpcUrl: endpoints.rpcUrl,
    indexerUrl: endpoints.indexerUrl,
    proverUrl: endpoints.proverUrl,
    tree: address(fixture.tree),
    commitment: "finalized",
    proofDataSource: "prover",
    indexerConfig: { poll: { numRetries: 180, delayMs: 250n, maxDelayMs: 2_000n } },
    fetch(input, init) {
      const url = new URL(input instanceof Request ? input.url : input);
      expect(Object.values(endpoints)).toContain(url.origin);
      if (originReads && url.origin === endpoints.rpcUrl && typeof init?.body === "string") {
        const { method } = JSON.parse(init.body) as { method?: string };
        if (method === "getTransaction") originReads.count++;
      }
      return globalThis.fetch(input, init);
    },
  });
  expect(await client.solanaRpc.getGenesisHash().send()).toBe(fixture.genesisHash);
  return client;
}

async function finalized(client: CustomRingClient, signature: Signature) {
  const deadline = performance.now() + 75_000;
  while (performance.now() < deadline) {
    const info = await client.solanaRpc
      .getTransaction(signature, {
        commitment: "finalized",
        encoding: "json",
        maxSupportedTransactionVersion: 1,
      })
      .send();
    if (info?.meta !== null && info?.meta !== undefined) return { ...info, meta: info.meta };
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error("The validator did not finalize the transaction");
}

async function land(
  client: CustomRingClient,
  feePayer: TransactionPartialSigner,
  instructions: readonly Instruction[],
  signers: readonly TransactionPartialSigner[],
  computeUnitLimit = CU_LIMIT,
) {
  const transaction = compileUnsignedTransaction({
    feePayer: feePayer.address,
    lifetime: await client.getLatestBlockhash(),
    computeUnitLimit,
    instructions,
  });
  const signed = await signTransactionWithSigners([feePayer, ...signers], transaction);
  const wire = getBase64EncodedWireTransaction(signed);
  const wireBytes = Buffer.from(wire, "base64").length;
  expect(wireBytes).toBeLessThanOrEqual(TRANSACTION_SIZE_LIMIT);
  const signature = await client.solanaRpc
    .sendTransaction(wire, {
      encoding: "base64",
      skipPreflight: true,
      preflightCommitment: "finalized",
    })
    .send();
  return { ...(await finalized(client, signature)), signature, wireBytes };
}

async function account(client: CustomRingClient, key: Address) {
  return (
    await client.solanaRpc
      .getAccountInfo(key, {
        encoding: "base64",
        commitment: "finalized",
      })
      .send()
  ).value;
}

async function balance(client: CustomRingClient, key: Address, asset: Asset) {
  if (asset.tokenProgram === undefined) return client.getBalance(key);
  const info = await client.solanaRpc
    .getTokenAccountBalance(key, { commitment: "finalized" })
    .send();
  return BigInt(info.value.amount);
}

function hold(actor: Actor, program: Address, asset: Asset): bigint {
  return actor.wallet
    .utxos()
    .filter(
      ({ spent, utxo }) => !spent && utxo.ringProgramId === program && utxo.asset === asset.mint,
    )
    .reduce((total, entry) => total + entry.utxo.amount, 0n);
}

function freshWallet(client: CustomRingClient, actor: Actor): Actor {
  return {
    signer: actor.signer,
    keypair: actor.keypair,
    wallet: new Wallet({ identity: actor.keypair.shieldedAddress() }),
    keys: LocalKeys.fromKeypair(actor.keypair, client.proofService),
  };
}

async function scan(client: CustomRingClient, actor: Actor, doc: AnonymousWallet) {
  await syncRingWallet({
    client,
    wallet: doc,
    keys: ringKeys(actor),
    config: { requireSlot: await client.getSlot() },
  });
}

function ledger(actor: Actor, ring: Address) {
  const history = ViewingHistory.create(
    actor.keypair.shieldedAddress().viewingPublicKey,
    0n,
  ).withRing(ring);
  return AnonymousWallet.create(actor.wallet, history);
}

async function auditKey(fixture: Fixture): Promise<X25519Keypair> {
  const custody = await readJson<{ auditor: { key_arn: string; ciphertext: string } }>(
    fixture.custodyFile,
  );
  const { auditor: wrapped } = custody;
  const reply = await fetch(loopback(fixture.kmsUrl), {
    method: "POST",
    redirect: "error",
    headers: {
      "content-type": "application/x-amz-json-1.1",
      "x-amz-target": "TrentService.Decrypt",
    },
    body: JSON.stringify({
      KeyId: wrapped.key_arn,
      CiphertextBlob: wrapped.ciphertext,
      EncryptionContext: {
        application: "zolana-custom-rings-v1",
        deployment: "localnet",
        genesis: fixture.genesisHash,
        ring: fixture.ringProgram,
        purpose: "auditor",
      },
    }),
  });
  expect(reply.ok).toBe(true);
  const decrypted = (await reply.json()) as { KeyId: string; Plaintext: string };
  expect(decrypted.KeyId === wrapped.key_arn).toBe(true);
  const bytes = Buffer.from(decrypted.Plaintext, "base64");
  try {
    return X25519Keypair.fromBytes(bytes);
  } finally {
    bytes.fill(0);
  }
}

describe.skipIf(process.env.RING_MEASURED_CONTEXT === undefined)("measured ring deposit", () => {
  it("settles genuine proofs, proves atomic rollback, and exits from independent history", async () => {
    const fixture = await readJson<Fixture>(requiredEnv("RING_MEASURED_CONTEXT"));
    const { rpcUrl, indexerUrl, proverUrl } = fixture;
    const client = await measuredClient(fixture, { rpcUrl, indexerUrl, proverUrl });
    const tree = address(fixture.tree);
    const ringProgramId = address(fixture.ringProgram);
    const { classic, metadata, fee, zeroFee } = await readJson<
      Record<"classic" | "metadata" | "fee" | "zeroFee", string>
    >(fixture.assetFile);
    const payer = await createKeyPairSignerFromBytes(
      Uint8Array.from(await readJson<number[]>(fixture.payerFile)),
    );
    const owner = await freshActor(client);
    const exitPayer = await generateKeyPairSigner();
    const exitRecipient = await generateKeyPairSigner();
    await airdrop(client, exitPayer.address);

    const sol = { name: "SOL", mint: SOL_MINT };
    const tokens: readonly Asset[] = [
      { name: "classic SPL", mint: address(classic), tokenProgram: SPL_TOKEN_PROGRAM_ID },
      {
        name: "Token-2022 metadata",
        mint: address(metadata),
        tokenProgram: SPL_TOKEN_2022_PROGRAM_ID,
      },
      {
        name: "Token-2022 zero transfer fee",
        mint: address(zeroFee),
        tokenProgram: SPL_TOKEN_2022_PROGRAM_ID,
      },
      {
        name: "Token-2022 transfer fee",
        mint: address(fee),
        tokenProgram: SPL_TOKEN_2022_PROGRAM_ID,
      },
    ];
    const actuals = new Map<Address, bigint>();
    const successes = new Map<
      Signature,
      Readonly<{ asset: Asset; actual: bigint; certificate: Uint8Array }>
    >();
    const prepareRing = vi.spyOn(client, "proveRing");
    const prepareRingSpp = vi.spyOn(client, "proveRingTransact");
    const prepareReservation = vi.spyOn(client, "proveTransact");
    for (const asset of [sol, ...tokens]) {
      const authority = await generateKeyPairSigner();
      const tokenProgram = "tokenProgram" in asset ? asset.tokenProgram : undefined;
      const tokenAccounts =
        tokenProgram === undefined
          ? undefined
          : {
              tokenProgram,
              inventoried: await associatedTokenAddress(payer.address, asset.mint, tokenProgram),
              execution: await associatedTokenAddress(authority.address, asset.mint, tokenProgram),
            };
      const vault =
        tokenAccounts === undefined ? SOL_INTERFACE : (await splInterfaceWithBump(asset.mint))[0];
      if (tokenAccounts !== undefined) {
        const setup = [];
        if ((await account(client, vault)) === null)
          setup.push(
            await createSplInterfaceInstruction({
              authority: payer,
              mint: asset.mint,
              tokenProgram: tokenAccounts.tokenProgram,
            }),
          );
        setup.push(
          await createAssociatedTokenAccountInstruction({
            payer,
            owner: authority.address,
            mint: asset.mint,
            tokenProgram: tokenAccounts.tokenProgram,
          }),
        );
        const created = await land(client, payer, setup, [], 200_000);
        expect(created.meta.err).toBeNull();
        expect(await account(client, vault)).not.toBeNull();
      }

      prepareRing.mockClear();
      prepareRingSpp.mockClear();
      prepareReservation.mockClear();
      const descriptor = await prepareActualOutputDescriptor({
        client,
        ringProgramId,
        feePayer: payer.address,
        depositor: authority,
        recipient: {
          kind: "anonymous",
          keys: ringKeys(owner),
          listSalt: await ringKeys(owner).listSalt(ringProgramId),
        },
        asset: asset.mint,
        ...(tokenAccounts === undefined
          ? {}
          : {
              splTokenAccount: tokenAccounts.execution,
              splTokenProgram: tokenAccounts.tokenProgram,
            }),
      });
      expect(prepareRing).toHaveBeenCalledTimes(1);
      expect(prepareRingSpp).not.toHaveBeenCalled();
      expect(prepareReservation).toHaveBeenCalledTimes(1);
      const reservationFor = (ownerUtxoHash: typeof descriptor.ownerUtxoHash) =>
        actualOutputReservationAddresses(ringProgramId, ownerUtxoHash);
      const reservation = await reservationFor(descriptor.ownerUtxoHash);
      const addressTree = await account(client, reservation.addressTree);
      if (addressTree === null) throw new Error("the canonical SPP address tree is missing");
      const { feePerTree: reservationFee } = decodeTreeFees(
        Buffer.from(addressTree.data[0], "base64"),
      ).fees;
      const isFee = asset.mint === fee;
      const actual = tokenAccounts === undefined ? 2_000_000n : 2_000n;
      const keepLamports = tokenAccounts === undefined ? 1_000_000n : 0n;
      const funded = actual + keepLamports;
      const minimum =
        tokenAccounts === undefined || asset.mint === classic ? actual : FIXTURE_MINIMUM;
      const funding =
        tokenAccounts === undefined
          ? getTransferSolInstruction({
              source: payer,
              destination: authority.address,
              amount: funded,
            })
          : getTransferCheckedInstruction(
              {
                source: tokenAccounts.inventoried,
                mint: asset.mint,
                destination: tokenAccounts.execution,
                authority: payer,
                amount: actual,
                decimals: 0,
              },
              { programAddress: tokenAccounts.tokenProgram },
            );
      const settlement = (minimumOutput: bigint, input = descriptor, signer = authority.address) =>
        buildSettleActualOutputInstruction({
          descriptor: input,
          authority: signer,
          reservationPayer: payer.address,
          minimumOutput,
          ...(tokenAccounts === undefined
            ? { keepLamports }
            : {
                sourceTokenAccount: tokenAccounts.execution,
                tokenProgram: tokenAccounts.tokenProgram,
              }),
        });
      const source = tokenAccounts?.execution ?? authority.address;
      const state = async () => ({
        vault: await balance(client, vault, asset),
        source: await balance(client, source, asset),
        inventory:
          tokenAccounts === undefined
            ? undefined
            : await balance(client, tokenAccounts.inventoried, asset),
        tree: createHash("sha256")
          .update((await account(client, tree))?.data[0] ?? "")
          .digest("hex"),
        addressTree: createHash("sha256")
          .update((await account(client, reservation.addressTree))?.data[0] ?? "")
          .digest("hex"),
        reservation: await account(client, reservation.nullifierPda),
      });

      const initial = await state();
      expect(initial.source).toBe(0n);
      expect(initial.reservation).toBeNull();
      const rejection = async (
        label: string,
        instruction: Instruction,
        signer: TransactionPartialSigner,
        expected?: unknown,
      ) => {
        const rejected = await land(client, payer, [funding, instruction], [signer]);
        expect(rejected.meta.err).not.toBeNull();
        if (expected !== undefined) expect(rejected.meta.err).toEqual(expected);
        expect(await state()).toEqual(initial);
        metric("rejection", { label, asset: asset.name, signature: rejected.signature });
        return rejected;
      };

      if (tokenAccounts === undefined) {
        await rejection(
          "below minimum",
          await settlement(actual + 1n),
          authority,
          RING_SETTLEMENT_ERROR,
        );
        const badTree = await settlement(minimum, {
          ...descriptor,
          outputTree: (await generateKeyPairSigner()).address,
        });
        await rejection("wrong tree", badTree, authority, RING_SETTLEMENT_ERROR);

        const invalid = descriptor.certificate.slice();
        invalid[0] = 0;
        await expect(
          settlement(minimum, { ...descriptor, certificate: invalid }),
        ).rejects.toMatchObject({ code: "RING_TRANSACT_DATA_INVALID" });

        const mismatched = { ...descriptor, ownerUtxoHash: descriptor.zeroChangeOwnerUtxoHash };
        const invalidReservation = await reservationFor(mismatched.ownerUtxoHash);
        await rejection(
          "wrong owner commitment",
          await settlement(minimum, mismatched),
          authority,
          RING_SETTLEMENT_ERROR,
        );
        expect(await account(client, invalidReservation.nullifierPda)).toBeNull();
      } else if (asset.mint === classic) {
        const outsider = await generateKeyPairSigner();
        const wrongAuthority = await settlement(minimum, descriptor, outsider.address);
        const authorityError = await rejection(
          "wrong token source authority",
          wrongAuthority,
          outsider,
        );
        expect(
          authorityError.meta.logMessages?.some((line) =>
            line.startsWith(`Program ${SHIELDED_POOL_PROGRAM_ID} failed:`),
          ),
        ).toBe(true);
        const correct = await settlement(minimum);
        const replaceAccount = (from: Address, to: Address): Instruction => ({
          ...correct,
          accounts:
            correct.accounts?.map((meta) =>
              meta.address === from ? { ...meta, address: to } : meta,
            ) ?? [],
        });
        await rejection(
          "wrong mint",
          replaceAccount(asset.mint, address(metadata)),
          authority,
          RING_SETTLEMENT_ERROR,
        );
        const vaultError = await rejection(
          "wrong settlement vault",
          replaceAccount(vault, outsider.address),
          authority,
        );
        expect(
          vaultError.meta.logMessages?.some((line) =>
            line.startsWith(`Program ${SHIELDED_POOL_PROGRAM_ID} failed:`),
          ),
        ).toBe(true);
      }

      const instruction = await settlement(minimum);
      const payerBefore = await client.getBalance(payer.address);
      const result = await land(client, payer, [funding, instruction], [authority]);
      if (isFee) {
        expect(result.meta.err).toEqual(SPP_SETTLEMENT_ERROR);
        expect(
          result.meta.logMessages?.some((line) =>
            line.startsWith(`Program ${SHIELDED_POOL_PROGRAM_ID} failed:`),
          ),
        ).toBe(true);
        expect(await state()).toEqual(initial);
        expect(payerBefore - (await client.getBalance(payer.address))).toBe(result.meta.fee);
        metric("upstream transfer fee rejection", { signature: result.signature, error: 7010 });
        continue;
      }
      expect(result.meta.err).toBeNull();
      const after = await state();
      expect(after.vault - initial.vault).toBe(actual);
      expect(after.source).toBe(keepLamports);
      if (after.inventory !== undefined && initial.inventory !== undefined)
        expect(initial.inventory - after.inventory).toBe(actual);
      expect(after.reservation?.owner).toBe(SHIELDED_POOL_PROGRAM_ID);
      expect(after.reservation?.space).toBe(11n);
      expect(after.addressTree).not.toBe(initial.addressTree);
      expect(after.tree).not.toBe(initial.tree);
      expect(payerBefore - (await client.getBalance(payer.address))).toBe(
        result.meta.fee + reservationFee + (tokenAccounts === undefined ? funded : 0n),
      );
      expect(result.meta.computeUnitsConsumed).toBeLessThan(BigInt(CU_LIMIT));
      const replay = await land(
        client,
        payer,
        [funding, await settlement(minimum - 1n)],
        [authority],
      );
      expect(replay.signature).not.toBe(result.signature);
      expect(replay.meta.err).toEqual(SPP_RESERVATION_EXISTS);
      expect(await state()).toEqual(after);
      successes.set(result.signature, { asset, actual, certificate: descriptor.certificate });
      actuals.set(asset.mint, actual);
      metric("settlement", {
        asset: asset.name,
        signature: result.signature,
        minimum: String(minimum),
        actual: String(actual),
        wireBytes: result.wireBytes,
        computeUnits: String(result.meta.computeUnitsConsumed),
        feeLamports: String(result.meta.fee),
        reservationFeeLamports: String(reservationFee),
        replaySignature: replay.signature,
      });
    }
    prepareRing.mockRestore();
    prepareRingSpp.mockRestore();
    prepareReservation.mockRestore();

    const auditor = await auditKey(fixture);
    try {
      const policy = await fetchRingPolicy(client, ringProgramId);
      expect(auditor.publicKey().equals(policy.auditorPublicKey)).toBe(true);
      const rows = await client.getShieldedTransactionsByTags(
        { tags: [], ringProgramId, limit: 32 },
        {
          ...client.indexerConfig,
          requireSlot: await client.getSlot(),
        },
      );
      const verified = new Set();
      for (const event of rows.data.flatMap((row) => row.events)) {
        const expected = successes.get(event.txSignature);
        if (expected === undefined) continue;
        const opening = await openActualOutputForAuditor({
          transaction: event,
          solanaRpc: client.solanaRpc,
          auditor,
        });
        if (opening === undefined) continue;
        expect(opening.mint).toBe(expected.asset.mint);
        expect(opening.amount).toBe(expected.actual);
        const output = event.outputSlots[0];
        if (output === undefined) throw new Error("missing measured event output");
        const { encrypted } = decodeRingDepositOutput(readOutputFrame(output.payload).body);
        expect(encrypted.ciphertext.subarray(0, RECORD_TAG.length)).toEqual(RECORD_TAG);
        expect(encrypted.ciphertext[RECORD_TAG.length]).toBe(1);
        expect(encrypted.ciphertext.subarray(RECORD_TAG.length + 1)).toEqual(expected.certificate);
        verified.add(event.txSignature);
      }
      expect(verified.size).toBe(successes.size);
      metric("canonical auditor", { openedEvents: verified.size });
    } finally {
      auditor.destroy();
    }

    const ringRpc = new RingRpc(loopback(fixture.ringRpcUrl), { allowInsecureHttp: true });
    await enroll({ owner, ring: { client, ringProgramId, ringRpc } });
    await promisify(execFile)(fixture.switchCommand);
    for (const service of fixture.closedServices)
      await expect(
        fetch(loopback(service), { signal: AbortSignal.timeout(2_000) }),
      ).rejects.toThrow();

    const independentNetwork = await readJson<Endpoints>(fixture.independentNetworkFile);
    expect(independentNetwork.rpcUrl).toBe(rpcUrl);
    expect(independentNetwork.indexerUrl).not.toBe(indexerUrl);
    expect(independentNetwork.proverUrl).not.toBe(proverUrl);
    const originReads = { count: 0 };
    const independent = await measuredClient(
      fixture,
      {
        rpcUrl: independentNetwork.rpcUrl,
        indexerUrl: independentNetwork.indexerUrl,
        proverUrl: independentNetwork.proverUrl,
      },
      originReads,
    );
    const recovered = freshWallet(independent, owner);
    const document = ledger(recovered, ringProgramId);
    await scan(independent, recovered, document);
    for (const asset of [sol, ...tokens])
      expect(hold(recovered, ringProgramId, asset)).toBe(actuals.get(asset.mint) ?? 0n);
    expect(originReads.count).toBeGreaterThanOrEqual(successes.size);

    for (const { asset, actual } of successes.values()) {
      const tokenProgram = asset.tokenProgram;
      const recipient = exitRecipient.address;
      const [vault, exitAccount] =
        tokenProgram === undefined
          ? [SOL_INTERFACE, recipient]
          : await Promise.all([
              splInterfaceWithBump(asset.mint).then(([address]) => address),
              associatedTokenAddress(recipient, asset.mint, tokenProgram),
            ]);
      const amounts = asset.mint === metadata ? [750n, actual - 750n] : [actual];
      for (const amount of amounts) {
        const left = hold(recovered, ringProgramId, asset);
        const beforeVault = await balance(independent, vault, asset);
        const beforeRecipient =
          (await account(independent, exitAccount)) === null
            ? 0n
            : await balance(independent, exitAccount, asset);
        const withdrawal = await createRingWithdrawalSubmission({
          client: independent,
          ringProgramId,
          wallet: recovered.wallet,
          keys: ringKeys(recovered),
          feePayer: exitPayer.address,
          recipient,
          amount,
          asset: asset.mint,
          ...(tokenProgram === undefined ? {} : { splTokenProgram: tokenProgram }),
          anonymous: {
            signer: messageSignerSpendSigner(recovered.signer),
            reply: null,
            ...(tokenProgram === undefined ? {} : { exitTokenOwner: recipient }),
          },
        });
        const confirmed = await settle(withdrawal, independent, [exitPayer]);
        const chain = await finalized(independent, confirmed.signature);
        expect(chain.meta.err).toBeNull();
        const publicReceived = (await balance(independent, exitAccount, asset)) - beforeRecipient;
        expect(beforeVault - (await balance(independent, vault, asset))).toBe(amount);
        expect(publicReceived).toBe(amount);
        const cold = freshWallet(independent, owner);
        await scan(independent, cold, ledger(cold, ringProgramId));
        expect(hold(cold, ringProgramId, asset)).toBe(left - amount);
        await scan(independent, recovered, document);
        expect(hold(recovered, ringProgramId, asset)).toBe(left - amount);
        metric("independent exit", {
          asset: asset.name,
          signature: confirmed.signature,
          slot: String(chain.slot),
          amount: String(amount),
          publicReceived: String(publicReceived),
          remaining: String(left - amount),
        });
      }
    }
  }, 900_000);
});
