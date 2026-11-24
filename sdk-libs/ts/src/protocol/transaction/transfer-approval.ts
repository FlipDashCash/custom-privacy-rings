/** A co-signer's approval of one transfer call a program makes, Rust `TransferApproval`. */
import type { Instruction } from "@solana/kit";

import {
  addressBytes,
  concatBytes,
  meta,
  sha256Bytes,
  type SignerAccount,
} from "@heliuslabs/spr/internal";
import type { Address, Bytes32 } from "@heliuslabs/spr/interface";

import { RingError } from "../error.js";
import { ringCoSignerMetas, ringPolicyAddress } from "../pda/index.js";
import { RingInstructionTag } from "./instruction-tag.js";
import { MAX_SPP_CPI_ACCOUNTS } from "./spp.js";
import { MAX_CO_SIGNERS } from "./transact.js";

/** The ring account, three registries and the co-signer pairs lead SPP's list. */
export const MAX_TRANSFER_ACCOUNTS = 1 + 3 + 2 * MAX_CO_SIGNERS + MAX_SPP_CPI_ACCOUNTS;

const DIGEST_LEN = 32;
const DATA_LEN = 2 + DIGEST_LEN;
const U32_MAX = 0xffff_ffff;

export interface TransferApproval {
  /** Index of the top-level instruction the transfer runs inside. */
  readonly position: number;
  readonly digest: Bytes32;
}

/** The ring `Transfer` call a digest commits to, `data` from the transfer tag. */
export interface ApprovedTransferCall {
  readonly ringProgramId: Address;
  readonly data: Uint8Array;
  /** In call order. */
  readonly accounts: readonly Address[];
}

export const TransferApproval = Object.freeze({
  DATA_LEN,

  /** `sha256(program id || u32 LE data length || data || addresses)`, the length keeps a data byte from reading as an address. */
  digest(call: ApprovedTransferCall): Bytes32 {
    if (call.data.length > U32_MAX) throw invalid("InvalidInstructionData");
    if (call.accounts.length > MAX_TRANSFER_ACCOUNTS) throw invalid("TooManyAccounts");
    const dataLength = new Uint8Array(4);
    new DataView(dataLength.buffer).setUint32(0, call.data.length, true);
    return sha256Bytes(
      concatBytes(
        addressBytes(call.ringProgramId),
        dataLength,
        call.data,
        ...call.accounts.map((account) => addressBytes(account)),
      ),
    );
  },

  /** The instruction data from the tag. */
  encode(approval: TransferApproval): Uint8Array {
    const { position, digest } = approval;
    if (
      !Number.isInteger(position) ||
      position < 0 ||
      position > 0xff ||
      digest.length !== DIGEST_LEN
    )
      throw invalid("InvalidInstructionData");
    const data = new Uint8Array(DATA_LEN);
    data[0] = RingInstructionTag.approveTransfer;
    data[1] = position;
    data.set(digest, 2);
    return data;
  },

  /** `undefined` unless `data` is exactly the tag and the body. */
  parse(data: Uint8Array): TransferApproval | undefined {
    const [tag, position] = data;
    if (data.length !== DATA_LEN || tag !== RingInstructionTag.approveTransfer) return undefined;
    return Object.freeze({ position: position ?? 0, digest: data.slice(2) as Bytes32 });
  },
});

/** The call `transfer` makes, as the ring program digests it. */
export function approvedTransferCall(transfer: Instruction): ApprovedTransferCall {
  return Object.freeze({
    ringProgramId: transfer.programAddress,
    data: new Uint8Array(transfer.data ?? []),
    accounts: Object.freeze((transfer.accounts ?? []).map((account) => account.address)),
  });
}

/** `coSigner` signs at the top level, the instruction writes nothing and covers one co-signer record. */
export async function approveTransferInstruction(
  input: Readonly<{
    ringProgramId: Address;
    coSigner: SignerAccount;
    approval: TransferApproval;
  }>,
): Promise<Instruction> {
  const { ringProgramId } = input;
  const [ring, pair] = await Promise.all([
    ringPolicyAddress(ringProgramId),
    ringCoSignerMetas(ringProgramId, input.coSigner),
  ]);
  return Object.freeze({
    programAddress: ringProgramId,
    accounts: Object.freeze([meta(ring, false, false), ...pair]),
    data: TransferApproval.encode(input.approval),
  });
}

/** Rust `transfer_approvals`, one per co-signer, the digest reads `transfer` as built. */
export function transferApprovals(
  input: Readonly<{
    transfer: Instruction;
    coSigners: readonly SignerAccount[];
    position: number;
  }>,
): Promise<readonly Instruction[]> {
  const { transfer, position } = input;
  const approval = { position, digest: TransferApproval.digest(approvedTransferCall(transfer)) };
  return Promise.all(
    input.coSigners.map((coSigner) =>
      approveTransferInstruction({ ringProgramId: transfer.programAddress, coSigner, approval }),
    ),
  );
}

function invalid(reason: string): RingError {
  return new RingError("RING_TRANSACT_DATA_INVALID", { details: { reason } });
}
