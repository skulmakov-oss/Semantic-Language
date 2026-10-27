import unittest

from reference_outcomes import outcome_key, reference_matches, reference_pair_matches


class ReferenceOutcomeTests(unittest.TestCase):
    def test_subset_rejection_does_not_waive_reference_admission(self):
        rejected = dict(kind="rejected_by_verifier", compile_rc=0, verify_rc=1, detail="invalid ownership metadata")
        self.assertFalse(reference_matches("ok", rejected))
        self.assertFalse(reference_pair_matches("ok", rejected, rejected))

    def test_internal_failure_is_not_expected_source_rejection(self):
        failed = dict(kind="compile_error", compile_rc=1, artifact_exists=False,
                      detail="internal compiler error while reporting unknown function 'nope'")
        self.assertFalse(reference_matches("reject:unknown function 'nope'", failed))

    def test_expected_rejection_requires_exact_exit_and_absent_artifact(self):
        rejected = dict(kind="compile_error", compile_rc=1, artifact_exists=False, detail="at 3: unknown function 'nope'")
        self.assertTrue(reference_matches("reject:unknown function 'nope'", rejected))
        self.assertFalse(reference_matches("reject:unknown function 'nope'", {**rejected, "compile_rc": 2}))
        self.assertFalse(reference_matches("reject:unknown function 'nope'", {**rejected, "artifact_exists": True}))
        self.assertFalse(reference_matches("reject:wrong argument type", rejected))

    def test_success_and_trap_require_admission_and_matching_execution(self):
        admitted = dict(kind="ok", compile_rc=0, verify_rc=0, run_rc=0, detail="")
        trapped = dict(kind="trap", compile_rc=0, verify_rc=0, run_rc=1, detail="runtime trap: DivisionByZero")
        self.assertTrue(reference_matches("ok", admitted))
        self.assertTrue(reference_matches("trap:DivisionByZero", trapped))
        self.assertFalse(reference_matches("ok", trapped))
        self.assertFalse(reference_matches("trap:ArithmeticOverflow", trapped))
        self.assertFalse(reference_matches("ok", {**admitted, "verify_rc": 1}))

    def test_verifier_failure_is_not_an_admitted_execution_outcome(self):
        with self.assertRaises(ValueError):
            outcome_key(dict(kind="rejected_by_verifier", detail="invalid ownership metadata"))

    def test_compile_failure_cannot_establish_execution_parity(self):
        with self.assertRaises(ValueError):
            outcome_key(dict(kind="compile_error", detail="internal compiler error"))


if __name__ == "__main__":
    unittest.main()
