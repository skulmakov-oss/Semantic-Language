"""Executable regression coverage for the bounded direct-local import slice."""
import os
import pathlib
import subprocess
import tempfile
import unittest


class DirectImportTests(unittest.TestCase):
    def setUp(self):
        self.repo = pathlib.Path(__file__).resolve().parents[2]
        self.smc = pathlib.Path(os.environ["SEMANTIC_SMC"])

    def run_compiler(self, root, entry="main.sm"):
        return subprocess.run(
            [self.smc, "run", "main.sm", "--profile", "cli-read-only",
             "--root", root, "--envelope", "trusted-compiler",
             "--max-steps", "20000000", "--max-calls", "50000",
             "--max-frames", "256", "--", entry],
            cwd=self.repo / "compiler", capture_output=True, text=True, timeout=120,
        )

    def test_imports_are_executed_and_module_boundaries_are_safe(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "helper.sm").write_bytes(
                b"fn add_one(value: i32) -> i32 {\n    return value + 1;\n}"
            )
            (root / "main.sm").write_bytes(
                b"// leading comment\r\n  Import \"./helper.sm\"\r\n\r\n"
                b"fn main() {\r\n    assert(add_one(1) == 2);\r\n    return;\r\n}\r\n"
            )
            first = self.run_compiler(root)
            second = self.run_compiler(root)
            self.assertEqual(first.returncode, 0, first.stdout + first.stderr)
            self.assertTrue(first.stdout.startswith("hex "), first.stdout)
            self.assertEqual(first.stdout, second.stdout)
            artifact = root / "imported.smc"
            artifact.write_bytes(bytes.fromhex(first.stdout[4:].strip()))
            for action in ("verify", "run-smc"):
                checked = subprocess.run(
                    [self.smc, action, artifact], capture_output=True, text=True, timeout=30
                )
                self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)

    def test_missing_cycle_and_duplicate_imports_fail_closed(self):
        cases = {
            "missing": {
                "main.sm": b'Import "missing.sm"\nfn main() { return; }\n',
            },
            "cycle": {
                "main.sm": b'Import "a.sm"\nfn main() { return; }\n',
                "a.sm": b'Import "b.sm"\n',
                "b.sm": b'Import "a.sm"\n',
            },
            "duplicate": {
                "main.sm": b'Import "./helper.sm"\nImport "helper.sm"\nfn main() { return; }\n',
                "helper.sm": b'fn value() { return; }\n',
            },
            "duplicate_symbols": {
                "main.sm": b'Import "left.sm"\nImport "right.sm"\nfn main() { return; }\n',
                "left.sm": b'fn value() { return; }\n',
                "right.sm": b'fn value() { return; }\n',
            },
            "root_escape": {
                "main.sm": b'Import "../helper.sm"\nfn main() { return; }\n',
            },
            "unsupported": {
                "main.sm": b'Import "helper.sm" { value }\nfn main() { return; }\n',
                "helper.sm": b'fn value() { return; }\n',
            },
        }
        for name, files in cases.items():
            with self.subTest(case=name), tempfile.TemporaryDirectory() as directory:
                root = pathlib.Path(directory)
                for filename, source in files.items():
                    (root / filename).write_bytes(source)
                first = self.run_compiler(root)
                second = self.run_compiler(root)
                first_text = first.stdout + first.stderr
                second_text = second.stdout + second.stderr
                self.assertTrue(first.returncode != 0 or "error " in first_text,
                                first_text)
                self.assertEqual(first_text, second_text)
                self.assertNotIn("hex ", first.stdout)


if __name__ == "__main__":
    unittest.main()
