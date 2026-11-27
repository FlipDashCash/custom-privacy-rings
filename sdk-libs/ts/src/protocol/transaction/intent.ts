/** The ring intents beside the pinned SPP intents. */
import { sha256 } from "@noble/hashes/sha2.js";

import type { Address, Bytes32 } from "@heliuslabs/spr";
import { ShieldedAddress } from "@heliuslabs/spr/keypair";
import {
  addressBytes,
  checkIntentApproval,
  checkPreparedTransfer,
  equalBytes,
  u64be,
} from "@heliuslabs/spr/internal";
import {
  intentHash,
  type ApprovalRequest,
  type IntentApproval,
  type PreparedTransfer,
  type TransactionIntent,
} from "@heliuslabs/spr/transaction";

import { RingAddress } from "../../crypto/keys/ring-address.js";
import { RingError } from "../error.js";

export type RingAnonymousTransferIntent = Readonly<{
  kind: "ringAnonymousTransfer";
  ringProgramId: Address;
  asset: Address;
  amount: bigint;
  /** The address the payment pays, inside `ringProgramId`. */
  recipient: RingAddress;
  /** Default-ring UTXO value the payment moves into the ring. */
  defaultFunding: bigint;
}>;

/** Several payments of one asset inside the ring, each one output of the transfer. */
export type RingMultiTransferIntent = Readonly<{
  kind: "ringMultiTransfer";
  ringProgramId: Address;
  asset: Address;
  /** In the order the caller named them, at least two. */
  payments: readonly Readonly<{ recipient: ShieldedAddress | RingAddress; amount: bigint }>[];
  /** Default-ring UTXO value the payments move into the ring. */
  defaultFunding: bigint;
}>;

export type RingTransactionIntent =
  | TransactionIntent
  | RingAnonymousTransferIntent
  | RingMultiTransferIntent;

export type RingApprovalRequest = Omit<ApprovalRequest, "intent"> &
  Readonly<{ intent: RingTransactionIntent }>;

export type RingApprovalHandler = (request: RingApprovalRequest) => Promise<IntentApproval>;

/** Separates ring intent hashes from core's `"zolana/intent/v1"` and its kind tags. */
const RING_INTENT_DOMAIN = "zolana-custom-rings/intent/v1";
const RING_ANONYMOUS_TRANSFER_TAG = 1;
const RING_MULTI_TRANSFER_TAG = 2;
const MAX_U64 = (1n << 64n) - 1n;

export const approveRingUnattended: RingApprovalHandler = (request) =>
  Promise.resolve(Object.freeze({ intentHash: ringIntentHash(request.intent) }));

/** The pinned `intentHash` for every SPP kind. */
export function ringIntentHash(intent: RingTransactionIntent): Bytes32 {
  if (intent.kind === "ringMultiTransfer") return multiTransferHash(intent);
  if (intent.kind !== "ringAnonymousTransfer") return intentHash(intent);
  if (
    !(intent.recipient instanceof RingAddress) ||
    intent.recipient.ringProgramId !== intent.ringProgramId
  )
    throw new RingError("RING_INTENT_MISMATCH", { details: { field: "recipient" } });
  const recipient = intent.recipient.toBytes();
  const hash = sha256.create();
  hash.update(new TextEncoder().encode(RING_INTENT_DOMAIN));
  hash.update(Uint8Array.of(RING_ANONYMOUS_TRANSFER_TAG));
  hash.update(addressBytes(intent.ringProgramId, "ringProgramId"));
  hash.update(addressBytes(intent.asset, "asset"));
  hash.update(u64be(intent.amount));
  hash.update(Uint8Array.of(recipient.length >> 8, recipient.length & 0xff));
  hash.update(recipient);
  hash.update(u64be(intent.defaultFunding));
  return hash.digest() as Bytes32;
}

export function checkRingIntentApproval(
  approval: IntentApproval,
  intent: RingTransactionIntent,
  mismatch: (field: string) => Error,
): void {
  if (intent.kind !== "ringAnonymousTransfer" && intent.kind !== "ringMultiTransfer") {
    checkIntentApproval(approval, intent, mismatch);
    return;
  }
  const got: unknown = (approval as Partial<IntentApproval> | null)?.intentHash;
  if (!(got instanceof Uint8Array) || got.length !== 32 || !equalBytes(got, ringIntentHash(intent)))
    throw mismatch("intentHash");
}

