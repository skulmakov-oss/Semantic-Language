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
complete. Exit code 1 = divergence found, OR a required field is missing,
mis-shaped, has malformed/out-of-domain entries, or does not cover every
operand combination exactly once, on either side (fail-closed: a field or
its entries degrading identically on both sides must never compare equal
by accident).
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
STATES = {"N", "F", "T", "S"}
# Frozen by the B0-00 contract itself (docs/bootstrap/b0/foundation_contract.md),
# not just "some" shape: after a slice is frozen, the validator must reject a
# technically well-formed but wrong value here, not merely check its shape.
FROZEN_OPERATION_FAMILY = "legacy_lattice"
FROZEN_STATE_ENCODING = {"N": 0, "F": 1, "T": 2, "S": 3}
UNARY_TABLES = {"not": ("a",)}
BINARY_TABLES = {"and": ("a", "b"), "or": ("a", "b"), "implies": ("a", "b"), "eq": ("a", "b")}
TABLE_LEN = {**{k: 4 for k in UNARY_TABLES}, **{k: 16 for k in BINARY_TABLES}}


def _is_plain_int(v) -> bool:
    return type(v) is int  # excludes bool, which is a subclass of int in Python


def _table_problems(label: str, key: str, entries, operand_keys) -> list:
    # Validates each entry's shape and domain, and that every operand
    # combination appears exactly once - not just that the list has the
    # right length. A list of 16 `null`s, or 16 copies of the same pair,
    # would satisfy a length-only check but is not a real truth table.
    problems = []
    expected_keys = set(operand_keys) | {"result"}
    seen_operands = set()
    for i, entry in enumerate(entries):
        if not isinstance(entry, dict) or set(entry.keys()) != expected_keys:
            got = sorted(entry.keys()) if isinstance(entry, dict) else repr(entry)
            problems.append(f"{label}.{key}[{i}] = {got}, expected keys {sorted(expected_keys)}")
            continue
        operands = tuple(entry[k] for k in operand_keys)
        for k, v in zip(operand_keys, operands):
            if v not in STATES:
                problems.append(f"{label}.{key}[{i}].{k} = {v!r}, expected one of {sorted(STATES)}")
        result = entry["result"]
        if key == "eq":
            if not isinstance(result, bool):
                problems.append(f"{label}.{key}[{i}].result = {result!r}, expected a bool")
        elif result not in STATES:
            problems.append(f"{label}.{key}[{i}].result = {result!r}, expected one of {sorted(STATES)}")
        if all(v in STATES for v in operands):
            seen_operands.add(operands)
    expected_count = len(STATES) ** len(operand_keys)
    if len(seen_operands) != expected_count:
        problems.append(
            f"{label}.{key} covers {len(seen_operands)}/{expected_count} distinct operand "
            f"combination(s) - expected each combination exactly once (duplicates or gaps present)"
        )
    return problems

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
        value = data[key]

        if key == "operation_family":
            if value != FROZEN_OPERATION_FAMILY:
                problems.append(f"{label}.{key} = {value!r}, expected exactly {FROZEN_OPERATION_FAMILY!r}")
            continue

        if key == "state_encoding":
            if not isinstance(value, dict) or not all(_is_plain_int(v) for v in value.values()):
                problems.append(f"{label}.{key} = {value!r}, expected int values")
            elif value != FROZEN_STATE_ENCODING:
                problems.append(f"{label}.{key} = {value}, expected exactly {FROZEN_STATE_ENCODING}")
            continue

        operand_keys = UNARY_TABLES.get(key) or BINARY_TABLES.get(key)
        if not isinstance(value, list) or len(value) != TABLE_LEN[key]:
            actual = len(value) if isinstance(value, list) else type(value).__name__
            problems.append(f"{label}.{key} has {actual} cases, expected {TABLE_LEN[key]}")
            continue
        problems.extend(_table_problems(label, key, value, operand_keys))

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
