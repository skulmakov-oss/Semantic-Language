"""Structural equality contract of the Semantic compiler, checked under pinned C0.

Equality is admitted for scalars and for acyclic Sequence, tuple, record and Option carriers
whose nested elements are comparable (compiler/sema.sm supports_eq; admitted on main by #61).
Accepted programs are verified and executed; rejections are checked by exact diagnostic.
"""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[2]
COMPILER = REPO / "compiler"
C0 = os.environ.get("SEMANTIC_SMC", "")
PAIR = "record Pair { a: i32, b: i32 }\n"
NESTED = "record In { t: text, f: bool }\nrecord Out { i: In, n: u32 }\n"
P = "record P { a: i32 }\n"


def call(*args, cwd=COMPILER):
    return subprocess.run([str(x) for x in args], cwd=cwd, capture_output=True,
                          text=True, errors="replace", timeout=180)


def main(body):
    return f"fn main() {{\n    {body}\n    return;\n}}\n"


@unittest.skipUnless(C0 and Path(C0).is_file(), "SEMANTIC_SMC must identify pinned C0")
class StructuralEqualityTests(unittest.TestCase):
    def compile_native(self, root, source):
        (root / "main.sm").write_text(source, encoding="utf-8", newline="\n")
        result = call(C0, "run", "main.sm", "--profile", "cli-read-only",
                      "--root", root, "--envelope", "trusted-compiler",
                      "--max-steps", "20000000", "--max-calls", "50000",
                      "--max-frames", "256", "--", "main.sm")
        self.assertEqual(result.returncode, 0, result.stdout[-200:] + result.stderr[-1500:])
        return result.stdout.strip()

    def artifact(self, root, source):
        line = self.compile_native(root, source)
        self.assertTrue(line.startswith("hex "), line[:250])
        path = root / "equality.smc"
        path.write_bytes(bytes.fromhex(line[4:]))
        verified = call(C0, "verify", path)
        self.assertEqual(verified.returncode, 0, verified.stdout + verified.stderr)
        return path

    def test_equal_and_unequal_values_execute(self):
        for name, source in (
            ("record_equal", PAIR + main("let x = Pair { a: 1, b: 2 }; let y = Pair { a: 1, b: 2 }; "
                                         "assert(x == y); assert(!(x != y));")),
            ("record_unequal", PAIR + main("let x = Pair { a: 1, b: 2 }; let y = Pair { a: 1, b: 3 }; "
                                           "assert(x != y); assert(!(x == y));")),
            ("nested_record", NESTED + main(
                'let x = Out { i: In { t: "a", f: true }, n: 1u32 }; '
                'let y = Out { i: In { t: "a", f: true }, n: 1u32 }; '
                'let z = Out { i: In { t: "b", f: true }, n: 1u32 }; assert(x == y); assert(x != z);')),
            ("option_of_record", P + main(
                "let x: Option(P) = Option::Some(P { a: 1 }); let y: Option(P) = Option::Some(P { a: 1 }); "
                "let n: Option(P) = Option::None; assert(x == y); assert(x != n);")),
            ("sequence_of_record", P + main(
                "let x: Sequence(P) = [P { a: 1 }, P { a: 2 }]; let y: Sequence(P) = [P { a: 1 }, P { a: 2 }]; "
                "let z: Sequence(P) = [P { a: 1 }]; assert(x == y); assert(x != z);")),
            ("tuple", main("let x: (i32, bool) = (1, true); let y: (i32, bool) = (1, true); "
                           "let z: (i32, bool) = (2, true); assert(x == y); assert(x != z);")),
        ):
            with self.subTest(case=name), tempfile.TemporaryDirectory() as tmp:
                executed = call(C0, "run-smc", self.artifact(Path(tmp), source))
                self.assertEqual(executed.returncode, 0, executed.stdout + executed.stderr)

    def test_false_comparison_reaches_the_assertion(self):
        # Guards against an equality that is admitted but always evaluates to true.
        source = PAIR + main("let x = Pair { a: 1, b: 2 }; let y = Pair { a: 1, b: 3 }; assert(x == y);")
        with tempfile.TemporaryDirectory() as tmp:
            executed = call(C0, "run-smc", self.artifact(Path(tmp), source))
            self.assertNotEqual(executed.returncode, 0)
            self.assertIn("assertion failed", executed.stdout + executed.stderr)

    def test_incomparable_operands_are_rejected(self):
        map_field = "record M { m: Map(i32, i32) }\n" + main(
            "let e: Map(i32, i32) = map_empty(); let x = M { m: e }; let y = M { m: e }; assert(x == y);")
        map_value = main("let x: Map(i32, i32) = map_empty(); let y: Map(i32, i32) = map_empty(); assert(x == y);")
        mismatch = "record A { a: i32 }\nrecord B { a: i32 }\n" + main(
            "let x = A { a: 1 }; let y = B { a: 1 }; assert(x == y);")
        ordering = P + main("let x = P { a: 1 }; let y = P { a: 2 }; assert(x < y);")
        for name, source, operator, message in (
            ("record_with_map_field", map_field, "== y",
             "error unsupported @{}: equality on M is outside the supported subset"),
            ("map", map_value, "== y",
             "error unsupported @{}: equality on Map(i32, i32) is outside the supported subset"),
            ("different_records", mismatch, "== y", "error type @{}: cannot compare A with B"),
            ("record_ordering", ordering, "< y",
             "error type @{}: ordering expects i32 or u32 operands of one family, got P and P"),
        ):
            with self.subTest(case=name), tempfile.TemporaryDirectory() as tmp:
                self.assertEqual(self.compile_native(Path(tmp), source),
                                 message.format(source.rindex(operator)))


if __name__ == "__main__":
    unittest.main()