/** An anonymous payment binds its outputs as a ring transfer to the address's recipient does. */
export function checkRingPreparedTransfer(
  prepared: PreparedTransfer,
  intent: RingTransactionIntent,
  mismatch: (field: string) => Error,
): void {
  if (intent.kind === "ringMultiTransfer") {
    checkPaymentOutputs(prepared, intent, mismatch);
    return;
  }
  // The ring binds an entry's change to itself, so the approved amount counts the change as entered value.
  if (intent.kind === "ringEntry") {
    checkPreparedTransfer({ ...prepared, senderOutputCount: 0 }, intent, mismatch);
    return;
  }
  checkPreparedTransfer(
    prepared,
    intent.kind === "ringAnonymousTransfer"
      ? {
          kind: "ringTransfer",
          ringProgramId: intent.ringProgramId,
          asset: intent.asset,
          amount: intent.amount,
          recipient: intent.recipient.recipient,
          boundary: "transfer",
          defaultFunding: intent.defaultFunding,
        }
      : intent,
    mismatch,
  );
}

/** Every payment names a recipient of the ring and a nonzero u64 amount. */
function multiTransferHash(intent: RingMultiTransferIntent): Bytes32 {
  const { payments } = intent;
  if (payments.length < 2 || payments.length > 0xff)
    throw new RingError("RING_INTENT_MISMATCH", { details: { field: "payments" } });
  const hash = sha256.create();
  hash.update(new TextEncoder().encode(RING_INTENT_DOMAIN));
  hash.update(Uint8Array.of(RING_MULTI_TRANSFER_TAG));
  hash.update(addressBytes(intent.ringProgramId, "ringProgramId"));
  hash.update(addressBytes(intent.asset, "asset"));
  hash.update(Uint8Array.of(payments.length));
  for (const { recipient, amount } of payments) {
    if (amount <= 0n || amount > MAX_U64)
      throw new RingError("RING_INTENT_MISMATCH", { details: { field: "amount" } });
    let bytes: Uint8Array;
    if (recipient instanceof ShieldedAddress) bytes = recipient.toBytes();
    else if (recipient instanceof RingAddress && recipient.ringProgramId === intent.ringProgramId)
      bytes = recipient.toBytes();
    else throw new RingError("RING_INTENT_MISMATCH", { details: { field: "recipient" } });
    hash.update(Uint8Array.of(recipient instanceof RingAddress ? 1 : 0));
    hash.update(Uint8Array.of(bytes.length >> 8, bytes.length & 0xff));
    hash.update(bytes);
    hash.update(u64be(amount));
  }
  hash.update(u64be(intent.defaultFunding));
  return hash.digest() as Bytes32;
}

/** Each payment takes one output of the ring in any order, and nothing else leaves the sender. */
function checkPaymentOutputs(
  prepared: PreparedTransfer,
  intent: RingMultiTransferIntent,
  mismatch: (field: string) => Error,
): void {
  if (prepared.interfaceTransfers.length > 0) throw mismatch("settlements");
  const unmatched = prepared.outputs
    .slice(prepared.senderOutputCount)
    .filter((output) => !output.isDummy());
  if (unmatched.length !== intent.payments.length) throw mismatch("payments");
  for (const { recipient, amount } of intent.payments) {
    const owner = (recipient instanceof RingAddress ? recipient.recipient : recipient).toBytes();
    const index = unmatched.findIndex(
      (output) =>
        output.ownerAddress !== undefined &&
        equalBytes(output.ownerAddress.toBytes(), owner) &&
        output.amount === amount &&
        output.asset === intent.asset &&
        output.ringProgramId === intent.ringProgramId,
    );
    if (index < 0) throw mismatch("payments");
    unmatched.splice(index, 1);
  }
  const defaultFunding = prepared.inputs
    .filter((input) => !input.isDummy() && input.utxo.ringProgramId === undefined)
    .reduce((sum, input) => sum + input.utxo.amount, 0n);
  if (defaultFunding !== intent.defaultFunding) throw mismatch("defaultFunding");
}
