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

By default, --reference-checkout must be a clean checkout with its git
HEAD exactly at the frozen reference commit (FROZEN_REFERENCE_COMMIT
below) - otherwise this proves nothing about the B0-00 contract, no
matter what the tables say. Pass --allow-reference-drift to intentionally
run a regression check against a different commit instead.

Exit code 0 means all three legs of the triangle hold: the frozen corpus
matches an independently-computed normative B0 algebra, the live
extraction matches that same normative algebra, and (redundantly, but
checked explicitly) the frozen corpus matches the live extraction. Exit
code 1 means any of: --reference-checkout isn't the frozen commit (or is
dirty) and drift wasn't allowed; the frozen corpus's own reference
metadata (repository/commit/crate) doesn't match what's actually frozen;
a required field is missing, mis-shaped, or has malformed/out-of-domain/
incorrectly-covered entries on either side (fail-closed); either side
disagrees with the normative algebra (closes the case where frozen corpus
and live extraction are corrupted *identically*, which frozen==live
agreement alone cannot catch); or an unexplained frozen-vs-live delta.
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
FROZEN_REFERENCE_REPOSITORY = "skulmakov-oss/Semantic"
FROZEN_REFERENCE_COMMIT = "979def10135e1a90d7333d5501405343d498579e"
FROZEN_REFERENCE_CRATE = "semantic-core-quad"
UNARY_TABLES = {"not": ("a",)}
BINARY_TABLES = {"and": ("a", "b"), "or": ("a", "b"), "implies": ("a", "b"), "eq": ("a", "b")}
TABLE_LEN = {**{k: 4 for k in UNARY_TABLES}, **{k: 16 for k in BINARY_TABLES}}
CANONICAL_ORDER = ["N", "F", "T", "S"]  # matches dump_b0_vectors.rs's explicit row/column order
NORMATIVE_TABLE_KEYS = ["not", "and", "or", "implies", "eq"]


def _normative_tables() -> dict:
    """Independently re-derives the frozen B0 legacy-lattice algebra from
    FROZEN_STATE_ENCODING's own bit values and the documented formulas
    (docs/spec/quad_logic_frame_v1.md: NOT = swap truth/falsity planes,
    AND = bitwise AND, OR = bitwise OR, IMPLIES = NOT(a) OR b) - entirely
    independent of both the Rust reference and the committed corpus.

    This is the third leg of the triangle: frozen==live agreement alone
    proves nothing if both sides can be corrupted identically (e.g. `T AND
    T` changed to `S` in both the corpus and the probe at once - shape,
    domain and coverage checks all still pass, and frozen==live trivially
    holds). Every table is additionally required to match this
    independently-computed oracle, which touches neither of the two files
    under comparison.
    """
    bits = FROZEN_STATE_ENCODING
    name_of = {v: k for k, v in bits.items()}

    def not_(a):
        c = bits[a]
        return name_of[((c & 0b10) >> 1) | ((c & 0b01) << 1)]

    def and_(a, b):
        return name_of[bits[a] & bits[b]]

    def or_(a, b):
        return name_of[bits[a] | bits[b]]

    def implies_(a, b):
        return name_of[bits[not_(a)] | bits[b]]

    return {
        "not": [{"a": a, "result": not_(a)} for a in CANONICAL_ORDER],
        "and": [{"a": a, "b": b, "result": and_(a, b)} for a in CANONICAL_ORDER for b in CANONICAL_ORDER],
        "or": [{"a": a, "b": b, "result": or_(a, b)} for a in CANONICAL_ORDER for b in CANONICAL_ORDER],
        "implies": [{"a": a, "b": b, "result": implies_(a, b)} for a in CANONICAL_ORDER for b in CANONICAL_ORDER],
        "eq": [{"a": a, "b": b, "result": a == b} for a in CANONICAL_ORDER for b in CANONICAL_ORDER],
    }


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


