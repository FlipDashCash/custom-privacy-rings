#!/usr/bin/env python3
"""Offline identity/export and byte intake for the dormant Jupiter descriptor."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tarfile
import tempfile

CUSTOM = "685a7437eddf4fa2559128c4ef7a36ab3c61d4d3"
PRIVACY = "0616075910dca92a33394ef149611cf9d9e963d2"
RELATION = {
    "schema": "jupiter-measured-deposit-descriptor-v1",
    "tag": 68,
    "wire_version": 1,
    "custom_prover_tree": "bb280b8cb3386f744e9e8ef76cd211751876fa7b",
    "privacy_commit": PRIVACY,
    "privacy_prover_server_tree": "2a367cfd058873a5f9f247216e35febae4321c84",
    "descriptor_go_git_blob": "5126dcc69e1e0780e56cb44d127155a0ae7026dc",
    "go": "go1.27.3",
    "gnark": "v0.16.4",
    "gnark_crypto": "v0.22.0",
    "curve": "BN254",
    "compress_threshold": 300,
    "constraints": 104671,
    "public_input": "PublicInputHash",
    "public_inputs": 1,
    "private_bsb22_commitments": 1,
    "public_committed": 0,
    "r1cs_encoding": "gnark.ConstraintSystem.WriteTo",
    "r1cs_sha256": "746aea82a384ca80d53c988242371a1789a04d4dc88b4d5390aa7a40dfaaeaa1",
}
TEST = "TestJupiterDescriptorRelationConstraints"
PACKAGE = "github.com/helius-labs/zolana-custom-rings/prover/circuits/dynamicdescriptor"
RELEASE_SCHEMA = "jupiter-descriptor-public-vk-release-v1"
VK_ENCODING = "gnark-bn254-groth16.VerifyingKey.WriteRawTo"
MAX_MANIFEST = 16 * 1024
MAX_RAW_VK = 1024 * 1024
SHAPE = (
    "relation-only: curve=BN254 compression_threshold=300 constraints=104671 "
    "public_inputs=1 public_input=PublicInputHash private_bsb22_commitments=1 public_committed=0"
)
SHAPE_LOG = re.compile(r"\s*jupiter_relation_shape_test\.go:\d+: " + re.escape(SHAPE))
SERIAL_LOG = re.compile(
    r"\s*jupiter_relation_shape_test\.go:\d+: NO_SETUP NO_PROOF: "
    r"r1cs_bytes=([1-9][0-9]{0,14}) r1cs_sha256=" + RELATION["r1cs_sha256"]
)


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False) + "\n").encode()


def has_fields(value, names):
    return isinstance(value, dict) and value.keys() == set(names.split())


def is_hex(value, count):
    return (
        isinstance(value, str)
        and re.fullmatch(f"[0-9a-f]{{{count}}}", value) is not None
        and any(digit != "0" for digit in value)
    )


def read_regular(path, cap, expected_size=None):
    # No symlink or pipe; cap memory and check the opened descriptor, not the path.
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as source:
        before = os.fstat(source.fileno())
        if not stat.S_ISREG(before.st_mode) or not 0 < before.st_size <= cap:
            raise ValueError("input is not a bounded nonempty regular file")
        if expected_size is not None and before.st_size != expected_size:
            raise ValueError("public VK size differs from release manifest")
        data = source.read(before.st_size + 1)
        after = os.fstat(source.fileno())
        if len(data) != before.st_size or (before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (
            after.st_size, after.st_mtime_ns, after.st_ctime_ns
        ):
            raise ValueError("input changed while reading")
    return data


def checked_manifest(data, expected_sha):
    if not is_hex(expected_sha, 64) or hashlib.sha256(data).hexdigest() != expected_sha:
        raise ValueError("manifest differs from independently supplied expected SHA-256")
    manifest = json.loads(data)
    if data != canonical(manifest) or not has_fields(manifest, "schema relation release public_vk"):
        raise ValueError("release manifest is not canonical or has unexpected fields")
    if manifest["schema"] != RELEASE_SCHEMA or canonical(manifest["relation"]) != canonical(RELATION):
        raise ValueError("release does not bind the exact descriptor relation")
    release = manifest["release"]
    if not has_fields(release, "repository commit") or not is_hex(release["commit"], 40):
        raise ValueError("release has no immutable source commit")
    if not isinstance(release["repository"], str) or not re.fullmatch(
        r"[A-Za-z0-9_-]{1,39}/[A-Za-z0-9_.-]{1,100}", release["repository"]
    ):
        raise ValueError("release repository is not an owner/repo identifier")
    vk = manifest["public_vk"]
    if (
        not has_fields(vk, "encoding bytes sha256")
        or vk["encoding"] != VK_ENCODING
        or type(vk["bytes"]) is not int
        or not 0 < vk["bytes"] <= MAX_RAW_VK
        or not is_hex(vk["sha256"], 64)
    ):
        raise ValueError("release does not pin bounded raw public VK bytes")
    return vk


def check_release(manifest_path, expected_sha, vk_path):
    vk = checked_manifest(read_regular(manifest_path, MAX_MANIFEST), expected_sha)
    artifact = read_regular(vk_path, MAX_RAW_VK, vk["bytes"])
    if hashlib.sha256(artifact).hexdigest() != vk["sha256"]:
        raise ValueError("raw public VK differs from release manifest")
    return {
        "status": "PUBLIC_VK_BYTES_MATCH_SUPPLIED_MANIFEST_PIN",
        "manifest_sha256": expected_sha,
        "r1cs_sha256": RELATION["r1cs_sha256"],
        "public_vk_sha256": vk["sha256"],
        "issuer_authenticated_by_this_tool": False,
        "proof_or_runtime_qualified": False,
    }


def relation_observation(output):
    events = [json.loads(line) for line in output.splitlines()]
    if any(not isinstance(event, dict) for event in events):
        raise ValueError("malformed Go test output")
    package = [event for event in events if event.get("Package") == PACKAGE]
    actions = [(event.get("Test"), event.get("Action")) for event in package]
    if (
        actions.count((TEST, "run")) != 1
        or actions.count((TEST, "pass")) != 1
        or actions.count((None, "pass")) != 1
        or any(action in ("fail", "skip") for _, action in actions)
        or any(action == "run" and test != TEST for test, action in actions)
    ):
        raise ValueError("exact keyless relation test did not pass once")
    logs = [
        line
        for event in package
        if event.get("Test") == TEST and event.get("Action") == "output"
        for line in event.get("Output", "").splitlines()
    ]
    shape = [line for line in logs if "relation-only:" in line]
    serial = [line for line in logs if "NO_SETUP NO_PROOF:" in line]
    if len(shape) != 1 or not SHAPE_LOG.fullmatch(shape[0]) or len(serial) != 1:
        raise ValueError("missing or ambiguous descriptor shape or serialization")
    match = SERIAL_LOG.fullmatch(serial[0])
    if not match:
        raise ValueError("compiled relation differs from the pinned R1CS serialization")
    return int(match.group(1))


def git(repo, *args):
    run = subprocess.run(
        ["git", "--no-replace-objects", "-C", str(repo), *args], capture_output=True, check=False
    )
    if run.returncode:
        raise ValueError("exact local source Git objects are missing")
    return run.stdout


def source_archive(repo, commit, scope, tree_sha, destination):
    if git(repo, "rev-parse", f"{commit}:{scope}").strip().decode("ascii") != tree_sha:
        raise ValueError("pinned relation source tree differs")
    tar = git(repo, "archive", "--format=tar", commit, scope)
    with tarfile.open(fileobj=io.BytesIO(tar)) as archive:
        members = archive.getmembers()
        if any(
            not (member.isfile() or member.isdir())
            or Path(member.name).suffix in {".key", ".pk", ".vk", ".pem"}
            for member in members
        ):
            raise ValueError("relation source archive contains unexpected members")
        archive.extractall(destination, filter="data")


def compile_relation(checkout):
    env = {
        name: os.environ[name] for name in ("HOME", "PATH", "GOCACHE", "GOMODCACHE") if name in os.environ
    }
    env.update(
        GOTOOLCHAIN="local", GOPROXY="off", GOSUMDB="off", GONOPROXY="none",
        GOPRIVATE="", GOWORK="off", GOFLAGS="", GOENV="off", CGO_ENABLED="0",
    )
    version = subprocess.run(["go", "env", "GOVERSION"], env=env, capture_output=True)
    if version.returncode or version.stdout.strip() != RELATION["go"].encode():
        raise ValueError("expected installed Go 1.27.3 and offline dependency cache")
    checkout = checkout.resolve()
    if git(checkout, "rev-parse", f"{CUSTOM}:zolana").strip().decode("ascii") != PRIVACY:
        raise ValueError("pinned nested source commit differs")
    with tempfile.TemporaryDirectory(prefix="jupiter-descriptor-source-") as tmp:
        exact = Path(tmp)
        source_archive(checkout, CUSTOM, "prover", RELATION["custom_prover_tree"], exact)
        source_archive(
            checkout / "zolana", PRIVACY, "prover/server", RELATION["privacy_prover_server_tree"],
            exact / "zolana",
        )
        result = subprocess.run(
            ["go", "test", "-mod=readonly", "-count=1", "-run", f"^{TEST}$", "-json", "./circuits/dynamicdescriptor"],
            cwd=exact / "prover", env=env, capture_output=True,
        )
        if result.returncode:
            raise ValueError("keyless pinned Go test failed; check exact offline module cache")
        observed = relation_observation(result.stdout.decode("utf-8"))
    return {"status": "PINNED_KEYLESS_RELATION_COMPILED", "relation": RELATION, "r1cs_bytes": observed}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("reference", help="print the pinned source identity without compiling")
    compile_command = commands.add_parser("compile", help="recompile the exact offline relation and print its identity")
    compile_command.add_argument("--checkout", type=Path, default=Path(__file__).resolve().parents[1])
    check = commands.add_parser("check", help="compare raw public VK bytes against an independently pinned manifest")
    check.add_argument("--manifest", type=Path, required=True)
    check.add_argument("--expected-manifest-sha256", required=True, help="digest from a separately approved source")
    check.add_argument("--raw-vk", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == "reference":
            receipt = {"status": "PINNED_RELATION_REFERENCE_NOT_RECOMPILED", "relation": RELATION}
        elif args.command == "compile":
            receipt = compile_relation(args.checkout)
        else:
            receipt = check_release(args.manifest, args.expected_manifest_sha256, args.raw_vk)
    except (ValueError, OSError, RecursionError, tarfile.TarError) as error:
        detail = str(error) if type(error) is ValueError else "invalid or unavailable input"
        parser.exit(1, f"Descriptor provenance refused: {detail}\n")
    sys.stdout.buffer.write(canonical(receipt))


if __name__ == "__main__":
    main()
