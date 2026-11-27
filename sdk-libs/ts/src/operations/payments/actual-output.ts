/** Audited runtime-amount output settlement using unchanged stock Ring certificates and SPP notes. */
import { getProgramDerivedAddress, type Instruction } from "@solana/kit";
import { initializePoseidon } from "@heliuslabs/spr";
import {
  Writer,
  Reader,
  readOutputFrame,
  EncryptedScheme,
  meta,
  ZERO_32,
  SYSTEM_PROGRAM,
  equalBytes,
  isZero,
  concatBytes,
  ringAuthAddress,
  splInterfaceWithBump,
  encodeBase58,
} from "@heliuslabs/spr/internal";
import {
  SHIELDED_POOL_PROGRAM_ID,
  SOL_INTERFACE,
  SPL_TOKEN_2022_PROGRAM_ID,
  SPL_TOKEN_PROGRAM_ID,
  type Address,
  type Bytes16,
  type Bytes32,
  type Bytes33,
  type RequestContext,
  type EncryptedRingDepositData,
} from "@heliuslabs/spr/interface";
import { X25519PublicKey, ViewingKey, type X25519Keypair } from "@heliuslabs/spr/keypair";
import {
  SOL_MINT,
  ownerUtxoHash,
  type AssetRegistry,
  type TreeId,
} from "@heliuslabs/spr/transaction";
import { fetchRingPolicy, fetchStorageRegistry } from "../../protocol/membership/config.js";
import { ringRootRegistries } from "../../protocol/codec/ring-codecs.js";
import { MEMBER_REGISTRY_EMPTY_ROOT } from "../../protocol/membership/member-opening.js";
import { ringPolicyAddress } from "../../protocol/pda/index.js";
import { RingTransactData } from "../../protocol/transaction/transact.js";
import { RingInstructionTag } from "../../protocol/transaction/instruction-tag.js";
import {
  openOutputSlot,
  openRingDisclosure,
  type OpenedOutput,
} from "../../crypto/audit/ring-disclosure.js";
import { openRecipientNote, type OpenedRingNote } from "../../crypto/derivation/blinding.js";
import { ringIdField, txKeySlotSecret } from "../../crypto/derivation/ring-stream.js";
import { RecordKind, type OutputRecord } from "../../protocol/codec/audit-stream.js";
import { RingError } from "../../protocol/error.js";
import {
  PreparedRingDeposit,
  type RingDepositParams,
  type RingDepositRecipient,
} from "./deposit.js";
import type { RingKeys } from "../../state/ring-keys.js";

export const ACTUAL_OUTPUT_VERSION = 1;
export const ACTUAL_OUTPUT_MAGIC = new TextEncoder().encode("CRADv001");
export const ACTUAL_OUTPUT_MINIMUM_OFFSET = 3;
const DESCRIPTOR_SLOT = 1;

/** Public, proof-authenticated ownership descriptor. Never substitute a real spend as a certificate. */
export interface ActualOutputDescriptor {
  readonly ringProgramId: Address;
  readonly asset: Address;
  readonly outputTree: Address;
  readonly registries: readonly Address[];
  readonly slot: number;
  readonly ownerUtxoHash: Bytes32;
  readonly privateTxBlinding: Bytes32;
  readonly zeroChangeOwnerUtxoHash: Bytes32;
  readonly certificate: Uint8Array;
}

