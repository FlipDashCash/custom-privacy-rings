/** Canonical finalized-chain provenance for runtime-amount ring deposits. */
import {
  Writer,
  addressBytes,
  concatBytes,
  decodeRingDepositOutput,
  EncryptedScheme,
  equalBytes,
  isZero,
  readOutputFrame,
  ringAuthAddress,
  treeAddress,
  type RingDepositOutput,
  type SolanaRpc,
} from "@heliuslabs/spr/internal";
import { InstructionTag, type Bytes32, type RequestContext } from "@heliuslabs/spr/interface";
import { X25519PublicKey, type X25519Keypair } from "@heliuslabs/spr/keypair";
import {
  SOL_MINT,
  ownerUtxoHash,
  type AssetRegistry,
  type IndexedShieldedTransaction,
  type OutputSlot,
} from "@heliuslabs/spr/transaction";
import { parseAuditorMessage } from "../crypto/audit/audit.js";
import {
  canonicalViewingKey,
  openOutputSlot,
  openRingDisclosure,
} from "../crypto/audit/ring-disclosure.js";
import { openRecipientNote, type OpenedRingNote } from "../crypto/derivation/blinding.js";
import { anonymousSlotTag, txKeySlotSecret } from "../crypto/derivation/ring-stream.js";
import { RecordKind, type OutputRecord } from "../protocol/codec/audit-stream.js";
import { RingError } from "../protocol/error.js";
import {
  ACTUAL_OUTPUT_FORMAT,
  RingInstructionTag,
} from "../protocol/transaction/instruction-tag.js";
import {
  confirmedInstructionGroups,
  originSlot,
  readOriginTransaction,
  ringEventIn,
} from "../protocol/transaction/origin.js";
import {
  RingTransactData,
  parseMeasuredCertificate,
  taggedX25519,
} from "../protocol/transaction/transact.js";
import type { RingKeys } from "./ring-keys.js";

const RECOVERY_MARKER = new TextEncoder().encode("CRADv001");
const DESCRIPTOR_SLOT = 1;

function decodeActualOutputEvent(payload: Uint8Array): RingDepositOutput | undefined {
  if (payload[0] !== 1 || payload[5] !== EncryptedScheme.ringDeposit) return undefined;
  const event = decodeRingDepositOutput(readOutputFrame(payload).body);
  if (!equalBytes(event.encrypted.ciphertext.subarray(0, RECOVERY_MARKER.length), RECOVERY_MARKER))
    return undefined;
  if (event.dataHash !== undefined || !isZero(event.ringDataHash) || event.amount <= 0n)
    throw invalid();
  return event;
}

interface ActualOutputCandidate {
  readonly transaction: IndexedShieldedTransaction;
  readonly event: RingDepositOutput;
  readonly slot: OutputSlot;
  readonly eventIndex: number;
}

