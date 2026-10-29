"""Focused OWN0 qualification for the Semantic-written compiler.

The test is intentionally fail-closed: it runs only with the approved compiler
role supplied through SEMANTIC_SMC and never substitutes the host Rust compiler.
"""

import os
import pathlib
import subprocess
import tempfile
import unittest


HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent.parent
SMC = os.environ.get("SEMANTIC_SMC")


def own_counts(artifact: bytes):
    counts = []
    start = 0
    while True:
        offset = artifact.find(b"OWN0", start)
        if offset < 0:
            return counts
        if offset + 6 > len(artifact):
            return counts
        counts.append(int.from_bytes(artifact[offset + 4:offset + 6], "little"))
        start = offset + 4


@unittest.skipUnless(SMC and pathlib.Path(SMC).is_file(), "SEMANTIC_SMC is required")
class OwnershipSemcodeTests(unittest.TestCase):
    def test_record_updates_emit_nonempty_own0_and_verify(self):
        source = """record R { x: i32, y: i32 }

fn main() {
    let x: i32 = 1;
    x = 2;
    let r: R = R { x: 1, y: 2 };
    let s: R = r with { x: 3 };
    assert(s.x == 3);
    return;
}
"""
        with tempfile.TemporaryDirectory(prefix="semantic-own0-") as tmp:
            root = pathlib.Path(tmp)
            (root / "ownership.sm").write_text(source, encoding="utf-8", newline="")
            result = subprocess.run(
                [SMC, "run", "main.sm", "--profile", "cli-read-only",
                 "--root", root, "--envelope", "trusted-compiler",
                 "--max-steps", "20000000", "--max-calls", "50000",
                 "--max-frames", "256", "--", "ownership.sm"],
                cwd=REPO / "compiler", capture_output=True, text=True,
                encoding="utf-8", timeout=1800,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            line = next((line for line in result.stdout.splitlines() if line.startswith("hex ")), "")
            self.assertTrue(line, result.stdout + result.stderr)
            artifact = bytes.fromhex(line[4:].strip())
            counts = own_counts(artifact)
            self.assertTrue(counts and max(counts) >= 2, counts)

            out = root / "ownership.smc"
            out.write_bytes(artifact)
            verified = subprocess.run([SMC, "verify", out], capture_output=True,
                                      text=True, encoding="utf-8", timeout=1800)
            self.assertEqual(verified.returncode, 0, verified.stdout + verified.stderr)
            executed = subprocess.run([SMC, "run-smc", out], capture_output=True,
                                       text=True, encoding="utf-8", timeout=1800)
            self.assertEqual(executed.returncode, 0, executed.stdout + executed.stderr)


if __name__ == "__main__":
    unittest.main()
