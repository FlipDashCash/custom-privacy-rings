/** Canonical finalized-chain provenance for runtime-amount ring deposits. */
import {
  Writer,
  addressBytes,
  equalBytes,
  ringAuthAddress,
  treeAddress,
  type SolanaRpc,
} from "@heliuslabs/spr/internal";
import { InstructionTag, type RequestContext } from "@heliuslabs/spr/interface";
import type { X25519Keypair, X25519PublicKey } from "@heliuslabs/spr/keypair";
import {
  SOL_MINT,
  type AssetRegistry,
  type IndexedShieldedTransaction,
  type OutputSlot,
} from "@heliuslabs/spr/transaction";
import {
  decodeActualOutputEvent,
  openActualOutputDisclosure,
  openActualOutputForRecipient,
  type ActualOutputEvent,
} from "../operations/payments/actual-output.js";
import type { OpenedRingNote } from "../crypto/derivation/blinding.js";
import type { OutputRecord } from "../protocol/codec/audit-stream.js";
import { RingError } from "../protocol/error.js";
import { RingInstructionTag } from "../protocol/transaction/instruction-tag.js";
import {
  confirmedInstructionGroups,
  readOriginTransaction,
  ringEventIn,
} from "../protocol/transaction/origin.js";
import type { RingKeys } from "./ring-keys.js";

interface ActualOutputCandidate {
  readonly transaction: IndexedShieldedTransaction;
  readonly event: ActualOutputEvent;
  readonly slot: OutputSlot;
  readonly eventIndex: number;
}

/** Recognize the proofless SPP output in the full Photon by-ring transaction row. */
export function actualOutputCandidate(
  transaction: IndexedShieldedTransaction,
): ActualOutputCandidate | undefined {
  const candidates = transaction.outputSlots.map((slot) => decodeActualOutputEvent(slot.payload));
  if (candidates.every((event) => event === undefined)) return undefined;
  const event = candidates[0];
  const slot = transaction.outputSlots[0];
  const { eventIndex, ringProgramId } = transaction;
  if (
    candidates.length !== 1 ||
    event === undefined ||
    slot === undefined ||
    event.ringProgramId !== ringProgramId ||
    !transaction.proofless ||
    eventIndex === undefined ||
    !Number.isInteger(eventIndex) ||
    eventIndex < 0 ||
    eventIndex > 0xffff ||
    transaction.nullifiers.length !== 0 ||
    transaction.messages.length !== 0 ||
    transaction.txViewingPublicKey !== undefined ||
    transaction.salt !== undefined ||
    slot.outputContext.tree !== treeAddress(slot.outputContext.treeId)
  )
    throw invalid();
  return { transaction, event, slot, eventIndex };
}

/** Candidate decryption filters unrelated rows; only a finalized SPP event can credit a note. */
export async function openFinalizedActualOutput(
  candidate: ActualOutputCandidate,
  input: Readonly<{
    solanaRpc: SolanaRpc | undefined;
    keys: RingKeys;
    assets: AssetRegistry;
    viewingPublicKey: X25519PublicKey;
  }>,
  context?: RequestContext,
): Promise<OpenedRingNote | undefined> {
  const { event, slot } = candidate;
  const opened = await openActualOutputForRecipient({
    event,
    treeId: slot.outputContext.treeId,
    commitment: slot.outputContext.hash,
    keys: input.keys,
    assets: input.assets,
    viewingPublicKey: input.viewingPublicKey,
    ...(context === undefined ? {} : { context }),
  });
  if (opened === undefined) return undefined;
  await verifyFinalizedOrigin(candidate, input.solanaRpc, context);
  return opened;
}

/** Opens a measured deposit for the configured auditor after binding its row to finalized Solana history. */
export async function openActualOutputForAuditor(
  input: Readonly<{
    transaction: IndexedShieldedTransaction;
    solanaRpc: SolanaRpc;
    auditor: X25519Keypair;
  }>,
  context?: RequestContext,
): Promise<OutputRecord | undefined> {
  const candidate = actualOutputCandidate(input.transaction);
  if (candidate === undefined) return undefined;
  await verifyFinalizedOrigin(candidate, input.solanaRpc, context);
  return openActualOutputDisclosure(candidate.event, input.auditor);
}

async function verifyFinalizedOrigin(
  candidate: ActualOutputCandidate,
  solanaRpc: SolanaRpc | undefined,
  context?: RequestContext,
): Promise<void> {
  const { transaction, slot, event, eventIndex } = candidate;
  if (solanaRpc === undefined)
    throw new RingError("RING_ORIGIN_UNAVAILABLE", {
      details: { reason: "runtime-amount recovery requires finalized Solana RPC history" },
    });
  const canonical = await readOriginTransaction(
    solanaRpc,
    transaction.txSignature,
    "finalized",
    context,
  );
  if (canonical["slot"] !== transaction.slot) throw invalid();
  const found = ringEventIn(confirmedInstructionGroups(canonical), eventIndex, event.ringProgramId);
  const auth = await ringAuthAddress(event.ringProgramId);
  if (
    found?.source.data[0] !== InstructionTag.ringDeposit ||
    found.wrapper?.data[0] !== RingInstructionTag.settleActualOutput ||
    found.source.accounts[0] !== slot.outputContext.tree ||
    found.source.accounts[2] !== auth ||
    transaction.ringConfig !== auth ||
    !equalBytes(found.event.data, canonicalDepositEvent(slot, event))
  )
    throw invalid();
}

/** Exact Borsh GeneralEvent emitted by the one-output SPP ring_deposit under tag 67. */
function canonicalDepositEvent(slot: OutputSlot, event: ActualOutputEvent): Uint8Array {
  const writer = new Writer()
    .u8(InstructionTag.emitEvent, "tag")
    .u8(1, "event.kind")
    .u32(0, "inputs.length")
    .u32(1, "outputs.length")
    .bytes(slot.viewTag, 32)
    .bytes(slot.outputContext.hash, 32)
    .u32(slot.payload.length, "output.data.length")
    .bytes(slot.payload)
    .u32(0, "messages.length")
    .bytes(new Uint8Array(33), 33)
    .bytes(new Uint8Array(16), 16)
    .u64(slot.outputContext.leafIndex, "firstOutputLeafIndex")
    .bytes(addressBytes(slot.outputContext.tree), 32)
    .u32(1, "splTransfers.length")
    .u8(1, "isDeposit")
    .u64(event.amount, "amount")
    .u8(event.asset === SOL_MINT ? 0 : 1, "asset.option");
  if (event.asset !== SOL_MINT) writer.bytes(addressBytes(event.asset), 32);
  return writer.finish();
}

function invalid(): RingError {
  return new RingError("RING_ORIGIN_DECODE", { details: { path: "actualOutput.finalizedEvent" } });
}
