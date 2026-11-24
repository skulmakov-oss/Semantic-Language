"""Binary-free unit tests for tests/selfhost (run: python -m unittest discover -s tests/selfhost -p "test_*.py")."""
import importlib.util
import json
import pathlib
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent.parent
sys.path.insert(0, str(HERE))
import check_results  # noqa: E402
import selfhost  # noqa: E402


class SelfhostToolTests(unittest.TestCase):
    def test_checker_framing_matches_the_contract_validator(self):
        """The checker reimplements shf0-source-set-v1; it must agree with bootstrap/validate_contract.py."""
        v = selfhost.load_validator()
        entries = [(f, (REPO / f).read_bytes()) for f in selfhost.PINS["source_set"]["files"]]
        self.assertEqual(check_results.frame_identity(entries), v.source_set_identity(entries))
        self.assertEqual(v.check_paths(selfhost.PINS["source_set"]["files"], "compiler"), [])

    def test_pins_cover_every_compiler_module_in_sorted_order(self):
        files = selfhost.PINS["source_set"]["files"]
        self.assertEqual(files, sorted(files))
        self.assertEqual(set(files), {"compiler/" + p.name for p in (REPO / "compiler").glob("*.sm")})
        self.assertEqual(selfhost.PINS["c0"]["sha"], "1d1e76c7509a03baa0133abaaa5819f5d32582b2")

    def test_classify_reads_only_the_compiler_output_shape(self):
        self.assertEqual(selfhost.classify(b"hex 00ff\n"), {"kind": "semcode", "semcode": b"\x00\xff"})
        self.assertEqual(selfhost.classify(b"hex 0g\n")["kind"], "malformed_hex")
        self.assertEqual(selfhost.classify(b"hex 0\n")["kind"], "malformed_hex")
        d = selfhost.classify(b"error type @12: cannot compare A with B\n")
        self.assertEqual((d["kind"], d["category"], d["message"]), ("diagnostic", "type", "cannot compare A with B"))
        self.assertEqual(selfhost.classify(b"")["kind"], "no_output")
        self.assertEqual(selfhost.classify(b"panic")["kind"], "other")
        for out in (b"hex 00", b"error x @1: m", b""):
            self.assertEqual(selfhost.classify(out)["kind"], check_results.classify(out)["kind"])

    def test_attempt_directory_is_fresh_and_outside_the_repository(self):
        with tempfile.TemporaryDirectory() as tmp:
            existing = pathlib.Path(tmp)
            with self.assertRaises(SystemExit):
                selfhost.Attempt(existing, create=True)
            fresh = selfhost.Attempt(existing / "a", create=True)
            fresh.create("x.bin", b"1")
            with self.assertRaises(FileExistsError):
                fresh.create("x.bin", b"2")
            with self.assertRaises(SystemExit):
                fresh.path("../escape.bin")
        with self.assertRaises(SystemExit):
            selfhost.Attempt(REPO / "tests" / "selfhost-attempt-must-not-be-created", create=True)
        self.assertFalse((REPO / "tests" / "selfhost-attempt-must-not-be-created").exists())

    def test_checker_fails_closed_on_an_empty_attempt(self):
        with tempfile.TemporaryDirectory() as tmp:
            c = check_results.check(tmp)
            self.assertTrue(c.violations)
            self.assertTrue(any("missing file preflight.json" in v for v in c.violations))

    def test_row_predicate_never_passes_on_exit_code_alone(self):
        ok_semcode = {"kind": "semcode", "semcode": b"\x01", "driver": {"outcome": "executed_ok"}}
        diag = {"kind": "diagnostic", "category": "type", "message": "m", "driver": {"outcome": "executed_ok"}}
        pure_ok = lambda tag, stream: b'{"schema":"semantic.ud1a.stored_exec_provenance/2","outcome":"executed_ok"}\n'  # noqa: E731
        exits = {}
        exit_of = exits.get
        # An expected-ok fixture whose compiler printed a diagnostic fails although every exit code is 0.
        f = check_results.recompute_row("ok", b"\x01", {"C2": diag, "C2.repeat": diag}, ["C2"], {}, exit_of, {})
        self.assertIn("no SemCode", f)
        # SemCode that differs from the C0 compilation fails even if it would verify and run.
        h = check_results.sha(b"\x02")[:16]
        exits.update({f"exec.{h}.verify": 0, f"exec.{h}.run_smc": 0, f"exec.{h}.driver_pure": 0})
        other = dict(ok_semcode, semcode=b"\x02")
        f = check_results.recompute_row("ok", b"\x01", {"C2": other, "C2.repeat": other}, ["C2"], {}, exit_of, {})
        self.assertIn("SemCode differs from C0", f)
        # A nondeterministic repeat fails.
        same = dict(ok_semcode)
        h1 = check_results.sha(b"\x01")[:16]
        exits.update({f"exec.{h1}.verify": 0, f"exec.{h1}.run_smc": 0, f"exec.{h1}.driver_pure": 0})
        f = check_results.recompute_row("ok", b"\x01", {"C2": same, "C2.repeat": other}, ["C2"], {}, exit_of, {})
        self.assertIn("nondeterministic", f)
        self.assertEqual(check_results.recompute_row("ok", b"\x01", {"C2": same, "C2.repeat": same}, ["C2"], {}, exit_of, {}, pure_ok), [])
        # Exit codes 0 everywhere but no proof that the second VM executed the artifact: fails.
        self.assertIn("run failed", check_results.recompute_row("ok", b"\x01", {"C2": same, "C2.repeat": same}, ["C2"], {}, exit_of, {}))
        # A process fault is not a compiler diagnostic, and vice versa.
        fault = {"kind": "no_output", "driver": {"outcome": "runtime_error", "error": "ABI FsRead failure [HostFault]"}}
        self.assertEqual(check_results.recompute_row("fault:ABI FsRead failure", None, {"C2": fault, "C2.repeat": fault}, ["C2"], {}, exit_of, {}), [])
        self.assertTrue(check_results.recompute_row("reject:type:m", None, {"C2": fault, "C2.repeat": fault}, ["C2"], {}, exit_of, {}))
        self.assertTrue(check_results.recompute_row("fault:ABI FsRead failure", None, {"C2": diag, "C2.repeat": diag}, ["C2"], {}, exit_of, {}))


if __name__ == "__main__":
    unittest.main()
