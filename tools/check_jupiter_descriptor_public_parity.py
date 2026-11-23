#!/usr/bin/env python3
"""Compare two offline stock hash implementations on the frozen public case."""

import os
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
PRIVACY = "0616075910dca92a33394ef149611cf9d9e963d2"
PINS = {
    "Cargo.lock": "4f9bc471770f30ab0f7e759fe73c71badf8b1a53",
    "prover/go.mod": "21108269a9edb1bc84cca1b92e29a5f1972c0bc9",
    "prover/go.sum": "5413e2cf4ddfd1a79f666c70771f90bb2b8ba4ec",
    "prover/circuits/dynamicdescriptor/descriptor.go": "5126dcc69e1e0780e56cb44d127155a0ae7026dc",
    "program-libs/interface/src/proof/jupiter_measured_descriptor_statement_draft.rs": "cdc98c73d0f9fa3508b28891645af6dd07c09b45",
    "test-vectors/settlement/jupiter_measured_descriptor_v1_public_input.json": "14291591cd6913bd7002cbf668d2be26c4afb0a0",
}
MARKER = re.compile(r"PYROL_JUPITER_V1_PUBLIC_PARITY ([0-9a-f]{1600})\.([0-9a-f]{64})$", re.MULTILINE)


def run(args, cwd=ROOT, env=None):
    run_result = subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True)
    if run_result.returncode:
        raise RuntimeError(f"{args[0]} exited {run_result.returncode}: {run_result.stderr.strip()}")
    return run_result.stdout


def projection(output, label):
    matches = MARKER.findall(output)
    if len(matches) != 1:
        raise RuntimeError(f"{label} did not produce exactly one 25-cell public result")
    cells, public_input = matches[0]
    return [cells[i : i + 64] for i in range(0, len(cells), 64)], public_input


def main():
    for path, expected in PINS.items():
        if run(["git", "hash-object", path]).strip() != expected:
            raise RuntimeError(f"frozen public parity source differs: {path}")
    if run(["git", "rev-parse", "HEAD"], ROOT / "zolana").strip() != PRIVACY:
        raise RuntimeError("wrong owned Privacy checkout")
    if run(
        ["git", "status", "--porcelain", "--", "prover/server", "program-libs/hasher"],
        ROOT / "zolana",
    ).strip():
        raise RuntimeError("pinned nested protocol graph is not clean")

    rust = dict(os.environ, CARGO_NET_OFFLINE="true", RUSTUP_TOOLCHAIN="1.98.1", RUSTUP_AUTO_INSTALL="0")
    go = dict(
        os.environ,
        GOTOOLCHAIN="local",
        GOPROXY="off",
        GOSUMDB="off",
        GONOPROXY="none",
        GOPRIVATE="",
        GOWORK="off",
        GOFLAGS="",
    )
    if not run(["rustc", "--version"], env=rust).startswith("rustc 1.98.1 "):
        raise RuntimeError("expected installed Rust 1.98.1")
    if run(["go", "env", "GOVERSION"], ROOT / "prover", go).strip() != "go1.27.3":
        raise RuntimeError("expected installed Go 1.27.3")

    from_rust = projection(run([
        "cargo", "test", "--offline", "--locked", "-p", "custom-ring-program",
        "--test", "jupiter_measured_v1_public_parity", "--",
        "--exact", "prints_one_frozen_public_descriptor_statement", "--nocapture",
    ], env=rust), "Rust")
    from_go = projection(run([
        "go", "test", "-mod=readonly", "-count=1", "-run", "^TestJupiterDescriptorPublicInputParity$",
        "-v", "./circuits/dynamicdescriptor",
    ], ROOT / "prover", go), "Go")
    for index, (a, b) in enumerate(zip(from_rust[0], from_go[0])):
        if a != b:
            raise RuntimeError(f"frozen public statement differs at cell {index}")
    if from_rust[1] != from_go[1]:
        raise RuntimeError("frozen public input hash differs despite matching cells")
    print(f"Matched 25 public CRDD cells and PublicInputHash={from_rust[1]}; no proof was tested")


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError) as error:
        print(f"Public descriptor parity refused: {error}", file=sys.stderr)
        sys.exit(1)
