#!/usr/bin/env python3
"""B0-00 differential qualification gate (Semantic-Language issue #2).

Re-extracts the legacy-lattice Quad truth tables (and structural equality)
from a live checkout of the reference repository (`skulmakov-oss/Semantic`)
by building and running a throwaway crate that depends on its
`semantic-core-quad` crate via a `path` dependency, then compares the
result byte-for-byte (structurally) against the frozen corpus committed at
reference/b0/reference_vectors.json. The probe project lives entirely in
its own temp directory (own Cargo.toml, own Cargo.lock, own target/) -
the reference checkout is only ever read from, never written to, so two
concurrent invocations against the same checkout cannot race on shared
mutable state there.

Usage:
    python3 qualification/b0/check_reference_vectors.py --reference-checkout <path-to-Semantic-repo>

Exit code 0 = no unexplained deltas and the corpus is structurally
complete. Exit code 1 = divergence found, OR a required field is missing
or the wrong shape on either side (fail-closed: a field silently absent
from both sides must never compare equal by accident).
"""
import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
FROZEN_VECTORS = REPO_ROOT / "reference" / "b0" / "reference_vectors.json"
PROBE_SOURCE = REPO_ROOT / "reference" / "b0" / "dump_b0_vectors.rs"
COMPARED_KEYS = ["operation_family", "state_encoding", "not", "and", "or", "implies", "eq"]

# Expected shape per field, checked before any value comparison. Catches a
# whole dimension quietly disappearing from the corpus or the extractor
# (which `dict.get(k) != dict.get(k)` -> `None != None` -> False would miss).
REQUIRED_SHAPE = {
    "state_encoding": ("dict_keys", {"N", "F", "T", "S"}),
    "not": ("list_len", 4),
    "and": ("list_len", 16),
    "or": ("list_len", 16),
    "implies": ("list_len", 16),
    "eq": ("list_len", 16),
}

CARGO_TOML = """\
[package]
name = "b0-extract"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
semantic-core-quad = {{ path = {crate_path} }}
"""


def extract_live(reference_checkout: Path) -> dict:
    crate_dir = reference_checkout / "crates" / "semantic-core-quad"
    if not (crate_dir / "Cargo.toml").is_file():
        sys.exit(f"error: {crate_dir} not found - is --reference-checkout the Semantic repo root?")

    with tempfile.TemporaryDirectory(prefix="b0-extract-") as tmp:
        tmp_path = Path(tmp)
        crate_path_toml = json.dumps(crate_dir.resolve().as_posix())  # safe double-quoted TOML string
        (tmp_path / "Cargo.toml").write_text(CARGO_TOML.format(crate_path=crate_path_toml))
        src_dir = tmp_path / "src"
        src_dir.mkdir()
        (src_dir / "main.rs").write_bytes(PROBE_SOURCE.read_bytes())

        try:
            result = subprocess.run(
                ["cargo", "run", "--manifest-path", str(tmp_path / "Cargo.toml"), "--quiet"],
                capture_output=True,
                text=True,
                check=True,
            )
        except subprocess.CalledProcessError as exc:
            sys.exit(f"error: reference probe failed to run:\n{exc.stderr}")

    return json.loads(result.stdout)


def check_shape(label: str, data: dict) -> list:
    problems = []
    for key in COMPARED_KEYS:
        if key not in data:
            problems.append(f"{label}.{key} is missing")
            continue
        shape = REQUIRED_SHAPE.get(key)
        if shape is None:
            continue
        kind, expected = shape
        value = data[key]
        if kind == "list_len":
            if not isinstance(value, list) or len(value) != expected:
                actual = len(value) if isinstance(value, list) else type(value).__name__
                problems.append(f"{label}.{key} has {actual} cases, expected {expected}")
        elif kind == "dict_keys":
            if not isinstance(value, dict) or set(value.keys()) != expected:
                got = sorted(value.keys()) if isinstance(value, dict) else repr(value)
                problems.append(f"{label}.{key} keys are {got}, expected {sorted(expected)}")
    return problems


def diff(frozen: dict, live: dict) -> list:
    deltas = []
    for key in COMPARED_KEYS:
        if key not in frozen or key not in live:
            continue  # already reported by check_shape; never silently treated as equal
        if frozen[key] != live[key]:
            deltas.append(key)
    return deltas


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference-checkout", required=True, type=Path)
    args = parser.parse_args()

    frozen = json.loads(FROZEN_VECTORS.read_text())
    live = extract_live(args.reference_checkout.resolve())

    shape_problems = check_shape("frozen", frozen) + check_shape("live", live)
    if shape_problems:
        print("FAIL: corpus is not structurally complete (fail-closed):")
        for p in shape_problems:
            print(f"  - {p}")
        return 1

    deltas = diff(frozen, live)
    if deltas:
        print(f"FAIL: unexplained divergence in field(s): {', '.join(deltas)}")
        for key in deltas:
            print(f"  --- frozen.{key} ---")
            print(f"  {json.dumps(frozen.get(key))}")
            print(f"  --- live.{key} ---")
            print(f"  {json.dumps(live.get(key))}")
        return 1

    print("PASS: reference_vectors.json matches live extraction from the reference implementation.")
    print(f"  not: {len(frozen['not'])} cases, and/or/implies/eq: {len(frozen['and'])} cases each")
    return 0


if __name__ == "__main__":
    sys.exit(main())
