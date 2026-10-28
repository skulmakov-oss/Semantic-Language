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
                b"// leading comment\r\n  Import \"./helper.sm\"\r\n\n"
                b"fn main() {\n    assert(add_one(1) == 2);\n    return;\n}\n"
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

    def test_unterminated_helper_comment_cannot_swallow_parent_declaration(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "helper.sm").write_bytes(b"// helper ends without a newline")
            (root / "main.sm").write_bytes(
                b'Import "helper.sm"\nfn main() { return; }\n'
            )
            result = self.run_compiler(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertTrue(result.stdout.startswith("hex "), result.stdout)

    def test_module_boundaries_cover_lf_and_eof_variants(self):
        cases = {
            "helper_no_lf_parent_lf": (b"fn add_one(value: i32) -> i32 {\n    return value + 1;\n}",
                                       b'Import "helper.sm"\nfn main() { assert(add_one(1) == 2); return; }\n'),
            "helper_no_lf_parent_no_lf": (b"fn add_one(value: i32) -> i32 {\n    return value + 1;\n}",
                                           b'Import "helper.sm"\nfn main() { assert(add_one(1) == 2); return; }'),
            "comment_no_lf_parent_no_lf": (b"// helper comment", b'Import "helper.sm"\nfn main() { return; }'),
            "nested_mixed": (b'Import "leaf.sm"\r\nfn add_one(value: i32) -> i32 {\r\n    return value + 1;\r\n}',
                              b'Import "helper.sm"\r\nfn main() { assert(add_one(1) == 2); return; }'),
        }
        for name, (helper, main) in cases.items():
            with self.subTest(case=name), tempfile.TemporaryDirectory() as directory:
                root = pathlib.Path(directory)
                (root / "helper.sm").write_bytes(helper)
                if name == "nested_mixed":
                    (root / "leaf.sm").write_bytes(b"fn leaf() { return; }\n")
                (root / "main.sm").write_bytes(main)
                result = self.run_compiler(root)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertTrue(result.stdout.startswith("hex "), result.stdout)
                artifact = root / "boundary.smc"
                artifact.write_bytes(bytes.fromhex(result.stdout[4:].strip()))
                for action in ("verify", "run-smc"):
                    checked = subprocess.run(
                        [self.smc, action, artifact], capture_output=True, text=True, timeout=30
                    )
                    self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)

    def test_missing_cycle_and_duplicate_imports_fail_closed(self):
        cases = {
            "missing": {
                "main.sm": b'Import "missing.sm"\nfn main() { return; }\n',
                "marker": "ABI FsRead failure",
            },
            "cycle": {
                "main.sm": b'Import "a.sm"\nfn main() { return; }\n',
                "a.sm": b'Import "b.sm"\n',
                "b.sm": b'Import "a.sm"\n',
                "marker": "cyclic executable helper import detected",
            },
            "duplicate": {
                "main.sm": b'Import "./helper.sm"\nImport "helper.sm"\nfn main() { return; }\n',
                "helper.sm": b'fn value() { return; }\n',
                "marker": "duplicate executable helper import",
            },
            "duplicate_symbols": {
                "main.sm": b'Import "left.sm"\nImport "right.sm"\nfn main() { return; }\n',
                "left.sm": b'fn value() { return; }\n',
                "right.sm": b'fn value() { return; }\n',
                "marker": "defined more than once",
            },
            "root_escape": {
                "main.sm": b'Import "../helper.sm"\nfn main() { return; }\n',
                "marker": "import path escapes the declared source root",
            },
            "unsupported": {
                "main.sm": b'Import "helper.sm" { value }\nfn main() { return; }\n',
                "helper.sm": b'fn value() { return; }\n',
                "marker": "unsupported import form",
            },
        }
        for name, files in cases.items():
            with self.subTest(case=name), tempfile.TemporaryDirectory() as directory:
                root = pathlib.Path(directory)
                marker = files.pop("marker")
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
                self.assertIn(marker, first_text)
                self.assertEqual(list(root.glob("*.smc")), [])


if __name__ == "__main__":
    unittest.main()
