"""Control flow after a guaranteed return must not produce dangling labels."""
import os
import pathlib
import subprocess
import tempfile
import unittest
from reference_identity import qualify_reference


class UnreachableControlTests(unittest.TestCase):
    def test_unreachable_control_flow_still_emits_admitted_artifacts(self):
        repo = pathlib.Path(__file__).resolve().parents[2]
        smc = pathlib.Path(os.environ["SEMANTIC_SMC"])
        qualify_reference(repo, smc, smc, os.environ)
        sources = (
            "fn main() { return; if true {} }",
            "fn main() { return; while false {} }",
            "fn main() { if true { return; } else { return; } if false {} }",
            "fn main() { if true { return; if false {} } return; }",
        )
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            for index, source in enumerate(sources):
                with self.subTest(case=index):
                    path = root / f"case{index}.sm"
                    path.write_bytes(source.encode("utf-8"))
                    command = [smc, "run", "main.sm", "--profile", "cli-read-only", "--root", root,
                               "--envelope", "trusted-compiler", "--max-steps", "20000000",
                               "--max-calls", "50000", "--max-frames", "256", "--", path.name]
                    result = subprocess.run(command, cwd=repo / "compiler", capture_output=True, text=True, timeout=120)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    self.assertTrue(result.stdout.startswith("hex "), result.stdout)
                    artifact = root / "native.smc"
                    artifact.write_bytes(bytes.fromhex(result.stdout[4:].strip()))
                    for action in ("verify", "run-smc"):
                        checked = subprocess.run([smc, action, artifact], capture_output=True, text=True, timeout=30)
                        self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)
                    for opt in ("O0", "O1"):
                        reference = root / f"reference.{opt}.smc"
                        compiled = subprocess.run([smc, "compile", path, "-o", reference, "--opt-level", opt],
                                                  capture_output=True, text=True, timeout=30)
                        self.assertEqual(compiled.returncode, 0, compiled.stdout + compiled.stderr)
                        for action in ("verify", "run-smc"):
                            checked = subprocess.run([smc, action, reference], capture_output=True, text=True, timeout=30)
                            self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)


if __name__ == "__main__":
    unittest.main()