def check_reference_metadata(frozen: dict) -> list:
    # The frozen corpus's own "reference" block (repository/commit/crate)
    # names exactly what was extracted against. It is not part of
    # COMPARED_KEYS (the live extraction has no such field), so nothing
    # else in this script would notice it silently drifting - validate it
    # against the frozen identity explicitly.
    problems = []
    ref = frozen.get("reference")
    if not isinstance(ref, dict):
        problems.append(f"frozen.reference = {ref!r}, expected an object")
        return problems
    checks = (
        ("repository", FROZEN_REFERENCE_REPOSITORY),
        ("commit", FROZEN_REFERENCE_COMMIT),
        ("crate", FROZEN_REFERENCE_CRATE),
    )
    for field, expected in checks:
        actual = ref.get(field)
        if actual != expected:
            problems.append(f"frozen.reference.{field} = {actual!r}, expected exactly {expected!r}")
    return problems


def check_checkout_identity(reference_checkout: Path, allow_drift: bool) -> list:
    # Validating the corpus's *claimed* commit (check_reference_metadata)
    # proves nothing about what --reference-checkout actually points at.
    # Without this, any checkout that happens to produce matching Quad
    # behavior - a newer commit, an older one, a dirty working tree, even
    # an unrelated repo with a compatible crate - would qualify as if it
    # were the frozen oracle.
    if allow_drift:
        return []
    problems = []

    def run_git(*args):
        return subprocess.run(
            ["git", "-C", str(reference_checkout), *args],
            capture_output=True, text=True, check=True,
        )

    try:
        head = run_git("rev-parse", "HEAD").stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError) as exc:
        problems.append(f"could not determine --reference-checkout's git HEAD: {exc}")
        return problems
    if head != FROZEN_REFERENCE_COMMIT:
        problems.append(
            f"--reference-checkout HEAD is {head}, expected the frozen reference commit "
            f"{FROZEN_REFERENCE_COMMIT}. This gate proves B0-00 against that exact commit; "
            f"pass --allow-reference-drift to intentionally run it against a different one "
            f"(the result is then a regression check, not a B0 qualification proof)."
        )

    try:
        status = run_git("status", "--porcelain").stdout
    except subprocess.CalledProcessError as exc:
        problems.append(f"could not check --reference-checkout's git status: {exc}")
        return problems
    if status.strip():
        problems.append(
            "--reference-checkout has uncommitted changes (git status --porcelain is "
            "non-empty). Pass --allow-reference-drift to intentionally qualify against a "
            "modified checkout."
        )

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
    parser.add_argument(
        "--allow-reference-drift",
        action="store_true",
        help=(
            "Skip the default check that --reference-checkout's git HEAD is exactly the "
            "frozen reference commit and its working tree is clean. Use this only to "
            "intentionally run a regression check against a different commit - the result "
            "is then not a B0-00 qualification proof."
        ),
    )
    args = parser.parse_args()

    checkout = args.reference_checkout.resolve()
    identity_problems = check_checkout_identity(checkout, args.allow_reference_drift)
    if identity_problems:
        print("FAIL: --reference-checkout does not match the frozen reference identity:")
        for p in identity_problems:
            print(f"  - {p}")
        return 1

    frozen = json.loads(FROZEN_VECTORS.read_text())
    live = extract_live(checkout)

    metadata_problems = check_reference_metadata(frozen)
    if metadata_problems:
        print("FAIL: frozen corpus's own reference metadata is wrong:")
        for p in metadata_problems:
            print(f"  - {p}")
        return 1

    shape_problems = check_shape("frozen", frozen) + check_shape("live", live)
    if shape_problems:
        print("FAIL: corpus is not structurally complete (fail-closed):")
        for p in shape_problems:
            print(f"  - {p}")
        return 1

    normative = _normative_tables()
    normative_problems = []
    for key in NORMATIVE_TABLE_KEYS:
        for label, data in (("frozen", frozen), ("live", live)):
            if data[key] != normative[key]:
                normative_problems.append((label, key))
    if normative_problems:
        print(
            "FAIL: disagrees with the independently-computed normative B0 algebra "
            "(frozen==live agreement alone is not sufficient - both sides can be "
            "wrong the same way):"
        )
        for label, key in normative_problems:
            data = frozen if label == "frozen" else live
            print(f"  --- {label}.{key} ---")
            print(f"  {json.dumps(data[key])}")
            print(f"  --- normative.{key} ---")
            print(f"  {json.dumps(normative[key])}")
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
