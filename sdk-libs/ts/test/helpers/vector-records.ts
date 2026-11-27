import type { Bytes32 } from "@heliuslabs/spr/interface";

import {
  RecordKind,
  decodeOutputRecord,
  namedNullifierPk,
  type OutputRecord,
} from "../../src/protocol/codec/audit-stream.js";
import type { AuditLayout, RecordForm } from "../../src/protocol/codec/ring-disclosure-layout.js";

/** A slot's published record and the key it names, absent on a dummy and a nonzero-key ring output. */
export interface VectorRecord {
  readonly record: OutputRecord;
  readonly nullifierPk: Bytes32 | undefined;
}

/** The published plaintext at the length its kind takes under `form`. */
export function vectorRecord(plaintext: Uint8Array, form: RecordForm): VectorRecord {
  const record = decodeOutputRecord(plaintext, form);
  return {
    record,
    nullifierPk: record.kind === RecordKind.dummy ? undefined : namedNullifierPk(record.key),
  };
}

/** The layout a vector's `layout` value names, a bit past the four refused. */
export function vectorLayout(value: number): AuditLayout {
  if (!Number.isInteger(value) || (value & ~15) !== 0) throw new Error(`layout ${value}`);
  return {
    vouched: (value & 1) !== 0,
    entry: (value & 2) !== 0,
    records: { data: (value & 4) !== 0, ringData: (value & 8) !== 0 },
  };
}
