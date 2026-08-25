#!/usr/bin/env python3
"""B0-00 differential qualification gate (Semantic-Language issue #2).

Re-extracts the legacy-lattice Quad truth tables from a live checkout of the
reference repository (`skulmakov-oss/Semantic`) by compiling and running the
probe at reference/b0/dump_b0_vectors.rs against its `semantic-core-quad`
crate, then compares the result byte-for-byte (structurally) against the
frozen corpus committed at reference/b0/reference_vectors.json.

Usage:
    python3 qualification/b0/check_reference_vectors.py --reference-checkout <path-to-Semantic-repo>

Exit code 0 = no unexplained deltas. Exit code 1 = divergence found.
"""
import argparse
import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
FROZEN_VECTORS = REPO_ROOT / "reference" / "b0" / "reference_vectors.json"
PROBE_SOURCE = REPO_ROOT / "reference" / "b0" / "dump_b0_vectors.rs"
COMPARED_KEYS = ["operation_family", "state_encoding", "not", "and", "or", "implies"]


def _atomic_write(path: Path, data: bytes) -> None:
    # Write-then-replace so a pre-existing file at `path` is never left
    # partially overwritten (disk full, permission revoked, interrupt
    # mid-write): the target is always either its old bytes or the new
    # bytes in full, never a half-written mix.
    tmp = path.with_name(path.name + ".b0-tmp")
    tmp.write_bytes(data)
    tmp.replace(path)


def extract_live(reference_checkout: Path) -> dict:
    crate_dir = reference_checkout / "crates" / "semantic-core-quad"
    if not crate_dir.is_dir():
        sys.exit(f"error: {crate_dir} not found - is --reference-checkout the Semantic repo root?")

    examples_dir = crate_dir / "examples"
    examples_dir_existed = examples_dir.is_dir()
    examples_dir.mkdir(exist_ok=True)
    installed_probe = examples_dir / "dump_b0_vectors.rs"
    # Never clobber pre-existing content at this path (local WIP in a shared
    # checkout, or a future upstream file of the same name): back it up and
    # restore it, rather than just skipping deletion of *our* copy. The
    # backup read and the first write both happen inside try/finally so the
    # restore below still runs even if the write itself fails partway.
    original_bytes = installed_probe.read_bytes() if installed_probe.exists() else None
    probe_bytes = PROBE_SOURCE.read_bytes()

    try:
        _atomic_write(installed_probe, probe_bytes)
        result = subprocess.run(
            ["cargo", "run", "-p", "semantic-core-quad", "--example", "dump_b0_vectors", "--quiet"],
            cwd=reference_checkout,
            capture_output=True,
            text=True,
            check=True,
        )
    except subprocess.CalledProcessError as exc:
        sys.exit(f"error: reference probe failed to run:\n{exc.stderr}")
    finally:
        if original_bytes is not None:
            _atomic_write(installed_probe, original_bytes)
        else:
            installed_probe.unlink(missing_ok=True)
            if not examples_dir_existed:
                try:
                    examples_dir.rmdir()
                except OSError:
                    pass  # not empty (other files appeared concurrently); leave it alone

    return json.loads(result.stdout)


def diff(frozen: dict, live: dict) -> list:
    deltas = []
    for key in COMPARED_KEYS:
        if frozen.get(key) != live.get(key):
            deltas.append(key)
    return deltas


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference-checkout", required=True, type=Path)
    args = parser.parse_args()

    frozen = json.loads(FROZEN_VECTORS.read_text())
    live = extract_live(args.reference_checkout.resolve())

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
    print(f"  not: {len(frozen['not'])} cases, and/or/implies: {len(frozen['and'])} cases each")
    return 0


if __name__ == "__main__":
    sys.exit(main())
