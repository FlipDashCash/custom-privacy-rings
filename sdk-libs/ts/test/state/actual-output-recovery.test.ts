import { beforeAll, describe, expect, it, vi } from "vitest";
import { getBase58Decoder } from "@solana/kit";

import { initializePoseidon } from "@heliuslabs/spr";
import {
  addressBytes,
  hashBytes,
  poseidon,
  ringAuthAddress,
  Writer,
  ZERO_32,
} from "@heliuslabs/spr/internal";
import {
  InstructionTag,
  SHIELDED_POOL_PROGRAM_ID,
  treeIdField,
  UTXO_DOMAIN,
} from "@heliuslabs/spr/interface";
import {
  AssetRegistry,
  SOL_MINT,
  Wallet,
  type IndexedShieldedTransaction,
} from "@heliuslabs/spr/transaction";
import {
  ACTUAL_OUTPUT_MAGIC,
  decodeActualOutputEvent,
  openActualOutputForAuditor,
  prepareActualOutputDescriptor,
} from "../../src/operations/payments/actual-output.js";
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
import {
  itemAt,
  PROOF_SERVICE,
  ringFixture,
  small,
  TREE,
  TREE_ID,
} from "../helpers/ring-pipeline.js";
import { keypair, ScanIndexer, signature } from "../helpers/ring-scan.js";

const base58 = getBase58Decoder();
const ACTUAL_AMOUNT = 9_007_199_254_740_993n;

beforeAll(initializePoseidon);

function keysFor(seed: number): LocalRingKeys {
  const original = keypair(seed);
  try {
    return LocalRingKeys.fromKeypair(original);
  } finally {
    original.destroy();
  }
}

async function fixture() {
  const original = keysFor(39);
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
  ring.enroll(original.address());
  const deposit = await prepareActualOutputDescriptor({
    client: {
      ...ring.client,
      proofDataSource: "prover",
      proofService: PROOF_SERVICE,
      getLatestBlockhash: async () => BLOCKHASH,
    },
    ringProgramId,
    feePayer: ring.authority,
    recipient: {
      kind: "anonymous",
      keys: original,
      listSalt: await original.listSalt(ringProgramId),
    },
  });
  original.destroy();
  const data = RingTransactData.parse(deposit.certificate.subarray(1), "anonymous");
  const value = new Uint8Array(32);
  new DataView(value.buffer).setBigUint64(24, ACTUAL_AMOUNT);
  const mint = hashBytes(addressBytes(SOL_MINT));
  const ringHash = poseidon([ZERO_32, hashBytes(addressBytes(ringProgramId))]);
  const hash = poseidon([
    small(UTXO_DOMAIN),
    treeIdField(TREE_ID),
    mint,
    value,
    ZERO_32,
    ringHash,
    deposit.ownerUtxoHash,
  ]);
  const ciphertext = new Writer()
    .bytes(ACTUAL_OUTPUT_MAGIC)
    .u8(deposit.slot, "slot")
    .bytes(deposit.certificate)
    .finish();
  const body = new Writer()
    .u8(8, "scheme")
    .bytes(deposit.ownerUtxoHash)
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
    viewTag: itemAt(data.outputs, deposit.slot).ownerTag,
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
  return { row, slot, canonical, ring };
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
    const { row, slot, canonical, ring } = await fixture();
    const { client, indexer, getTransaction } = recoveryClient(row, canonical);
    const keys = keysFor(39);
    const document = wallet(keys, row);
    const scan = AnonymousSync.create(keys, document.wallet.registry);

    const recovered = await scan.fetch(client, document.history());
    expect(recovered.utxos.map((note) => note.utxo.amount)).toEqual([ACTUAL_AMOUNT]);
    const nullifier = itemAt(recovered.utxos, 0).nullifier;
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
    const { row, slot, canonical } = await fixture();
    const keys = keysFor(39);
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

    const stranger = keysFor(40);
    const unrelated = recoveryClient(row, null);
    const found = await AnonymousSync.create(stranger, new AssetRegistry()).fetch(
      unrelated.client,
      wallet(stranger, row).history(),
    );
    expect(found.utxos).toEqual([]);
    expect(unrelated.getTransaction).not.toHaveBeenCalled();
  });
});
