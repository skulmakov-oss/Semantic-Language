"""Identity rejection must precede compiler execution or evidence creation."""
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest
from reference_identity import qualify_reference


class ReferenceIdentityTests(unittest.TestCase):
    def test_wrong_binary_with_complete_valid_proof_is_rejected(self):
        repo = pathlib.Path(__file__).resolve().parents[2]
        smc = os.environ.get("SEMANTIC_SMC")
        self.assertTrue(smc, "qualification requires SEMANTIC_SMC")
        # Establish that the same proof is valid before changing only the binary.
        qualify_reference(repo, pathlib.Path(smc), pathlib.Path(smc), os.environ)
        with self.assertRaisesRegex(ValueError, "REFERENCE_MISMATCH"):
            qualify_reference(repo, pathlib.Path(smc), pathlib.Path(sys.executable), os.environ)

    def test_non_reference_checkout_is_rejected_before_binary_execution(self):
        repo = pathlib.Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            invalid_metadata = pathlib.Path(directory) / "metadata.json"
            invalid_metadata.write_text("{}", encoding="utf-8")
            env = dict(SEMANTIC_C0_CHECKOUT=str(repo),
                       SEMANTIC_C0_PROVENANCE=str(invalid_metadata),
                       SEMANTIC_C0_ANCHOR=str(invalid_metadata))
            with self.assertRaisesRegex(ValueError, "REFERENCE_MISMATCH.*HEAD"):
                qualify_reference(repo, pathlib.Path(sys.executable), pathlib.Path(sys.executable), env)

    def test_missing_provenance_rejects_an_existing_wrong_binary(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            env = os.environ.copy()
            for key in ("SEMANTIC_C0_CHECKOUT", "SEMANTIC_C0_PROVENANCE", "SEMANTIC_C0_ANCHOR"):
                env.pop(key, None)
            env.update(SEMANTIC_SMC=sys.executable, SEMANTIC_C0_SMC=sys.executable,
                       SEMANTIC_TEST_OUT=str(root / "evidence"))
            result = subprocess.run([sys.executable, str(pathlib.Path(__file__).with_name("run_tests.py"))],
                                    env=env, capture_output=True, text=True, timeout=15)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("REFERENCE_MISMATCH", result.stdout + result.stderr)
            self.assertFalse((root / "evidence").exists())


if __name__ == "__main__":
    unittest.main()
