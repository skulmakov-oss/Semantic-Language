"""Match arms follow C0 dispatch: explicit arms first, '_' as the fallback (Codex P2 on PR #66).

C0 (`skulmakov-oss/Semantic@1d1e76c7`) evaluates `match 1 { _ => { 10 } 1 => { 20 } }` to 20 and
rejects a function whose arm after '_' can fall through ("may exit without returning"). The
lowering mirrors C0 byte for byte; the return-path proof must agree with that dispatch.
"""
import os
import pathlib
import subprocess
import tempfile
import unittest

REPO = pathlib.Path(__file__).resolve().parents[2]
SMC = os.environ.get("SEMANTIC_SMC")


def compile_with_semantic(root, name):
    result = subprocess.run(
        [SMC, "run", "main.sm", "--profile", "cli-read-only", "--root", root, "--envelope", "trusted-compiler",
         "--max-steps", "20000000", "--max-calls", "100000", "--max-frames", "256", "--", name],
        cwd=REPO / "compiler", capture_output=True, text=True, encoding="utf-8", timeout=1800)
    return result, next((l for l in result.stdout.splitlines() if l.startswith(("hex ", "error "))), "")


@unittest.skipUnless(SMC and pathlib.Path(SMC).is_file(), "SEMANTIC_SMC is required")
class MatchDefaultOrderTests(unittest.TestCase):
    def test_arm_after_default_runs_like_c0(self):
        with tempfile.TemporaryDirectory(prefix="semantic-match-order-") as tmp:
            root = pathlib.Path(tmp)
            (root / "prog.sm").write_text(
                "fn main() { let v: i32 = match 1 { _ => { 10 } 1 => { 20 } }; assert(v == 20); }\n",
                encoding="utf-8", newline="")
            result, line = compile_with_semantic(root, "prog.sm")
            self.assertTrue(line.startswith("hex "), result.stdout + result.stderr)
            out = root / "prog.smc"
            out.write_bytes(bytes.fromhex(line[4:].strip()))
            reference = root / "prog.c0.smc"
            subprocess.run([SMC, "compile", root / "prog.sm", "-o", reference], check=True, capture_output=True, timeout=600)
            self.assertEqual(out.read_bytes(), reference.read_bytes())
            run = subprocess.run([SMC, "run-smc", out], capture_output=True, text=True, encoding="utf-8", timeout=600)
            self.assertEqual(run.returncode, 0, run.stdout + run.stderr)

    def test_fallthrough_arm_after_default_is_rejected_like_c0(self):
        with tempfile.TemporaryDirectory(prefix="semantic-match-order-") as tmp:
            root = pathlib.Path(tmp)
            (root / "prog.sm").write_text(
                "fn f(x: i32) -> i32 { match x { _ => { return 1; } 1 => { } } }\nfn main() { assert(f(1) == 1); }\n",
                encoding="utf-8", newline="")
            result, line = compile_with_semantic(root, "prog.sm")
            self.assertTrue(line.startswith("error "), result.stdout + result.stderr)
            self.assertIn("return", line)
            c0 = subprocess.run([SMC, "compile", root / "prog.sm", "-o", root / "c0.smc"], capture_output=True,
                                text=True, encoding="utf-8", timeout=600)
            self.assertNotEqual(c0.returncode, 0)
            self.assertIn("may exit without returning", c0.stdout + c0.stderr)


if __name__ == "__main__":
    unittest.main()