/** Prepares a fresh amount-one all-dummy stock deposit; the synthetic instruction is never dispatched. */
export async function prepareActualOutputDescriptor(
  params: Omit<RingDepositParams, "amount" | "cosigner" | "recipient"> & {
    readonly recipient: Exclude<RingDepositRecipient, { kind: "confidential" }>;
  },
  context?: RequestContext,
): Promise<ActualOutputDescriptor> {
  const policy = await fetchRingPolicy(params.client, params.ringProgramId, context);
  if (policy.mode !== "anonymous" || policy.hasPolicy || policy.coSignerClasses !== undefined)
    throw invalid();
  const registries = ringRootRegistries(policy, "transfer");
  const ownRevision = policy.records === SYSTEM_PROGRAM ? undefined : 0n;
  if (ownRevision !== undefined) {
    await initializePoseidon();
    const records = await fetchStorageRegistry(params.client, policy.records, context);
    const newest = records.roots[0];
    if (
      records.nextIndex !== 1n ||
      records.trackingRegistry !== policy.tracking ||
      records.trackingFloor !== 0n ||
      !equalBytes(records.ringId, ringIdField(params.ringProgramId)) ||
      newest?.revision !== ownRevision ||
      newest.expiresAt !== 0n ||
      !equalBytes(newest.hash, MEMBER_REGISTRY_EMPTY_ROOT)
    )
      throw invalid();
  }
  const prepared = await PreparedRingDeposit.prepare({ ...params, amount: 1n }, context);
  const proven = await prepared.proveForComposition(params.client, () => {}, context);
  if (
    proven.descriptorBlinding === undefined ||
    proven.accounts.registries.length !== registries.length ||
    proven.accounts.registries.some((address, index) => address !== registries[index])
  )
    throw invalid();
  const published = proven.outputs.filter((output) => !output.isCompact());
  const [change, output] = published;
  const asset = params.asset ?? SOL_MINT;
  // Stock anonymous deposits name a participant via a live zero-SOL change slot.
  if (
    published.length !== 2 ||
    change === undefined ||
    change.isDummy() ||
    change.asset !== SOL_MINT ||
    change.amount !== 0n ||
    change.ringProgramId !== params.ringProgramId ||
    output === undefined ||
    output.isDummy() ||
    output.asset !== asset ||
    output.amount !== 1n ||
    output.ringProgramId !== params.ringProgramId
  )
    throw invalid();
  const instruction = await proven.instruction();
  const certificate = Uint8Array.from(instruction.data ?? []);
  const frame = RingTransactData.parse(certificate.subarray(1), "anonymous");
  if (frame.ownRevision !== ownRevision || frame.approval !== 0) throw invalid();
  return Object.freeze({
    ringProgramId: params.ringProgramId,
    asset,
    outputTree: proven.accounts.outputTree,
    registries: proven.accounts.registries,
    slot: DESCRIPTOR_SLOT,
    ownerUtxoHash: ownerUtxoHash(output.ownerHash(), output.blinding),
    privateTxBlinding: proven.descriptorBlinding.slice() as Bytes32,
    zeroChangeOwnerUtxoHash: ownerUtxoHash(change.ownerHash(), change.blinding),
    certificate,
  });
}

export interface ActualOutputSettlementParams {
  readonly descriptor: ActualOutputDescriptor;
  /** Wallet-client-only ephemeral operation payer/trade authority, signs the entire reviewed message. */
  readonly authority: Address;
  readonly receiptPayer: Address;
  readonly minimumOutput: bigint;
  readonly keepLamports?: bigint;
  readonly sourceTokenAccount?: Address;
  readonly tokenProgram?: Address;
}

export async function actualOutputReceiptAddress(ring: Address, owner: Bytes32): Promise<Address> {
  return (
    await getProgramDerivedAddress({
      programAddress: ring,
      seeds: [new TextEncoder().encode("actual_output"), owner],
    })
  )[0];
}

/** No dispatch/signing. Compose after the stock private withdrawal and reviewed venue instructions. */
export async function buildSettleActualOutputInstruction(
  params: ActualOutputSettlementParams,
): Promise<Instruction> {
  const { descriptor: d } = params;
  if (
    params.minimumOutput <= 0n ||
    (params.keepLamports ?? 0n) < 0n ||
    d.slot !== DESCRIPTOR_SLOT ||
    (d.registries.length !== 1 && d.registries.length !== 2)
  )
    throw invalid();
  const [ring, ringAuth, receipt] = await Promise.all([
    ringPolicyAddress(d.ringProgramId),
    ringAuthAddress(d.ringProgramId),
    actualOutputReceiptAddress(d.ringProgramId, d.ownerUtxoHash),
  ]);
  const out = new Writer()
    .u8(RingInstructionTag.settleActualOutput, "tag")
    .u8(ACTUAL_OUTPUT_VERSION, "version")
    .u8(d.slot, "slot")
    .u64(params.minimumOutput, "minimumOutput")
    .u64(params.keepLamports ?? 0n, "keepLamports")
    .bytes(d.ownerUtxoHash, 32)
    .bytes(ZERO_32, 32)
    .bytes(d.privateTxBlinding, 32)
    .bytes(d.zeroChangeOwnerUtxoHash, 32);
  const group = [];
  if (d.asset === SOL_MINT) {
    if (params.sourceTokenAccount !== undefined || params.tokenProgram !== undefined)
      throw invalid();
    out.u8(0, "asset");
    group.push(meta(SYSTEM_PROGRAM, false, false), meta(SOL_INTERFACE, false, true));
  } else {
    if (
      params.sourceTokenAccount === undefined ||
      (params.tokenProgram !== SPL_TOKEN_PROGRAM_ID &&
        params.tokenProgram !== SPL_TOKEN_2022_PROGRAM_ID) ||
      (params.keepLamports ?? 0n) !== 0n
    )
      throw invalid();
    const [splInterface, bump] = await splInterfaceWithBump(d.asset);
    out.u8(1, "asset").u8(bump, "splInterfaceBump");
    group.push(
      meta(params.tokenProgram, false, false),
      meta(d.asset, false, false),
      meta(params.sourceTokenAccount, false, true),
      meta(splInterface, false, true),
    );
  }
  out.u16(d.certificate.length, "certificate.length").bytes(d.certificate);
  return Object.freeze({
    programAddress: d.ringProgramId,
    accounts: [
      meta(ring, false, false),
      ...d.registries.map((address) => meta(address, false, false)),
      meta(receipt, false, true),
      meta(params.receiptPayer, true, true),
      meta(SYSTEM_PROGRAM, false, false),
      meta(d.outputTree, false, true),
      meta(params.authority, true, true),
      meta(ringAuth, false, false),
      meta(SHIELDED_POOL_PROGRAM_ID, false, false),
      ...group,
    ],
    data: out.finish(),
  });
}

