import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

import { test } from "vitest";

import {
  decodeJupiterMeasuredDepositDraftV1,
  encodeJupiterMeasuredDepositDraftV1,
  type JupiterMeasuredDepositDraftV1,
} from "../../../drafts/jupiter-measured-deposit-wire-v1.js";

const fixtureUrl = new URL(
  "../../../../../program-libs/interface/tests/fixtures/jupiter_measured_deposit_draft_v1.hex",
  import.meta.url,
);
const fixtureHex = readFileSync(fixtureUrl, "utf8").trim();
const fixtureBytes = Uint8Array.from(Buffer.from(fixtureHex, "hex"));

function inertFields(): JupiterMeasuredDepositDraftV1 {
  const txViewingPubkey = new Uint8Array(33);
  txViewingPubkey.set([4, 9]);
  const recipientTag = new Uint8Array(32).fill(0x55, 24);
  return {
    expirySlot: 0x0123_4567_89ab_cdefn,
    trackingRevision: 0x1020_3040_5060_7080n,
    trackingRoot: new Uint8Array(32).fill(0x11),
    ownerUtxoHash: new Uint8Array(32).fill(0x22),
    txViewingPubkey,
    salt: new Uint8Array(16).fill(0x44),
    recipientTag,
    recipientZeroCiphertext: new Uint8Array(105).fill(0x66),
    auditorCiphertext: new Uint8Array(64).fill(0x77),
    descriptorProofBytes: new Uint8Array(192), // Inert; not a valid proof.
  };
}

test("encodes the same 524 bytes as the Rust interface's inert V1 fixture", () => {
  assert.match(fixtureHex, /^[0-9a-f]{1048}$/u);
  assert.equal(
    createHash("sha256").update(fixtureBytes).digest("hex"),
    "57c452371730cbca4b9622f98b378440e264f35cd4b61b2567b3e3cd546e6d4f",
  );
  assert.deepEqual(encodeJupiterMeasuredDepositDraftV1(inertFields()), fixtureBytes);
});

test("decodes the shared vector from an offset view and re-encodes it exactly", () => {
  const backing = new Uint8Array(fixtureBytes.length + 9);
  backing.set(fixtureBytes, 5);
  const decoded = decodeJupiterMeasuredDepositDraftV1(backing.subarray(5, 529));
  backing.fill(0xff);
  assert.deepEqual(decoded, inertFields());
  assert.deepEqual(encodeJupiterMeasuredDepositDraftV1(decoded), fixtureBytes);
});

test("keeps both u64 endpoints exact and owns the encoded field bytes", () => {
  const fields = { ...inertFields(), expirySlot: 0n, trackingRevision: (1n << 64n) - 1n };
  const bytes = encodeJupiterMeasuredDepositDraftV1(fields);
  fields.salt.fill(0);
  assert.deepEqual(
    Array.from(bytes.subarray(2, 18)),
    [0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255],
  );
  assert.deepEqual(decodeJupiterMeasuredDepositDraftV1(bytes), {
    ...inertFields(),
    expirySlot: 0n,
    trackingRevision: (1n << 64n) - 1n,
  });
});
