import { beforeAll, describe, expect, it, vi } from "vitest";
import {
  createSignableMessage,
  generateKeyPairSigner,
  getBase58Decoder,
  type MessagePartialSigner,
} from "@solana/kit";

import { initializePoseidon } from "@heliuslabs/spr";
import { addressBytes, ringAuthAddress, Writer, ZERO_32 } from "@heliuslabs/spr/internal";
import { InstructionTag, SHIELDED_POOL_PROGRAM_ID, type Bytes32 } from "@heliuslabs/spr/interface";
import { ed25519DerivationMessage, ViewingKey } from "@heliuslabs/spr/keypair";
import {
  AssetRegistry,
  LocalShieldedKeys,
  SOL_MINT,
  Utxo,
  Wallet,
  ownerUtxoHash,
  type IndexedShieldedTransaction,
} from "@heliuslabs/spr/transaction";
import {
  ACTUAL_OUTPUT_MAGIC,
  decodeActualOutputEvent,
  openActualOutputForAuditor,
} from "../../src/operations/payments/actual-output.js";
import {
  PreparedRingDeposit,
  type RingDepositClient,
} from "../../src/operations/payments/deposit.js";
import { RingInstructionTag } from "../../src/protocol/transaction/instruction-tag.js";
import { RingTransactData } from "../../src/protocol/transaction/transact.js";
import { ringPolicyPda } from "../../src/protocol/pda/index.js";
import { AnonymousSync } from "../../src/state/anonymous-sync.js";
import { AnonymousWallet } from "../../src/state/anonymous-wallet.js";
import { LocalRingKeys } from "../../src/state/ring-keys.js";
import { syncRingWallet } from "../../src/state/ring-sync.js";
import { ViewingHistory } from "../../src/state/viewing-history.js";
import { BLOCKHASH, solanaRpcReads } from "../../../../zolana/sdk-libs/ts/test/helpers/clients.js";
import { ownedAccount, ringPolicyData } from "../helpers/ring-accounts.js";
import { itemAt, PROOF_SERVICE, ringFixture, TREE, TREE_ID } from "../helpers/ring-pipeline.js";
import { ScanIndexer, signature } from "../helpers/ring-scan.js";

const base58 = getBase58Decoder();
const ACTUAL_AMOUNT = 9_007_199_254_740_993n;

beforeAll(initializePoseidon);

async function recoverKeys(signer: MessagePartialSigner): Promise<LocalRingKeys> {
  const solanaPublicKey = signer.address;
  const [signed] = await signer.signMessages([
    createSignableMessage(ed25519DerivationMessage(addressBytes(solanaPublicKey) as Bytes32)),
  ]);
  const signature = signed?.[solanaPublicKey];
  if (signature === undefined) throw new Error("missing derivation signature");
  const derivationSeed = Uint8Array.from(signature);
  const roles = LocalShieldedKeys.fromDerivationSeed({ solanaPublicKey, derivationSeed });
  const viewing = ViewingKey.fromDerivationSeed(derivationSeed);
  try {
    return roles.withNullifierKey((nullifierKey) =>
      LocalRingKeys.fromKeys({
        address: roles.address(),
        viewingKeys: [viewing],
        listRoot: viewing,
        nullifierKey,
      }),
    );
  } finally {
    roles.destroy();
    viewing.destroy();
    derivationSeed.fill(0);
  }
}

