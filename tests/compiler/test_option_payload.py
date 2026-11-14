"""PR #63 review regressions: Option::Some payload typing in semantic analysis.

Every result is emitted by the real Semantic compiler under pinned C0. Rejections are checked by
category, message and source offset; accepted programs are verified and executed.
"""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[2]
COMPILER = REPO / "compiler"
C0 = os.environ.get("SEMANTIC_SMC", "")
NOOP = "fn noop() {\n    return;\n}\n\n"
UNIT = "Option::Some payload has unit type"


def call(*args, cwd=COMPILER):
    return subprocess.run([str(x) for x in args], cwd=cwd, capture_output=True,
                          text=True, errors="replace", timeout=180)


@unittest.skipUnless(C0 and Path(C0).is_file(), "SEMANTIC_SMC must identify pinned C0")
class OptionPayloadTests(unittest.TestCase):
    def compile_native(self, root, source):
        (root / "main.sm").write_text(source, encoding="utf-8", newline="\n")
        result = call(C0, "run", "main.sm", "--profile", "cli-read-only",
                      "--root", root, "--envelope", "trusted-compiler",
                      "--max-steps", "20000000", "--max-calls", "50000",
                      "--max-frames", "256", "--", "main.sm")
        self.assertEqual(result.returncode, 0, result.stdout[-200:] + result.stderr[-1500:])
        return result.stdout.strip()

    def test_unit_payload_rejected_in_semantic_analysis(self):
        for name, source, payload in (
            ("function", NOOP + "fn main() {\n    let x = Option::Some(noop());\n    return;\n}\n", "noop()"),
            ("contextual", NOOP + "fn main() {\n    let x: Option(i32) = Option::Some(noop());\n    return;\n}\n", "noop()"),
            ("nested", NOOP + "fn main() {\n    let x = Option::Some(Option::Some(noop()));\n    return;\n}\n", "noop()"),
            ("builtin", 'fn main() {\n    let x = Option::Some(stdout_write("x"));\n    return;\n}\n', "stdout_write("),
        ):
            with self.subTest(case=name), tempfile.TemporaryDirectory() as tmp:
                line = self.compile_native(Path(tmp), source)
                # One diagnostic line and no "hex" artifact: lowering and emission were not reached.
                self.assertEqual(line, f"error type @{source.rindex(payload)}: {UNIT}")

    def test_payload_diagnostics_are_preserved(self):
        for name, source, expected in (
            ("unknown_function", "fn main() {\n    let x = Option::Some(nope());\n    return;\n}\n",
             "error name @37: unknown function 'nope'"),
            ("arity_zero", "fn main() {\n    let x = Option::Some();\n    return;\n}\n",
             "error type @38: Option::Some expects exactly one payload item (found ';')"),
            ("arity_two", "fn main() {\n    let x = Option::Some(1, 2);\n    return;\n}\n",
             "error type @42: Option::Some expects exactly one payload item (found ';')"),
            ("variant", "fn main() {\n    let x = Option::Maybe(1);\n    return;\n}\n",
             "error syntax @37: unknown Option variant 'Maybe' (found '(')"),
            ("none_without_context", "fn main() {\n    let x = Option::None;\n    return;\n}\n",
             "error type @24: Option::None currently requires contextual Option(T) type in v0"),
        ):
            with self.subTest(case=name), tempfile.TemporaryDirectory() as tmp:
                self.assertEqual(self.compile_native(Path(tmp), source), expected)

    def test_value_payloads_compile_verify_and_run(self):
        for name, body in (
            ("i32", "let x = Option::Some(42); assert(x == Option::Some(42));"),
            ("bool", "let x = Option::Some(true); assert(x == Option::Some(true));"),
            ("text", 'let x = Option::Some("hello");'),
            ("nested", "let x: Option(Option(i32)) = Option::Some(Option::Some(7)); "
                       "let y: Option(Option(i32)) = Option::Some(Option::None); assert(x != y);"),
            ("contextual_none", "let x: Option(i32) = Option::None; assert(x != Option::Some(1));"),
            ("unit_call_statement", "let x = Option::Some(1); noop();"),
        ):
            with self.subTest(case=name), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                line = self.compile_native(root, NOOP + f"fn main() {{\n    {body}\n    return;\n}}\n")
                self.assertTrue(line.startswith("hex "), line[:250])
                artifact = root / "option-payload.smc"
                artifact.write_bytes(bytes.fromhex(line[4:]))
                for action in ("verify", "run-smc"):
                    result = call(C0, action, artifact)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
