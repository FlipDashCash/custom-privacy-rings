#!/usr/bin/env python3
"""Optional, source-derived layout diagnostic; not a release or decoder gate."""
import argparse
import json
from pathlib import Path
import struct

from check_jupiter_descriptor_release import read_regular

G1, G2 = 64, 128
FIXED_POINTS = (G1, G1, G2, G2, G1, G2)
K_POINTS = 3
COMMITMENT_ROWS = 1
PUBLIC_INDICES = 0
COMMITMENT_KEYS = 1
PREDICTED_BYTES = sum(FIXED_POINTS) + 4 + K_POINTS * G1 + 3 * 4 + 2 * G2


def inspect_layout(data):
    if len(data) != PREDICTED_BYTES:
        raise ValueError(f"expected source-predicted {PREDICTED_BYTES} bytes, got {len(data)}")

    offset = 0

    def raw_point(width):
        nonlocal offset
        if offset + width > len(data):
            raise ValueError("truncated point span")
        # The safe gnark decoder accepts both widths. Check every mode flag
        # before depending on any fixed cursor, without interpreting coordinates.
        if data[offset] & 0b11000000:
            raise ValueError(f"non-raw point marker at byte {offset}")
        offset += width

    def count(label, expected):
        nonlocal offset
        if offset + 4 > len(data):
            raise ValueError("truncated count")
        value = struct.unpack_from(">I", data, offset)[0]
        if value != expected:
            raise ValueError(f"unexpected {label} count at byte {offset}")
        offset += 4

    for width in FIXED_POINTS:
        raw_point(width)
    count("K point", K_POINTS)
    for _ in range(K_POINTS):
        raw_point(G1)
    count("commitment row", COMMITMENT_ROWS)
    count("public index", PUBLIC_INDICES)
    count("commitment key", COMMITMENT_KEYS)
    raw_point(G2)
    raw_point(G2)
    if offset != len(data):
        raise ValueError("unconsumed input bytes")

    return {
        "status": "MATCHES_SOURCE_PREDICTED_RAW_LAYOUT",
        "evidence": "SOURCE_INFERENCE_NOT_MEASURED_WITH_PINNED_CODEC",
        "bytes_consumed_by_layout_walk": offset,
        "k_points": K_POINTS,
        "commitment_rows": COMMITMENT_ROWS,
        "public_indices": PUBLIC_INDICES,
        "commitment_keys": COMMITMENT_KEYS,
        "points_or_canonical_decoder_checked": False,
        "release_or_circuit_provenance_verified": False,
        "admission_decision_made": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("raw_public_vk", type=Path)
    args = parser.parse_args()
    try:
        receipt = inspect_layout(read_regular(args.raw_public_vk, PREDICTED_BYTES))
    except (OSError, ValueError) as error:
        detail = str(error) if isinstance(error, ValueError) else "invalid or unavailable input"
        parser.exit(1, f"Raw layout diagnostic did not match: {detail}\n")
    print(json.dumps(receipt, sort_keys=True, separators=(",", ":")))


if __name__ == "__main__":
    main()