async function fixture() {
  const signingKey = await generateKeyPairSigner();
  const original = await recoverKeys(signingKey);
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
      }),
    ),
  );
  const listSalt = await original.listSalt(ringProgramId);
  ring.enroll(original.address());
  // Use the stock deposit frame and cipher. Chain provenance and Groth16 are fixture data.
  const proofClient: RingDepositClient = {
    ...ring.client,
    proofDataSource: "prover",
    proofService: PROOF_SERVICE,
    getLatestBlockhash: async () => BLOCKHASH,
  };
  const prepared = await PreparedRingDeposit.prepare({
    client: proofClient,
    ringProgramId,
    feePayer: ring.authority,
    recipient: { kind: "anonymous", keys: original, listSalt },
    amount: 1n,
  });
  const proved = await prepared.proveForComposition(proofClient, () => {});
  const certificate = (await proved.instruction()).data;
  if (certificate === undefined) throw new Error("missing stock descriptor certificate");
  const data = RingTransactData.parse(Uint8Array.from(certificate.subarray(1)), "anonymous");
  const outputs = proved.outputs.filter((output) => !output.isCompact());
  const descriptorSlot = outputs.findIndex((output) => output.amount === 1n && !output.isDummy());
  const synthetic = itemAt(outputs, descriptorSlot);
  const actual = new Utxo({
    owner: original.address().signingPublicKey,
    ringProgramId,
    asset: SOL_MINT,
    amount: ACTUAL_AMOUNT,
    blinding: synthetic.blinding,
  });
  const hash = actual.hash(original.address().nullifierPublicKey, TREE_ID);
  const [nullifier] = await original.derive([
    { kind: "nullifier", utxoHash: hash, blinding: actual.blinding },
  ]);
  if (nullifier === undefined) throw new Error("missing original nullifier");
  const ciphertext = new Writer()
    .bytes(ACTUAL_OUTPUT_MAGIC)
    .u8(descriptorSlot, "slot")
    .bytes(Uint8Array.from(certificate))
    .finish();
  const body = new Writer()
    .u8(8, "scheme")
    .bytes(ownerUtxoHash(synthetic.ownerHash(), synthetic.blinding))
    .bytes(addressBytes(SOL_MINT))
    .u64(ACTUAL_AMOUNT, "amount")
    .u8(0, "data hash")
    .bytes(addressBytes(ringProgramId))
    .bytes(ZERO_32)
    .u8(4, "X25519")
    .bytes(data.txViewingU)
    .bytes(data.salt)
    .u32(ciphertext.length, "ciphertext length")
    .bytes(ciphertext)
    .finish();
  const payload = new Writer()
    .u8(1, "encrypted")
    .u32(body.length, "body length")
    .bytes(body)
    .finish();
  const auth = await ringAuthAddress(ringProgramId);
  const slot = {
    viewTag: itemAt(data.outputs, descriptorSlot).ownerTag,
    payload,
    outputContext: { hash, tree: TREE, treeId: TREE_ID, leafIndex: 39n },
  };
  const row: IndexedShieldedTransaction = {
    txSignature: signature(39n),
    eventIndex: 1,
    slot: 9n,
    proofless: true,
    ringProgramId,
    ringConfig: auth,
    nullifiers: [],
    messages: [],
    outputSlots: [slot],
  };
  const event = new Writer()
    .u8(InstructionTag.emitEvent, "emit")
    .u8(1, "deposit event")
    .u32(0, "inputs")
    .u32(1, "outputs")
    .bytes(slot.viewTag)
    .bytes(hash)
    .u32(payload.length, "payload")
    .bytes(payload)
    .u32(0, "messages")
    .bytes(new Uint8Array(49))
    .u64(slot.outputContext.leafIndex, "first leaf")
    .bytes(addressBytes(TREE))
    .u32(1, "settlement legs")
    .u8(1, "deposit")
    .u64(ACTUAL_AMOUNT, "amount")
    .u8(0, "SOL")
    .finish();
  const emit = { programIdIndex: 0, accounts: [], data: base58.decode(event), stackHeight: 3 };
  const canonical = {
    slot: row.slot,
    transaction: {
      signatures: [row.txSignature],
      message: {
        accountKeys: [SHIELDED_POOL_PROGRAM_ID, ringProgramId, TREE, auth, ring.authority],
        instructions: [
          {
            programIdIndex: 0,
            accounts: [],
            data: base58.decode(Uint8Array.of(InstructionTag.transact)),
          },
          {
            programIdIndex: 1,
            accounts: [],
            data: base58.decode(Uint8Array.of(RingInstructionTag.settleActualOutput)),
          },
        ],
      },
    },
    meta: {
      err: null,
      innerInstructions: [
        { index: 0, instructions: [{ ...emit, stackHeight: 2 }] },
        {
          index: 1,
          instructions: [
            {
              programIdIndex: 0,
              accounts: [2, 4, 3],
              data: base58.decode(Uint8Array.of(InstructionTag.ringDeposit)),
              stackHeight: 2,
            },
            emit,
          ],
        },
      ],
    },
  };
  original.destroy();
  return { row, slot, canonical, signingKey, nullifier, ring };
}

function recoveryClient(row: IndexedShieldedTransaction, canonical: unknown) {
  const indexer = new ScanIndexer();
  indexer.publish(row);
  const getTransaction = vi.fn(() => ({ send: async () => canonical }));
  const client = { ...indexer.client(), solanaRpc: solanaRpcReads({ getTransaction }) };
  return { client, indexer, getTransaction };
}

