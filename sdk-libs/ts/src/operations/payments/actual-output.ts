/** Audited runtime-amount output settlement using unchanged stock Ring certificates and SPP notes. */
import type { Instruction } from "@solana/kit";
import { initializePoseidon } from "@heliuslabs/spr";
import type { SprClient } from "@heliuslabs/spr/client";
import {
  Writer,
  meta,
  SYSTEM_PROGRAM,
  equalBytes,
  concatBytes,
  ringAuthAddress,
  splInterfaceWithBump,
} from "@heliuslabs/spr/internal";
import {
  SHIELDED_POOL_PROGRAM_ID,
  SOL_INTERFACE,
  SPL_TOKEN_2022_PROGRAM_ID,
  SPL_TOKEN_PROGRAM_ID,
  type Address,
  type Bytes32,
  type RequestContext,
  type TransactProof,
} from "@heliuslabs/spr/interface";
import { SOL_MINT, ownerUtxoHash } from "@heliuslabs/spr/transaction";
import { fetchRingPolicy, fetchStorageRegistry } from "../../protocol/membership/config.js";
import { MEMBER_REGISTRY_EMPTY_ROOT } from "../../protocol/membership/member-opening.js";
import { ringPolicyAddress } from "../../protocol/pda/index.js";
import {
  compactMeasuredCertificate,
  RingTransactData,
} from "../../protocol/transaction/transact.js";
import {
  ACTUAL_OUTPUT_FORMAT,
  RingInstructionTag,
} from "../../protocol/transaction/instruction-tag.js";
import { ringIdField } from "../../crypto/derivation/ring-stream.js";
import { RingError } from "../../protocol/error.js";
import {
  PreparedRingDeposit,
  type RingDepositParams,
  type RingDepositRecipient,
} from "./deposit.js";
import { measuredDepositCertificate } from "../workflow/pipeline.js";
import { ringRegistrySlots, transferRootRead } from "../workflow/ring-reads.js";
import {
  actualOutputReservationAddresses,
  proveActualOutputReservation,
} from "./actual-output-reservation.js";

/** Public ownership certificate and its one-use address proof. */
export interface ActualOutputDescriptor {
  readonly ringProgramId: Address;
  readonly asset: Address;
  readonly outputTree: Address;
  readonly registries: readonly Address[];
  readonly ownerUtxoHash: Bytes32;
  readonly privateTxBlinding: Bytes32;
  readonly zeroChangeOwnerUtxoHash: Bytes32;
  readonly certificate: Uint8Array;
  readonly nativeReservation: Readonly<{ nullifierRootIndex: number; proof: TransactProof }>;
}