/** The encrypted Ring deposit fields published in a canonical SPP event. */
export interface ActualOutputEvent {
  readonly ownerUtxoHash: Bytes32;
  readonly asset: Address;
  readonly amount: bigint;
  readonly ringProgramId: Address;
  readonly ringDataHash: Bytes32;
  readonly encrypted: EncryptedRingDepositData;
}

/** Decodes the full canonical SPP output payload: Encrypted(scheme8, Borsh RingDepositOutput). */
export function decodeActualOutputEvent(payload: Uint8Array): ActualOutputEvent | undefined {
  if (payload[0] !== 1 || payload[5] !== EncryptedScheme.ringDeposit) return undefined;
  const { body } = readOutputFrame(payload);
  const framing = new Reader(body);
  const ownerUtxoHash = framing.bytes(32, "ownerUtxoHash") as Bytes32;
  const asset = encodeBase58(framing.bytes(32, "asset"));
  const amount = framing.u64("amount");
  const dataHash = framing.option("dataHash", (reader) => reader.bytes(32, "dataHash"));
  const ringProgramId = encodeBase58(framing.bytes(32, "ringProgramId"));
  const ringDataHash = framing.bytes(32, "ringDataHash") as Bytes32;
  const txViewingPublicKey = framing.bytes(33, "txViewingPublicKey") as Bytes33;
  const salt = framing.bytes(16, "salt") as Bytes16;
  const ciphertext = framing.bytes(framing.u32("ciphertext.length"), "ciphertext");
  framing.done();
  if (!equalBytes(ciphertext.subarray(0, 8), ACTUAL_OUTPUT_MAGIC)) return undefined;
  if (dataHash !== undefined || !isZero(ringDataHash) || amount <= 0n) throw invalid();
  return Object.freeze({
    ownerUtxoHash,
    asset,
    amount,
    ringProgramId,
    ringDataHash,
    encrypted: { txViewingPublicKey, salt, ciphertext },
  });
}

function descriptorOf(event: ActualOutputEvent) {
  const body = event.encrypted.ciphertext;
  if (
    !equalBytes(body.subarray(0, 8), ACTUAL_OUTPUT_MAGIC) ||
    body[9] !== RingInstructionTag.transfer
  )
    throw invalid();
  const slot = body[8];
  const data = RingTransactData.parse(body.subarray(10), "anonymous");
  const output = slot === undefined ? undefined : data.outputs[slot];
  const firstNullifier = data.nullifiers[0];
  const txViewingKey = X25519PublicKey.fromBytes(concatBytes(Uint8Array.of(4), data.txViewingU));
  if (
    slot === undefined ||
    output === undefined ||
    firstNullifier === undefined ||
    output.body.kind !== "recipient" ||
    !data.noSpend ||
    !data.publicOwners ||
    data.hasData ||
    data.ringData ||
    data.entry ||
    data.vouched ||
    !equalBytes(event.encrypted.salt, data.salt) ||
    !equalBytes(event.encrypted.txViewingPublicKey, txViewingKey.toBytes())
  )
    throw invalid();
  return {
    slot,
    data,
    output,
    firstNullifier,
    txViewingKey,
    body: concatBytes(output.record, output.body.payload),
  };
}

