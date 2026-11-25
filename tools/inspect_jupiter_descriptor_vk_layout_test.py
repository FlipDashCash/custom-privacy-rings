"""Byte-layout controls only. No curve points, keys, proof or native codec."""
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest

import inspect_jupiter_descriptor_vk_layout as tool


def layout_only_fixture():
    # Independently specified spans/counts. 0x31 has a raw prefix but these
    # filler coordinates are not canonical field elements, let alone a VK.
    return (
        bytes([0x31]) * 576 + struct.pack(">I", 3) + bytes([0x31]) * 192
        + struct.pack(">III", 1, 0, 1) + bytes([0x31]) * 256
    )


class LayoutTests(unittest.TestCase):
    def test_walks_source_predicted_exact_shape_without_admitting_anything(self):
        result = tool.inspect_layout(layout_only_fixture())
        self.assertEqual(result["status"], "MATCHES_SOURCE_PREDICTED_RAW_LAYOUT")
        self.assertEqual(result["bytes_consumed_by_layout_walk"], 1040)
        self.assertEqual(tuple(result[k] for k in ("k_points", "commitment_rows", "public_indices", "commitment_keys")),
                         (3, 1, 0, 1))
        self.assertIs(result["points_or_canonical_decoder_checked"], False)
        self.assertIs(result["release_or_circuit_provenance_verified"], False)
        self.assertIs(result["admission_decision_made"], False)

    def test_missing_or_trailing_bytes_cannot_match(self):
        template = layout_only_fixture()
        for sample in (b"", template[:-1], template[:-256], template + b"\0"):
            with self.subTest(length=len(sample)), self.assertRaisesRegex(ValueError, "1040 bytes"):
                tool.inspect_layout(sample)

    def test_valid_length_alone_is_not_enough_for_any_count(self):
        for offset, value, name in (
            (576, 2, "K point"),
            (772, 0, "commitment row"),
            (776, 1, "public index"),
            (780, 0, "commitment key"),
        ):
            with self.subTest(offset=offset):
                candidate = bytearray(layout_only_fixture())
                struct.pack_into(">I", candidate, offset, value)
                with self.assertRaisesRegex(ValueError, f"{name} count at byte {offset}"):
                    tool.inspect_layout(candidate)

    def test_every_point_starts_in_raw_mode_even_with_correct_fixed_counts(self):
        for offset in (0, 64, 128, 256, 384, 448, 580, 644, 708, 784, 912):
            for metadata in (0x40, 0x80, 0xC0):
                with self.subTest(offset=offset, metadata=metadata):
                    candidate = bytearray(layout_only_fixture())
                    candidate[offset] |= metadata
                    with self.assertRaisesRegex(ValueError, f"non-raw point marker at byte {offset}"):
                        tool.inspect_layout(candidate)

    def test_cli_only_reports_an_optional_shape_receipt(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "layout-only.bytes"
            args = [sys.executable, tool.__file__, str(path)]
            path.write_bytes(layout_only_fixture())
            good = subprocess.run(args, capture_output=True)
            self.assertEqual(good.returncode, 0, good.stderr)
            self.assertIs(json.loads(good.stdout)["admission_decision_made"], False)
            path.write_bytes(layout_only_fixture() + b"\0")
            bad = subprocess.run(args, capture_output=True)
            self.assertEqual(bad.returncode, 1, bad.stderr)
            self.assertEqual(bad.stdout, b"")


if __name__ == "__main__":
    unittest.main()
