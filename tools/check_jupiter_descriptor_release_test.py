"""Artifact-control tests. Test bytes are not a verifying key or a release."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import check_jupiter_descriptor_release as tool

DUMMY = b"not a raw verifying key: checksum test only"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def release():
    return {
        "schema": tool.RELEASE_SCHEMA,
        "relation": copy.deepcopy(tool.RELATION),
        "release": {"repository": "fixture/test-only", "commit": "4" * 40},
        "public_vk": {"bytes": len(DUMMY), "encoding": tool.VK_ENCODING, "sha256": digest(DUMMY)},
    }


def test_events():
    return [
        {"Package": tool.PACKAGE, "Test": tool.TEST, "Action": "run"},
        {"Package": tool.PACKAGE, "Test": tool.TEST, "Action": "output",
         "Output": f"    jupiter_relation_shape_test.go:42: {tool.SHAPE}\n"},
        {"Package": tool.PACKAGE, "Test": tool.TEST, "Action": "output",
         "Output": "    jupiter_relation_shape_test.go:59: NO_SETUP NO_PROOF: "
                   f"r1cs_bytes=456 r1cs_sha256={tool.RELATION['r1cs_sha256']}\n"},
        {"Package": tool.PACKAGE, "Test": tool.TEST, "Action": "pass"},
        {"Package": tool.PACKAGE, "Action": "pass"},
    ]


def encoded(events):
    return "\n".join(json.dumps(event) for event in events)


class ManifestTests(unittest.TestCase):
    def test_fixed_reference_is_not_a_release(self):
        raw = tool.canonical(tool.RELATION)
        with self.assertRaisesRegex(ValueError, "unexpected fields"):
            tool.checked_manifest(raw, digest(raw))

    def test_canonical_manifest(self):
        doc = release()
        raw = tool.canonical(doc)
        self.assertEqual(tool.checked_manifest(raw, digest(raw)), doc["public_vk"])

    def test_correct_version_shape_cannot_replace_circuit_digest(self):
        for field, wrong in (
            ("r1cs_sha256", "d" * 64),
            ("r1cs_encoding", "gnark.VerifyingKey.WriteRawTo"),
            ("public_inputs", True),
            ("privacy_prover_server_tree", "c" * 40),
            ("tag", 67),
        ):
            with self.subTest(field=field):
                doc = release()
                doc["relation"][field] = wrong
                raw = tool.canonical(doc)
                with self.assertRaisesRegex(ValueError, "exact descriptor relation"):
                    tool.checked_manifest(raw, digest(raw))

    def test_no_self_declared_approval_or_ambiguous_json(self):
        raw = tool.canonical(dict(release(), approved=True))
        with self.assertRaises(ValueError):
            tool.checked_manifest(raw, digest(raw))
        raw = tool.canonical(release())
        duplicate = b'{"schema":"ignored",' + raw[1:]
        for candidate in (raw.rstrip(), duplicate, b"{}", b"[]"):
            with self.subTest(candidate=candidate[:30]), self.assertRaises(ValueError):
                tool.checked_manifest(candidate, digest(candidate))
        with self.assertRaisesRegex(ValueError, "supplied expected"):
            tool.checked_manifest(raw, "e" * 64)

    def test_manifest_refuses_non_raw_formats_and_moving_refs(self):
        for field, value in (("encoding", "whole-setup-bundle"), ("bytes", True), ("sha256", "0" * 64)):
            with self.subTest(field=field), self.assertRaises(ValueError):
                doc = release()
                doc["public_vk"][field] = value
                raw = tool.canonical(doc)
                tool.checked_manifest(raw, digest(raw))
        doc = release()
        doc["release"]["commit"] = "main"
        raw = tool.canonical(doc)
        with self.assertRaisesRegex(ValueError, "immutable source commit"):
            tool.checked_manifest(raw, digest(raw))


class ObservationTests(unittest.TestCase):
    def test_one_exact_compile_observation(self):
        self.assertEqual(tool.relation_observation(encoded(test_events())), 456)

    def test_stale_ambiguous_or_incomplete_output_fails(self):
        events = test_events()
        wrong_circuit = [{**row, "Output": row["Output"].replace(tool.RELATION["r1cs_sha256"], "1" * 64)}
                         if "r1cs_sha256=" in row.get("Output", "") else row for row in events]
        wrong_shape = [{**row, "Output": row["Output"].replace("constraints=104671", "constraints=2")}
                       if "relation-only:" in row.get("Output", "") else row for row in events]
        wrong_package = [{**row, "Package": "other"} for row in events]
        for bad in (events[1:], events[:-1], events + events, wrong_circuit, wrong_shape, wrong_package):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                tool.relation_observation(encoded(bad))


class GitArchiveTests(unittest.TestCase):
    def test_archives_exact_commit_and_not_the_working_tree_or_key_payloads(self):
        with tempfile.TemporaryDirectory() as tmp:
            fixture = Path(tmp)
            repo = fixture / "repo"
            repo.mkdir()

            def git(*args, data=None):
                return subprocess.run(
                    ["git", "-C", str(repo), *args], input=data, capture_output=True, check=True,
                ).stdout.strip().decode()

            git("init", "-q")
            source_blob = git("hash-object", "-t", "blob", "-w", "--stdin", data=b"pinned circuit source")
            changed_blob = git("hash-object", "-t", "blob", "-w", "--stdin", data=b"unreviewed source")
            env = ("-c", "user.name=Offline Test", "-c", "user.email=offline@example.test")

            def commit_for(name, blob):
                tree = git("mktree", data=f"100644 blob {blob}\t{name}\n".encode())
                root = git("mktree", data=f"040000 tree {tree}\tprover\n".encode())
                commit = git(*env, "commit-tree", root, data=b"test source\n")
                return commit, tree

            commit, tree = commit_for("original.go", source_blob)
            _, changed_tree = commit_for("original.go", changed_blob)
            (repo / "prover").mkdir()
            (repo / "prover" / "original.go").write_text("dirty fixture working tree")
            with self.assertRaisesRegex(ValueError, "source tree differs"):
                tool.source_archive(repo, commit, "prover", changed_tree, fixture / "wrong")
            stage = fixture / "stage"
            tool.source_archive(repo, commit, "prover", tree, stage)
            self.assertEqual((stage / "prover" / "original.go").read_bytes(), b"pinned circuit source")
            self.assertEqual((repo / "prover" / "original.go").read_text(), "dirty fixture working tree")
            unfit_commit, unfit_tree = commit_for("example.key", source_blob)
            with self.assertRaisesRegex(ValueError, "unexpected members"):
                tool.source_archive(repo, unfit_commit, "prover", unfit_tree, fixture / "unfit")
            self.assertFalse((fixture / "unfit").exists())


class CLITests(unittest.TestCase):
    def invoke(self, *args):
        return subprocess.run([sys.executable, tool.__file__, *map(str, args)], capture_output=True)

    def test_reference_is_byte_stable_and_does_not_claim_compile(self):
        first = self.invoke("reference")
        second = self.invoke("reference")
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(first.stdout, second.stdout)
        self.assertEqual(json.loads(first.stdout)["status"], "PINNED_RELATION_REFERENCE_NOT_RECOMPILED")

    def test_raw_artifact_needs_exact_content_and_separate_pin(self):
        with tempfile.TemporaryDirectory() as tmp:
            fixture = Path(tmp)
            manifest, vk = fixture / "manifest", fixture / "raw-vk"
            raw = tool.canonical(release())
            manifest.write_bytes(raw)
            args = ("check", "--manifest", manifest, "--raw-vk", vk, "--expected-manifest-sha256")
            vk.write_bytes(DUMMY)
            result = self.invoke(*args, digest(raw))
            self.assertEqual(result.returncode, 0, result.stderr)
            receipt = json.loads(result.stdout)
            self.assertEqual(receipt["status"], "PUBLIC_VK_BYTES_MATCH_SUPPLIED_MANIFEST_PIN")
            self.assertIs(receipt["issuer_authenticated_by_this_tool"], False)
            self.assertIs(receipt["proof_or_runtime_qualified"], False)
            for bad in (b"!" + DUMMY[1:], DUMMY + b"\0", DUMMY[:-1]):
                vk.write_bytes(bad)
                result = self.invoke(*args, digest(raw))
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertEqual(result.stdout, b"")
            vk.unlink()
            result = self.invoke(*args, "1" * 64)
            self.assertEqual(result.returncode, 1)
            self.assertIn(b"independently supplied", result.stderr)
            self.assertEqual(result.stdout, b"")

    def test_no_symbolic_links_for_real_file_checks(self):
        with tempfile.TemporaryDirectory() as tmp:
            fixture = Path(tmp)
            a, b = fixture / "actual", fixture / "link"
            a.write_bytes(DUMMY)
            b.symlink_to(a)
            with self.assertRaises(OSError):
                tool.read_regular(b, len(DUMMY))


if __name__ == "__main__":
    unittest.main()