function wallet(keys: LocalRingKeys, ring: IndexedShieldedTransaction) {
  if (ring.ringProgramId === undefined) throw new Error("missing Ring");
  const address = keys.address();
  return AnonymousWallet.create(
    new Wallet({ identity: address }),
    ViewingHistory.create(address.viewingPublicKey, 0n).withRing(ring.ringProgramId),
  );
}

describe("canonical actual-output recovery", () => {
  it("admits the measured amount from a fresh wallet through both stock ledgers and reconciles its spend", async () => {
    const { row, slot, canonical, signingKey, nullifier, ring } = await fixture();
    const { client, indexer, getTransaction } = recoveryClient(row, canonical);
    const keys = await recoverKeys(signingKey);
    const document = wallet(keys, row);
    const scan = AnonymousSync.create(keys, document.wallet.registry);

    const recovered = await scan.fetch(client, document.history());
    expect(recovered.utxos.map((note) => [note.utxo.amount, note.nullifier])).toEqual([
      [ACTUAL_AMOUNT, nullifier],
    ]);
    expect(recovered.found[0]?.stream.kind).toBe("actualOutput");
    expect(getTransaction).toHaveBeenCalledWith(row.txSignature, {
      encoding: "json",
      commitment: "finalized",
      maxSupportedTransactionVersion: 1,
    });
    await document.absorb(recovered);
    expect(document.notes().map((note) => note.outputContext.hash)).toEqual([
      slot.outputContext.hash,
    ]);

    const core = wallet(keys, row);
    await syncRingWallet({ wallet: core, keys, client });
    expect(core.notes().map((note) => note.nullifier)).toEqual([nullifier]);

    indexer.publish({
      ...row,
      txSignature: signature(40n),
      slot: 10n,
      proofless: false,
      eventIndex: 0,
      outputSlots: [],
      nullifiers: [nullifier],
    });
    const spent = await scan.withHeldNotes(document.notes()).fetch(client, document.history());
    expect(spent.spentNullifiers).toEqual([nullifier]);
    await document.absorb(spent);
    expect(document.notes().every((note) => note.spent)).toBe(true);

    const event = decodeActualOutputEvent(slot.payload);
    if (event === undefined) throw new Error("missing actual-output event");
    expect(openActualOutputForAuditor(event, ring.auditor)).toMatchObject({
      amount: ACTUAL_AMOUNT,
    });
  });

  it("refuses unavailable finality, a different leaf, or an event emitted by the wrong caller", async () => {
    const { row, slot, canonical, signingKey } = await fixture();
    const keys = await recoverKeys(signingKey);
    const document = wallet(keys, row);
    const scan = AnonymousSync.create(keys, new AssetRegistry());
    const read = (event: IndexedShieldedTransaction, record: unknown) =>
      scan.fetch(recoveryClient(event, record).client, document.history());
    await expect(read(row, null)).rejects.toMatchObject({ code: "RING_ORIGIN_UNAVAILABLE" });
    await expect(
      read(
        {
          ...row,
          outputSlots: [
            {
              ...slot,
              outputContext: {
                ...slot.outputContext,
                leafIndex: slot.outputContext.leafIndex + 1n,
              },
            },
          ],
        },
        canonical,
      ),
    ).rejects.toMatchObject({ code: "RING_ORIGIN_DECODE" });
    await expect(read({ ...row, eventIndex: 0 }, canonical)).rejects.toMatchObject({
      code: "RING_ORIGIN_DECODE",
    });
    const wrong = structuredClone(canonical);
    itemAt(wrong.transaction.message.instructions, 1).data = base58.decode(
      Uint8Array.of(RingInstructionTag.transfer),
    );
    await expect(read(row, wrong)).rejects.toMatchObject({ code: "RING_ORIGIN_DECODE" });

    const stranger = await recoverKeys(await generateKeyPairSigner());
    const unrelated = recoveryClient(row, null);
    const found = await AnonymousSync.create(stranger, new AssetRegistry()).fetch(
      unrelated.client,
      wallet(stranger, row).history(),
    );
    expect(found.utxos).toEqual([]);
    expect(unrelated.getTransaction).not.toHaveBeenCalled();
  });
});