function openDescriptorRecord(secret: Bytes32, body: Uint8Array, asset: Address): OpenedOutput {
  const opened = openOutputSlot(secret, body, { data: false, ringData: false });
  if (
    opened.record.kind !== RecordKind.ring ||
    opened.record.amount !== 1n ||
    opened.record.mint !== asset
  )
    throw invalid();
  return opened;
}

/** Reopens stock ciphertext with the original wallet keys; only canonical amount replaces synthetic one. */
export async function openActualOutputForRecipient(
  input: Readonly<{
    event: ActualOutputEvent;
    treeId: TreeId;
    commitment: Bytes32;
    keys: RingKeys;
    assets: AssetRegistry;
    viewingPublicKey?: X25519PublicKey;
    context?: RequestContext;
  }>,
): Promise<OpenedRingNote | undefined> {
  const cert = descriptorOf(input.event);
  const owner = input.keys.address();
  const [secret] = await input.keys.ringDeliverySecrets(
    [
      {
        viewingPublicKey: input.viewingPublicKey ?? owner.viewingPublicKey,
        txViewingPublicKey: cert.txViewingKey,
        salt: cert.data.salt,
        outputIndex: cert.slot,
        firstNullifier: cert.firstNullifier,
      },
    ],
    input.context,
  );
  if (secret === undefined) return undefined;
  try {
    const opened = openDescriptorRecord(secret, cert.body, input.event.asset);
    const reader = {
      owner: {
        signingPublicKey: owner.signingPublicKey,
        nullifierPublicKey: owner.nullifierPublicKey,
      },
      ring: input.event.ringProgramId,
      treeId: input.treeId,
      assets: input.assets,
    };
    // Authenticate the synthetic descriptor before changing a single field.
    const synthetic = openRecipientNote({ ...reader, commitment: cert.output.hash }, opened);
    if (synthetic === undefined) return undefined;
    if (
      !equalBytes(
        ownerUtxoHash(owner.ownerHash(), synthetic.utxo.blinding),
        input.event.ownerUtxoHash,
      )
    )
      return undefined;
    return openRecipientNote(
      { ...reader, commitment: input.commitment },
      { ...opened, record: { ...opened.record, amount: input.event.amount } },
    );
  } catch {
    return undefined;
  } finally {
    secret.fill(0);
  }
}

/** The same unchanged audited descriptor opens to the configured auditor; runtime amount stays public. */
export function openActualOutputForAuditor(
  event: ActualOutputEvent,
  auditor: X25519Keypair,
): OutputRecord {
  const cert = descriptorOf(event);
  const layout = RingTransactData.streamLayout(cert.data);
  const disclosure = openRingDisclosure(auditor, {
    layout,
    salt: cert.data.salt,
    message: {
      ephemeralPublicKey: cert.txViewingKey,
      envelope: cert.data.envelope,
      recipientKeys: Array.from({ length: cert.data.outputs.length }, (_, i) =>
        cert.data.recipientKeys.slice(i * 32, (i + 1) * 32),
      ),
    },
  });
  const recipient = disclosure.recipientKeys[cert.slot];
  if (recipient === undefined) throw invalid();
  const txKey = ViewingKey.fromBytes(disclosure.envelope.txViewingSecret.slice() as Bytes32);
  let secret: Bytes32 | undefined;
  try {
    if (!txKey.publicKey().equals(cert.txViewingKey)) throw invalid();
    secret = txKeySlotSecret({
      txViewingKey: txKey,
      recipientViewingKey: X25519PublicKey.fromBytes(concatBytes(Uint8Array.of(4), recipient)),
      salt: cert.data.salt,
      outputIndex: cert.slot,
      firstNullifier: cert.firstNullifier,
    });
    const opened = openDescriptorRecord(secret, cert.body, event.asset);
    return Object.freeze({ ...opened.record, amount: event.amount });
  } finally {
    secret?.fill(0);
    txKey.destroy();
    disclosure.envelope.txViewingSecret.fill(0);
  }
}

function invalid(): RingError {
  return new RingError("RING_BUILD_DEPOSIT", {
    details: { reason: "invalid actual-output descriptor or settlement" },
  });
}
