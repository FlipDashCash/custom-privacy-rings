/** Rust `tag`, the first byte of the ring program's instruction data. */
export const RingInstructionTag = Object.freeze({
  transfer: 55,
  settleActualOutput: 67,
  merge: 57,
  updateAuthority: 6,
  setPaused: 11,
  setCoSigner: 27,
  clearCoSigner: 62,
  setDelegate: 64,
  approveTransfer: 65,
  authorityTransfer: 56,
  initialize: 53,
  updatePolicy: 54,
  enrollBatch: 66,
} as const);
export type RingInstructionTag = (typeof RingInstructionTag)[keyof typeof RingInstructionTag];

/** Format discriminator written after the measured settlement instruction tag. */
export const ACTUAL_OUTPUT_FORMAT = 4;
