/**
 * Inert transport mirror of Custom's unregistered Jupiter wire draft V1.
 * Kept outside src and the package build. It neither proves nor verifies the
 * descriptor and must not be used to construct a transaction or infer credit.
 */
const TAG = 68;
const VERSION = 1;
const FRAME_BYTES = 524;
const U64_MAX = (1n << 64n) - 1n;

const BYTE_FIELDS = [
  ["trackingRoot", 18, 32],
  ["ownerUtxoHash", 50, 32],
  ["txViewingPubkey", 82, 33],
  ["salt", 115, 16],
  ["recipientTag", 131, 32],
  ["recipientZeroCiphertext", 163, 105],
  ["auditorCiphertext", 268, 64],
  ["descriptorProofBytes", 332, 192],
] as const;
type ByteField = (typeof BYTE_FIELDS)[number][0];

export type JupiterMeasuredDepositDraftV1 = Readonly<
  { expirySlot: bigint; trackingRevision: bigint } & Record<ByteField, Uint8Array>
>;

const FIELD_NAMES = new Set<PropertyKey>([
  "expirySlot",
  "trackingRevision",
  ...BYTE_FIELDS.map(([name]) => name),
]);

function requireU64(value: bigint): bigint {
  if (typeof value !== "bigint" || value < 0n || value > U64_MAX) {
    throw new RangeError("Invalid Jupiter measured-deposit draft u64");
  }
  return value;
}

/** Serialize already supplied opaque fields. No descriptor relation is selected. */
export function encodeJupiterMeasuredDepositDraftV1(
  frame: JupiterMeasuredDepositDraftV1,
): Uint8Array {
  if (!frame || typeof frame !== "object") {
    throw new TypeError("Invalid Jupiter measured-deposit draft fields");
  }
  const fields = Reflect.ownKeys(frame);
  if (fields.length !== FIELD_NAMES.size || fields.some((name) => !FIELD_NAMES.has(name))) {
    throw new TypeError("Invalid Jupiter measured-deposit draft fields");
  }

  const bytes = new Uint8Array(FRAME_BYTES);
  bytes[0] = TAG;
  bytes[1] = VERSION;
  const view = new DataView(bytes.buffer);
  view.setBigUint64(2, requireU64(frame.expirySlot), true);
  view.setBigUint64(10, requireU64(frame.trackingRevision), true);
  for (const [name, offset, length] of BYTE_FIELDS) {
    const field = frame[name];
    if (!(field instanceof Uint8Array) || field.byteLength !== length) {
      throw new RangeError("Invalid Jupiter measured-deposit draft field width");
    }
    bytes.set(field, offset);
  }
  return bytes;
}

/** Parse the fixed envelope only. Returned byte arrays are detached copies. */
export function decodeJupiterMeasuredDepositDraftV1(
  input: Uint8Array,
): JupiterMeasuredDepositDraftV1 {
  if (!(input instanceof Uint8Array) || input.byteLength !== FRAME_BYTES) {
    throw new RangeError("Invalid Jupiter measured-deposit draft size");
  }
  const bytes = Uint8Array.from(input);
  if (bytes[0] !== TAG || bytes[1] !== VERSION) {
    throw new TypeError("Invalid Jupiter measured-deposit draft header");
  }
  const view = new DataView(bytes.buffer);
  const fields = Object.fromEntries(
    BYTE_FIELDS.map(([name, offset, length]) => [name, bytes.slice(offset, offset + length)]),
  ) as Record<ByteField, Uint8Array>;
  return Object.freeze({
    expirySlot: view.getBigUint64(2, true),
    trackingRevision: view.getBigUint64(10, true),
    ...fields,
  });
}
