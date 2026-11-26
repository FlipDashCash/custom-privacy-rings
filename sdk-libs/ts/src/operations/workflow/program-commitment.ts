/** What a program proves over before its ring transfer stages, Rust `program_commitment`. */
import {
  addressBytes,
  bigIntBytes,
  encodeBase58,
  equalBytes,
  hashBytes,
  poseidon,
  ZERO_32,
} from "@heliuslabs/spr/internal";
import type { Bytes16, Bytes32, Shape } from "@heliuslabs/spr/interface";
import {
  SOL_MINT,
  type ProofOutputUtxo,
  type SettlementTransfer,
  type SppProofInputs,
} from "@heliuslabs/spr/transaction";

import { RingError } from "../../protocol/error.js";
import { settlementAccounts } from "../../protocol/transaction/spp.js";

/** Rust `ProgramCommitment`. */
export interface ProgramCommitment {
  readonly salt: Bytes16;
  readonly privateTxHash: Bytes32;
  readonly transactionHash: Bytes32;
}

/** The sealed transfer's values, staging under its salt reproduces each one, Rust `ProgramPlan`. */
export class ProgramPlan {
  readonly salt: Bytes16;
  readonly blindingSeed: Bytes32;
  readonly shape: Shape;
  readonly firstNullifier: Bytes32;
  readonly outputTreeId: number;
  /** Final slot order with every blinding, padding included. */
  readonly outputs: readonly ProofOutputUtxo[];
  readonly privateTxHash: Bytes32;
  /** `privateTxHash` chained with the public transfers in leg order. */
  readonly transactionHash: Bytes32;

  /** @internal */
  constructor(sealed: SppProofInputs) {
    const hashes = committedHashes(sealed);
    this.salt = sealed.externalData.salt;
    this.blindingSeed = sealed.blindingSeed;
    this.shape = sealed.checkShape();
    this.firstNullifier = sealed.firstNullifier();
    this.outputTreeId = sealed.outputTreeId;
    this.outputs = sealed.outputs;
    this.privateTxHash = hashes.privateTxHash;
    this.transactionHash = hashes.transactionHash;
    Object.freeze(this);
  }

  commitment(): ProgramCommitment {
    return Object.freeze({
      salt: this.salt,
      privateTxHash: this.privateTxHash,
      transactionHash: this.transactionHash,
    });
  }
}

/** Refuses a staged transfer whose hashes differ from the ones the program's proof holds, the details name both in base58. */
export function checkProgramCommitment(
  commitment: ProgramCommitment,
  staged: SppProofInputs,
): void {
  const hashes = committedHashes(staged);
  if (!equalBytes(hashes.privateTxHash, commitment.privateTxHash))
    throw new RingError("RING_PROGRAM_COMMITMENT_MISMATCH", {
      details: mismatch(commitment.privateTxHash, hashes.privateTxHash),
    });
  if (!equalBytes(hashes.transactionHash, commitment.transactionHash))
    throw new RingError("RING_PROGRAM_TRANSACTION_HASH_MISMATCH", {
      details: mismatch(commitment.transactionHash, hashes.transactionHash),
    });
}

function mismatch(committed: Bytes32, staged: Bytes32): Readonly<Record<string, string>> {
  return { committed: encodeBase58(committed), staged: encodeBase58(staged) };
}

function committedHashes(
  sealed: SppProofInputs,
): Readonly<{ privateTxHash: Bytes32; transactionHash: Bytes32 }> {
  const privateTxHash = sealed.privateTxHash();
  return {
    privateTxHash,
    transactionHash: transactionHash(privateTxHash, sealed.externalData.interfaceTransfers),
  };
}

/** Rust `transaction_hash`, the private hash alone when no leg is public. */
export function transactionHash(
  privateTxHash: Bytes32,
  transfers: readonly SettlementTransfer[],
): Bytes32 {
  if (transfers.length === 0) return privateTxHash;
  const transfersHash = transfers.reduce<Uint8Array>(
    (chain, transfer) => poseidon([chain, publicTransferHash(transfer)]),
    ZERO_32,
  );
  return poseidon([privateTxHash, transfersHash]);
}

/** Rust `PublicTransfer::hash`, the amount and the direction as right-aligned integers. */
function publicTransferHash(transfer: SettlementTransfer): Bytes32 {
  return poseidon([
    hashBytes(addressBytes(transfer.kind === "sol" ? SOL_MINT : transfer.mint)),
    bigIntBytes(transfer.amount),
    bigIntBytes(transfer.isDeposit ? 1n : 0n),
    hashBytes(addressBytes(settlementAccounts(transfer).user)),
  ]);
}