/** Prepares a synthetic amount-one certificate and a stock SPP address reservation. */
export async function prepareActualOutputDescriptor(
  params: Omit<RingDepositParams, "amount" | "cosigner" | "recipient"> & {
    readonly recipient: Exclude<RingDepositRecipient, { kind: "confidential" }>;
    readonly client: RingDepositParams["client"] & Pick<SprClient, "proveTransact">;
    /** Tree-0 proving client when the deposit client is configured for another output tree. */
    readonly reservationClient?: Pick<SprClient, "proveTransact">;
  },
  context?: RequestContext,
): Promise<ActualOutputDescriptor> {
  const policy = await fetchRingPolicy(params.client, params.ringProgramId, context);
  if (policy.mode !== "anonymous" || policy.hasPolicy || policy.coSignerClasses !== undefined)
    throw invalid();
  const read = transferRootRead(policy, false);
  const registries = ringRegistrySlots(policy, read);
  const ownRevision = read === "own" ? 0n : undefined;
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
  let privateTxBlinding: Bytes32 | undefined;
  try {
    const proven = await prepared.proveTransfer(params.client, context, {
      transaction: {},
      [measuredDepositCertificate]: true,
      approve({ proofInputs }) {
        // The pipeline checks that this certificate spends no inputs before approving it.
        privateTxBlinding = proofInputs.privateTxBlinding();
        return Promise.resolve();
      },
    });
    const { data, accounts } = proven;
    if (
      privateTxBlinding === undefined ||
      data.hasData ||
      data.ringData ||
      data.entry ||
      data.vouched ||
      data.ownRevision !== ownRevision ||
      data.approval !== 0 ||
      accounts.registries.length !== registries.length ||
      accounts.registries.some((address, index) => address !== registries[index])
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
    const outputCommitment = ownerUtxoHash(output.ownerHash(), output.blinding);
    const nativeReservation = await proveActualOutputReservation(
      params.reservationClient ?? params.client,
      params.ringProgramId,
      outputCommitment,
      params.feePayer,
      context,
    );
    return Object.freeze({
      ringProgramId: params.ringProgramId,
      asset,
      outputTree: accounts.outputTree,
      registries: accounts.registries,
      ownerUtxoHash: outputCommitment,
      privateTxBlinding,
      zeroChangeOwnerUtxoHash: ownerUtxoHash(change.ownerHash(), change.blinding),
      certificate: concatBytes(
        Uint8Array.of(RingInstructionTag.transfer),
        RingTransactData.encode(data),
      ),
      nativeReservation,
    });
  } catch (cause) {
    privateTxBlinding?.fill(0);
    throw cause;
  }
}

export interface ActualOutputSettlementParams {
  readonly descriptor: ActualOutputDescriptor;
  /** Signing owner of the SOL balance or SPL source account. */
  readonly authority: Address;
  /** Pays the SPP address tree fee. */
  readonly reservationPayer: Address;
  readonly minimumOutput: bigint;
  readonly keepLamports?: bigint;
  readonly sourceTokenAccount?: Address;
  readonly tokenProgram?: Address;
}

/** Builds the unsigned settlement instruction to run after the instructions that produce its output. */
export async function buildSettleActualOutputInstruction(
  params: ActualOutputSettlementParams,
): Promise<Instruction> {
  const { descriptor, minimumOutput, keepLamports = 0n } = params;
  if (
    minimumOutput <= 0n ||
    keepLamports < 0n ||
    (descriptor.registries.length !== 1 && descriptor.registries.length !== 2)
  )
    throw invalid();
  const [ring, ringAuth, reservation] = await Promise.all([
    ringPolicyAddress(descriptor.ringProgramId),
    ringAuthAddress(descriptor.ringProgramId),
    actualOutputReservationAddresses(descriptor.ringProgramId, descriptor.ownerUtxoHash),
  ]);
  const out = new Writer()
    .u8(RingInstructionTag.settleActualOutput, "tag")
    .u8(ACTUAL_OUTPUT_FORMAT, "format")
    .u64(minimumOutput, "minimumOutput")
    .u64(keepLamports, "keepLamports")
    .bytes(descriptor.ownerUtxoHash, 32)
    .bytes(descriptor.privateTxBlinding, 32)
    .bytes(descriptor.zeroChangeOwnerUtxoHash, 32);
  const group = [];
  if (descriptor.asset === SOL_MINT) {
    if (params.sourceTokenAccount !== undefined || params.tokenProgram !== undefined)
      throw invalid();
    out.u8(0, "asset");
    group.push(meta(SYSTEM_PROGRAM, false, false), meta(SOL_INTERFACE, false, true));
  } else {
    if (
      params.sourceTokenAccount === undefined ||
      (params.tokenProgram !== SPL_TOKEN_PROGRAM_ID &&
        params.tokenProgram !== SPL_TOKEN_2022_PROGRAM_ID) ||
      keepLamports !== 0n
    )
      throw invalid();
    const [splInterface, bump] = await splInterfaceWithBump(descriptor.asset);
    out.u8(1, "asset").u8(bump, "splInterfaceBump");
    group.push(
      meta(params.tokenProgram, false, false),
      meta(descriptor.asset, false, false),
      meta(params.sourceTokenAccount, false, true),
      meta(splInterface, false, true),
    );
  }
  const certificate = compactMeasuredCertificate(descriptor.certificate);
  out.u16(certificate.length, "certificate.length").bytes(certificate);
  const { nullifierRootIndex, proof } = descriptor.nativeReservation;
  out
    .u16(nullifierRootIndex, "reservation.nullifierRootIndex")
    .bytes(proof.a, 32)
    .bytes(proof.b, 128)
    .bytes(proof.c, 32);
  return Object.freeze({
    programAddress: descriptor.ringProgramId,
    accounts: [
      meta(ring, false, false),
      ...descriptor.registries.map((address) => meta(address, false, false)),
      meta(params.reservationPayer, true, true),
      meta(SYSTEM_PROGRAM, false, false),
      meta(reservation.addressTree, false, true),
      meta(reservation.nullifierPda, false, true),
      meta(reservation.owner, false, false),
      meta(descriptor.outputTree, false, true),
      meta(params.authority, true, true),
      meta(ringAuth, false, false),
      meta(SHIELDED_POOL_PROGRAM_ID, false, false),
      ...group,
    ],
    data: out.finish(),
  });
}

function invalid(): RingError {
  return new RingError("RING_BUILD_DEPOSIT", {
    details: { reason: "invalid actual-output descriptor or settlement" },
  });
}
