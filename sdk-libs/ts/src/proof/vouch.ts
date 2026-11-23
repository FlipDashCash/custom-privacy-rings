/** The vouch every ring transfer proof verifies, Go `policy/voucher.go`. */
import { copyBytes } from "@heliuslabs/spr/internal";
import type { Bytes32, RequestContext } from "@heliuslabs/spr/interface";
import { SigningKey } from "@heliuslabs/spr/keypair";

import { RingDomain, domainHash } from "../crypto/derivation/ring-domain.js";
import type { AnonymousSpendSigner } from "../operations/payments/anonymous.js";
import type { VoucherProofInput } from "../rpc/prover/ring.js";

/** The transfer a voucher signs for, Rust `VouchMessage`. */
export interface VouchMessage {
  readonly privateTxHash: Bytes32;
  readonly withdrawalHash: Bytes32;
}

/** `Poseidon(CRVC, private_tx_hash, withdrawal_hash)`, the voucher signs its 32 big endian bytes. */
export function vouchDigest(message: VouchMessage): Bytes32 {
  return domainHash(RingDomain.vouch, [
    copyBytes(message.privateTxHash),
    copyBytes(message.withdrawalHash),
  ]);
}

/** A fresh throwaway key signs a transfer nobody vouches for, the proof verifies one signature on every frame. */
export async function signVouch(
  voucher: AnonymousSpendSigner | undefined,
  message: VouchMessage,
  context?: RequestContext,
): Promise<VoucherProofInput> {
  const digest = vouchDigest(message);
  if (voucher !== undefined)
    return Object.freeze({
      publicKey: voucher.publicKey().ed25519(),
      signature: await voucher.sign(digest, context),
    });
  const throwaway = SigningKey.generate();
  try {
    return Object.freeze({
      publicKey: throwaway.publicKey().ed25519(),
      signature: throwaway.sign(digest),
    });
  } finally {
    throwaway.destroy();
  }
}
