"""A stale generator must not delete or alter approved regression fixtures."""
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest


class CorpusGenerationTests(unittest.TestCase):
    def test_generator_reproduces_committed_corpus_bytes_and_is_deterministic(self):
        here = pathlib.Path(__file__).resolve().parent
        # Git's canonical blob is independent of checkout core.autocrlf policy.
        manifest = subprocess.run(["git", "show", ":tests/compiler/cases.json"],
                                  cwd=here, capture_output=True, check=True)
        expected = {"cases.json": manifest.stdout}
        expected.update({"cases/" + p.name: p.read_bytes() for p in (here / "cases").glob("*.sm")})
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            shutil.copyfile(here / "make_cases.py", root / "make_cases.py")
            for repetition in range(2):
                result = subprocess.run([sys.executable, str(root / "make_cases.py")],
                                        capture_output=True, text=True, timeout=30)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                actual = {"cases.json": (root / "cases.json").read_bytes()}
                actual.update({"cases/" + p.name: p.read_bytes() for p in (root / "cases").glob("*.sm")})
                self.assertEqual(set(actual), set(expected), f"fixture filenames, repetition {repetition}")
                for name in expected:
                    self.assertEqual(actual[name], expected[name], name)


if __name__ == "__main__":
    unittest.main()
