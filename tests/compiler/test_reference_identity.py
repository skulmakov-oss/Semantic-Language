"""Identity rejection must precede compiler execution or evidence creation."""
import os
import pathlib
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest import mock
from reference_identity import qualify_reference


class ReferenceIdentityTests(unittest.TestCase):
    def qualify_with_cleanup_lock(self, release_delay):
        repo = pathlib.Path(__file__).resolve().parents[2]
        smc = pathlib.Path(os.environ["SEMANTIC_SMC"])
        original = tempfile.TemporaryDirectory.cleanup
        handles, timers, directories = [], [], []

        def lock_once(directory):
            executable = pathlib.Path(directory.name) / "identity.exe"
            if not handles and executable.exists():
                handle = executable.open("rb")
                handles.append(handle)
                directories.append(directory)
                if release_delay is not None:
                    timer = threading.Timer(release_delay, handle.close)
                    timers.append(timer)
                    timer.start()
            return original(directory)

        try:
            with mock.patch.object(tempfile.TemporaryDirectory, "cleanup", lock_once):
                return qualify_reference(repo, smc, smc, os.environ)
        finally:
            self.assertTrue(handles, "did not reach the real fingerprint executable cleanup")
            for timer in timers:
                timer.join()
            for handle in handles:
                handle.close()
            for directory in directories:
                for attempt in range(100):
                    try:
                        original(directory)
                        break
                    except PermissionError:
                        if attempt == 99:
                            raise
                        time.sleep(0.1)

    @unittest.skipUnless(os.name == "nt", "Windows executable sharing contract")
    def test_transient_cleanup_lock_does_not_replace_valid_identity_with_mismatch(self):
        proof = self.qualify_with_cleanup_lock(0.2)
        self.assertEqual(proof["sha"], "1d1e76c7509a03baa0133abaaa5819f5d32582b2")

    @unittest.skipUnless(os.name == "nt", "Windows executable sharing contract")
    def test_persistent_cleanup_lock_still_fails_closed(self):
        with self.assertRaisesRegex(ValueError, "REFERENCE_MISMATCH"):
            self.qualify_with_cleanup_lock(None)

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
