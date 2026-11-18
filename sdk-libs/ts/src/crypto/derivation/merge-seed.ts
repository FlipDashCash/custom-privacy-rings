/** Rust `zolana_ring_transaction::MergeSeed`, the owner's nullifier secret as a merge record carries it. */
import type { Bytes32 } from "@heliuslabs/spr";
import {
  checkedBytes,
  DOMAIN_MERGE_DUMMY_NULLIFIER,
  DOMAIN_MERGE_OUTPUT_BLINDING_V1,
  fieldU32,
  poseidon,
} from "@heliuslabs/spr/internal";

export function seededMergeOutputBlinding(seed: Bytes32, firstNullifier: Bytes32): Bytes32 {
  return seededHash(DOMAIN_MERGE_OUTPUT_BLINDING_V1, seed, firstNullifier);
}

export function seededMergeDummyNullifier(
  seed: Bytes32,
  firstNullifier: Bytes32,
  slotIndex: number,
): Bytes32 {
  if (!Number.isInteger(slotIndex) || slotIndex < 0 || slotIndex > 0xff)
    throw new RangeError("merge dummy slot index must fit in u8");
  return poseidon([
    fieldU32(DOMAIN_MERGE_DUMMY_NULLIFIER),
    checkedBytes<Bytes32>(seed, 32, "merge seed"),
    checkedBytes<Bytes32>(firstNullifier, 32, "first nullifier"),
    fieldU32(slotIndex),
  ]) as Bytes32;
}

function seededHash(domain: number, seed: Bytes32, firstNullifier: Bytes32): Bytes32 {
  return poseidon([
    fieldU32(domain),
    checkedBytes<Bytes32>(seed, 32, "merge seed"),
    checkedBytes<Bytes32>(firstNullifier, 32, "first nullifier"),
  ]) as Bytes32;
}