/** Recognize the proofless SPP output in the full Photon by-ring transaction row. */
export function actualOutputCandidate(
  transaction: IndexedShieldedTransaction,
): ActualOutputCandidate | undefined {
  const { eventIndex, ringProgramId, outputSlots } = transaction;
  const event = outputSlots.map(({ payload }) => decodeActualOutputEvent(payload)).find(Boolean);
  if (event === undefined) return undefined;
  const [slot] = outputSlots;
  if (
    outputSlots.length !== 1 ||
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

/** Candidate decryption filters unrelated rows before reading finalized Solana history. */
export async function openFinalizedActualOutput(
  candidate: ActualOutputCandidate,
  input: Readonly<{
    solanaRpc?: SolanaRpc | undefined;
    keys: RingKeys;
    assets: AssetRegistry;
    viewingPublicKeys: readonly X25519PublicKey[];
    checkViewTag?: boolean;
  }>,
  context?: RequestContext,
): Promise<OpenedRingNote | undefined> {
  const { event, slot } = candidate;
  const cert = descriptorOf(event);
  const owner = input.keys.address();
  const reader = {
    owner,
    ring: event.ringProgramId,
    treeId: slot.outputContext.treeId,
    assets: input.assets,
  };
  const secrets = await input.keys.ringDeliverySecrets(
    input.viewingPublicKeys.map((viewingPublicKey) => ({ ...cert.delivery, viewingPublicKey })),
    context,
  );
  let note: OpenedRingNote | undefined;
  try {
    for (const secret of secrets) {
      if (input.checkViewTag && !equalBytes(anonymousSlotTag(secret), slot.viewTag)) continue;
      try {
        // Authenticate the synthetic descriptor before replacing its amount.
        const opened = openDescriptorRecord(cert.body, event.asset, secret);
        const synthetic = openRecipientNote({ ...reader, commitment: cert.commitment }, opened);
        if (
          synthetic === undefined ||
          !equalBytes(
            ownerUtxoHash(owner.ownerHash(), synthetic.utxo.blinding),
            event.ownerUtxoHash,
          )
        )
          continue;
        note = openRecipientNote(
          { ...reader, commitment: slot.outputContext.hash },
          { ...opened, record: { ...opened.record, amount: event.amount } },
        );
      } catch {
        continue;
      }
      if (note !== undefined) break;
    }
  } finally {
    for (const secret of secrets) secret.fill(0);
  }
  if (note !== undefined) await verifyFinalizedOrigin(candidate, input.solanaRpc, context);
  return note;
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
  const { event } = candidate;
  const { data, delivery, body } = descriptorOf(event);
  const layout = RingTransactData.streamLayout(data);
  const disclosure = openRingDisclosure(input.auditor, {
    layout,
    salt: delivery.salt,
    message: parseAuditorMessage(RingTransactData.auditorMessage(data), layout, 0),
  });
  const txKey = canonicalViewingKey(disclosure.envelope.txViewingSecret);
  let secret: Bytes32 | undefined;
  try {
    const recipient = disclosure.recipientKeys[DESCRIPTOR_SLOT];
    if (recipient === undefined || !txKey.publicKey().equals(delivery.txViewingPublicKey))
      throw invalid();
    secret = txKeySlotSecret({
      ...delivery,
      txViewingKey: txKey,
      recipientViewingKey: X25519PublicKey.fromBytes(taggedX25519(recipient)),
    });
    const opened = openDescriptorRecord(body, event.asset, secret);
    return Object.freeze({ ...opened.record, amount: event.amount });
  } finally {
    secret?.fill(0);
    txKey.destroy();
  }
}

function descriptorOf(event: RingDepositOutput) {
  const ciphertext = event.encrypted.ciphertext;
  if (ciphertext[RECOVERY_MARKER.length] !== DESCRIPTOR_SLOT) throw invalid();
  const data = parseMeasuredCertificate(ciphertext.subarray(RECOVERY_MARKER.length + 1));
  const output = data.outputs[DESCRIPTOR_SLOT];
  const txViewingKey = taggedX25519(data.txViewingU);
  if (
    output === undefined ||
    !equalBytes(event.encrypted.salt, data.salt) ||
    !equalBytes(event.encrypted.txViewingPublicKey, txViewingKey)
  )
    throw invalid();
  return {
    data,
    commitment: output.hash,
    delivery: {
      txViewingPublicKey: X25519PublicKey.fromBytes(txViewingKey),
      salt: data.salt,
      outputIndex: DESCRIPTOR_SLOT,
      firstNullifier: RingTransactData.firstNullifier(data),
    },
    body: concatBytes(output.record, output.body.payload),
  };
}

function openDescriptorRecord(
  body: Uint8Array,
  asset: RingDepositOutput["asset"],
  secret: Bytes32,
) {
  const opened = openOutputSlot(secret, body, { data: false, ringData: false });
  if (
    opened.record.kind !== RecordKind.ring ||
    opened.record.amount !== 1n ||
    opened.record.mint !== asset
  )
    throw invalid();
  return opened;
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
  if (originSlot(canonical) !== transaction.slot) throw invalid();
  const found = ringEventIn(confirmedInstructionGroups(canonical), eventIndex, event.ringProgramId);
  const auth = await ringAuthAddress(event.ringProgramId);
  const [tag, format] = found?.wrapper?.data ?? [];
  if (
    found?.source.data[0] !== InstructionTag.ringDeposit ||
    tag !== RingInstructionTag.settleActualOutput ||
    format !== ACTUAL_OUTPUT_FORMAT ||
    found.source.accounts[0] !== slot.outputContext.tree ||
    found.source.accounts[2] !== auth ||
    transaction.ringConfig !== auth ||
    !equalBytes(found.event.data, canonicalDepositEvent(slot, event))
  )
    throw invalid();
}

/** Exact Borsh GeneralEvent emitted by the one-output SPP ring_deposit under tag 67. */
function canonicalDepositEvent(slot: OutputSlot, event: RingDepositOutput): Uint8Array {
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
