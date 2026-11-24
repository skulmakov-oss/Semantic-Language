"""A unit-valued if expression has no result local: no store, no load (Codex P2 on PR #66).

C0 rejects these sources ("value-producing block requires trailing value expression"), so they are
outside the C0-parity corpus. The Semantic compiler has always accepted them; lowering must still
emit an admitted artifact that runs, never fault or read an unassigned slot.
"""
import os
import pathlib
import subprocess
import tempfile
import unittest

REPO = pathlib.Path(__file__).resolve().parents[2]
SMC = os.environ.get("SEMANTIC_SMC")

SOURCES = {
    "no_locals": "fn main() { (if true {} else {}); }\n",
    "with_local": "fn main() { let x: i32 = 1; (if true {} else {}); assert(x == 1); }\n",
    "with_calls": ("fn f() { return; }\n"
                   "fn main() { let y: i32 = 7; let t: text = \"s\"; (if y == 7 { f(); } else { f(); }); assert(y == 7); }\n"),
}


@unittest.skipUnless(SMC and pathlib.Path(SMC).is_file(), "SEMANTIC_SMC is required")
class UnitIfExpressionTests(unittest.TestCase):
    def test_unit_valued_if_expression_emits_a_running_artifact(self):
        for name, source in SOURCES.items():
            with self.subTest(name), tempfile.TemporaryDirectory(prefix="semantic-unit-if-") as tmp:
                root = pathlib.Path(tmp)
                (root / "prog.sm").write_text(source, encoding="utf-8", newline="")
                result = subprocess.run(
                    [SMC, "run", "main.sm", "--profile", "cli-read-only", "--root", root,
                     "--envelope", "trusted-compiler", "--max-steps", "20000000", "--max-calls", "100000",
                     "--max-frames", "256", "--", "prog.sm"],
                    cwd=REPO / "compiler", capture_output=True, text=True, encoding="utf-8", timeout=1800)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                line = next((l for l in result.stdout.splitlines() if l.startswith("hex ")), "")
                self.assertTrue(line, result.stdout + result.stderr)
                out = root / "prog.smc"
                out.write_bytes(bytes.fromhex(line[4:].strip()))
                for cmd in (["verify", out], ["run-smc", out]):
                    run = subprocess.run([SMC, *cmd], capture_output=True, text=True, encoding="utf-8", timeout=600)
                    self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
                    self.assertNotIn("uninitialized", run.stdout + run.stderr)


if __name__ == "__main__":
    unittest.main()
