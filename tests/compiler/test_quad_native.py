"""Quad-native self-compiler differential qualification (C0 exact binary required)."""
import hashlib
import json
import os
import shutil
from pathlib import Path
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[2]
COMPILER = REPO / "compiler"
C0 = os.environ.get("SEMANTIC_SMC", "")
LUT = ("N", "F", "T", "S")


def build_quad_cases():
    lines = [
        "fn join(a: quad, b: quad) -> quad { return a || b; }",
        "fn meet(a: quad, b: quad) -> quad { return a && b; }",
        "fn invert(a: quad) -> quad { return !a; }",
        "fn classify(value: quad) -> i32 {",
        "    match value {",
        "        N => { return 0; }",
        "        F => { return 1; }",
        "        T => { return 2; }",
        "        S => { return 3; }",
        "        _ => { return 4; }",
        "    }",
        "}",
        "fn main() {",
    ]
    for i, a in enumerate(LUT):
        lines.append(f"    assert(classify({a}) == {i});")
        j = ((i & 1) << 1) | ((i & 2) >> 1)
        lines.append(f"    assert(invert({a}) == {LUT[j]});")
        for k, b in enumerate(LUT):
            lines.append(f"    assert(join({a}, {b}) == {LUT[i | k]});")
            lines.append(f"    assert(meet({a}, {b}) == {LUT[i & k]});")
    lines.extend([
        "    let mut knowledge: quad = N;",
        "    knowledge = knowledge || T;",
        "    assert(knowledge == T);",
        "    knowledge = knowledge || F;",
        "    assert(knowledge == S);",
        "    knowledge = knowledge || N;",
        "    assert(knowledge == S);",
        "    return;",
        "}",
    ])
    return "\n".join(lines) + "\n"


NEGATIVES = {
    "non_bool_condition": "fn main() { let q: quad = N; if q { return; } return; }\n",
    "incomplete_match": "fn main() { let q: quad = N; match q { N => { return; }, T => { return; } } return; }\n",
    "duplicate_quad_arm": "fn main() { let q: quad = T; match q { N => { return; }, N => { return; }, _ => { return; } } return; }\n",
    "mixed_bool_quad": "fn main() { let q: quad = T; let b: bool = true; let x: quad = q || b; return; }\n",
    "duplicate_function": "fn same() { return; }\nfn same() { return; }\nfn main() { return; }\n",
    "duplicate_record": "record Item { x: i32 }\nrecord Item { x: i32 }\nfn main() { return; }\n",
}


def command(args, cwd=COMPILER):
    return subprocess.run([str(x) for x in args], cwd=cwd, capture_output=True,
                          text=True, errors="replace", timeout=180)


@unittest.skipUnless(C0 and Path(C0).is_file(), "SEMANTIC_SMC must be the attested pinned C0")
class QuadNativeTests(unittest.TestCase):
    def run_native(self, root, filename):
        result = command([C0, "run", "main.sm", "--profile", "cli-read-only",
                          "--root", root, "--envelope", "trusted-compiler",
                          "--max-steps", "20000000", "--max-calls", "50000",
                          "--max-frames", "256", "--metrics", "json", "--", filename])
        metrics = [json.loads(line) for line in result.stderr.splitlines()
                   if line.startswith('{"schema":"semantic.run.metrics')]
        self.assertEqual(result.returncode, 0, result.stderr[-1000:])
        self.assertTrue(metrics, result.stderr[-500:])
        self.assertEqual(metrics[-1]["outcome"], "ok", metrics)
        return result.stdout, metrics[-1]

    def test_full_quad_truth_4x4_and_match_vs_c0(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "main.sm").write_text(build_quad_cases(), encoding="utf-8", newline="\n")
            stdout, metrics = self.run_native(root, "main.sm")
            self.assertTrue(stdout.startswith("hex "), stdout[:400])
            native = root / "quad.native.smc"
            native.write_bytes(bytes.fromhex(stdout[4:].strip()))
            oracle = root / "quad.c0.smc"
            c0c = command([C0, "compile", root / "main.sm", "-o", oracle, "--opt-level", "O0"])
            self.assertEqual(c0c.returncode, 0, c0c.stdout + c0c.stderr)
            for artifact in (native, oracle):
                verified = command([C0, "verify", artifact])
                self.assertEqual(verified.returncode, 0, verified.stdout + verified.stderr)
                executed = command([C0, "run-smc", artifact])
                self.assertEqual(executed.returncode, 0, executed.stdout + executed.stderr)
            disasm = command([C0, "disasm", native])
            self.assertIn("LOAD_Q", disasm.stdout)
            self.assertIn("Q_OR", disasm.stdout)
            self.assertIn("Q_AND", disasm.stdout)
            self.assertIn("Q_NOT", disasm.stdout)
            print("quad-native C0/native differential: 16 joins, 16 meets, 4 not, 4 match, provenance",
                  hashlib.sha256(native.read_bytes()).hexdigest(), metrics["counters"])

    def test_semantic_analyzer_return_path_witnesses(self):
        # Link the real Semantic-written Sema implementation, not an imitation
        # of its four-state analysis in Python.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for src in COMPILER.glob("*.sm"):
                if src.name != "main.sm":
                    shutil.copy2(src, root / src.name)
            fixture = REPO / "tests/compiler/programs/quad_flow_internal.sm"
            (root / "main.sm").write_bytes(fixture.read_bytes())
            artifact = root / "flow_check.smc"
            built = command([C0, "compile", root / "main.sm", "-o", artifact])
            self.assertEqual(built.returncode, 0, built.stdout + built.stderr)
            for action in ("verify", "run-smc"):
                checked = command([C0, action, artifact])
                self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)
            disasm = command([C0, "disasm", artifact])
            self.assertIn("Q_OR", disasm.stdout)

    def test_type_constraint_evidence_from_real_sema(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for src in COMPILER.glob("*.sm"):
                if src.name != "main.sm":
                    shutil.copy2(src, root / src.name)
            fixture = REPO / "tests/compiler/programs/quad_type_constraint_internal.sm"
            (root / "main.sm").write_bytes(fixture.read_bytes())
            artifact = root / "type_evidence.smc"
            built = command([C0, "compile", root / "main.sm", "-o", artifact])
            self.assertEqual(built.returncode, 0, built.stdout + built.stderr)
            for action in ("verify", "run-smc"):
                checked = command([C0, action, artifact])
                self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)
            disasm = command([C0, "disasm", artifact])
            self.assertIn("Q_OR", disasm.stdout)

    def test_q2_if_expression_legacy_diagnostics_and_execution(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            good = "fn main() { let x: i32 = if true { 1 } else { 2 }; assert(x == 1); return; }\n"
            bad = "fn main() { let x: i32 = if true { 1 } else { true }; return; }\n"
            for name, source in (("valid", good), ("mismatch", bad)):
                with self.subTest(name=name):
                    (root / "main.sm").write_text(source, encoding="utf-8", newline="\n")
                    stdout, _ = self.run_native(root, "main.sm")
                    oracle_art = root / (name + ".c0.smc")
                    oracle = command([C0, "compile", root / "main.sm", "-o", oracle_art])
                    if name == "valid":
                        self.assertTrue(stdout.startswith("hex "), stdout[:300])
                        self.assertEqual(oracle.returncode, 0, oracle.stderr)
                        native = root / (name + ".native.smc")
                        native.write_bytes(bytes.fromhex(stdout[4:].strip()))
                        for artifact in (native, oracle_art):
                            for action in ("verify", "run-smc"):
                                run = command([C0, action, artifact])
                                self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
                    else:
                        self.assertTrue(stdout.startswith("error type "), stdout[:300])
                        self.assertIn("branches have different types", stdout)
                        self.assertNotEqual(oracle.returncode, 0)
                        self.assertIn("if expression branch type mismatch", oracle.stdout + oracle.stderr)

    def _run_real_semantic_fixture(self, name, max_steps="1000000"):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for src in COMPILER.glob("*.sm"):
                if src.name != "main.sm":
                    shutil.copy2(src, root / src.name)
            source = REPO / "tests/compiler/programs" / name
            (root / "main.sm").write_bytes(source.read_bytes())
            artifact = root / "phase_proof.smc"
            built = command([C0, "compile", root / "main.sm", "-o", artifact])
            self.assertEqual(built.returncode, 0, built.stdout + built.stderr)
            verified = command([C0, "verify", artifact])
            self.assertEqual(verified.returncode, 0, verified.stdout + verified.stderr)
            # Internal analyzer/provenance fixtures now exceed the run-smc
            # utility's fixed 100k-step cap. Keep an explicit, bounded
            # research budget; do not claim normative Bootstrap qualification.
            executed = command([C0, "run", "main.sm", "--profile", "cli-read-only",
                                "--root", root, "--envelope", "trusted-compiler",
                                "--max-steps", max_steps, "--max-calls", "50000",
                                "--max-frames", "256", "--", "main.sm"], cwd=root)
            self.assertEqual(executed.returncode, 0, executed.stdout + executed.stderr)

    def test_q2_call_and_record_type_evidence(self):
        self._run_real_semantic_fixture("quad_type_coverage_internal.sm")

    def test_q3_q4_lowering_admission_and_provenance(self):
        self._run_real_semantic_fixture("quad_lowering_provenance_internal.sm")

    def test_q2_q3_native_vs_c0_calls_and_records(self):
        cases = (
            ("good", "record Pair { left: i32, right: i32 } fn add(a: i32, b: i32) -> i32 { return a + b; } fn main() { let x: i32 = add(1, 2); let p: Pair = Pair { left: 1, right: 2 }; let z: i32 = if true { x } else { p.left }; assert(z == 3); return; }\n"),
            ("bad_call", "fn add(a: i32, b: i32) -> i32 { return a + b; } fn main() { let x: i32 = add(1, true); return; }\n"),
            ("bad_record", "record Pair { left: i32, right: i32 } fn main() { let p: Pair = Pair { left: 1, right: true }; return; }\n"),
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for case, source in cases:
                with self.subTest(case=case):
                    (root / "main.sm").write_text(source, encoding="utf-8", newline="\n")
                    stdout, _ = self.run_native(root, "main.sm")
                    reference = root / (case + ".c0.smc")
                    oracle = command([C0, "compile", root / "main.sm", "-o", reference])
                    if case == "good":
                        self.assertTrue(stdout.startswith("hex "), stdout[:300])
                        self.assertEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)
                        native = root / (case + ".native.smc")
                        native.write_bytes(bytes.fromhex(stdout[4:].strip()))
                        for artifact in (native, reference):
                            for action in ("verify", "run-smc"):
                                outcome = command([C0, action, artifact])
                                self.assertEqual(outcome.returncode, 0, outcome.stdout + outcome.stderr)
                    else:
                        self.assertTrue(stdout.startswith("error type "), stdout[:300])
                        self.assertNotEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)

    def test_q2_q3_q4_binding_assignment_return_evidence(self):
        self._run_real_semantic_fixture("quad_binding_return_internal.sm")

    def test_q2_binding_return_native_c0_admission_parity(self):
        cases = (
            ("annotated_ok", "fn main() { let x: i32 = 1; assert(x == 1); return; }\n", True),
            ("annotated_bad", "fn main() { let x: i32 = true; return; }\n", False),
            ("inferred_ok", "fn main() { let x = 1; assert(x == 1); return; }\n", True),
            ("assignment_ok", "fn main() { let mut x: i32 = 1; x = 2; assert(x == 2); return; }\n", True),
            ("assignment_bad", "fn main() { let mut x: i32 = 1; x = true; return; }\n", False),
            ("return_ok", "fn f() -> i32 { return 3; } fn main() { assert(f() == 3); return; }\n", True),
            ("return_bad", "fn f() -> i32 { return true; } fn main() { return; }\n", False),
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, src, should_pass in cases:
                with self.subTest(name=name):
                    (root / "main.sm").write_text(src, encoding="utf-8", newline="\n")
                    stdout, _ = self.run_native(root, "main.sm")
                    oracle_art = root / (name + ".c0.smc")
                    oracle = command([C0, "compile", root / "main.sm", "-o", oracle_art])
                    if should_pass:
                        self.assertTrue(stdout.startswith("hex "), stdout[:300])
                        self.assertEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)
                        artifact = root / (name + ".native.smc")
                        artifact.write_bytes(bytes.fromhex(stdout[4:].strip()))
                        for candidate in (artifact, oracle_art):
                            for action in ("verify", "run-smc"):
                                result = command([C0, action, candidate])
                                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    else:
                        self.assertTrue(stdout.startswith("error type "), stdout[:250])
                        self.assertNotEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)

    def test_q2_q3_q4_tuple_sequence_match_provenance(self):
        self._run_real_semantic_fixture("quad_container_match_internal.sm")

    def test_q24_tuple_sequence_match_c0_admission_parity(self):
        cases = (
            ("tuple_t", "fn main() { let t: (i32, bool) = (1, true); return; }", True),
            ("tuple_s", "fn main() { let t: (i32, bool) = (1, 2); return; }", False),
            ("tuple_infer", "fn main() { let t = (1, true); return; }", True),
            ("seq_t", "fn main() { let xs: Sequence(i32) = [1, 2, 3]; assert(xs[1] == 2); return; }", True),
            ("seq_s", "fn main() { let xs: Sequence(i32) = [1, true]; return; }", False),
            ("seq_f", "fn main() { let xs: Sequence(i32) = [true]; return; }", False),
            ("seq_empty", "fn main() { let xs: Sequence(i32) = []; assert(len(xs) == 0); return; }", True),
            ("match_t", "fn main() { let x: i32 = match 1 { 1 => { 10 } _ => { 20 } }; assert(x == 10); return; }", True),
            ("match_s", "fn main() { let x: i32 = match 1 { 1 => { 10 } _ => { true } }; return; }", False),
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, source, should_pass in cases:
                with self.subTest(name=name):
                    (root / "main.sm").write_text(source + "\n", encoding="utf-8", newline="\n")
                    stdout, metrics = self.run_native(root, "main.sm")
                    oracle_art = root / (name + ".c0.smc")
                    oracle = command([C0, "compile", root / "main.sm", "-o", oracle_art])
                    if should_pass:
                        self.assertTrue(stdout.startswith("hex "), stdout[:350])
                        self.assertEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)
                        native = root / (name + ".native.smc")
                        native.write_bytes(bytes.fromhex(stdout[4:].strip()))
                        for artifact in (native, oracle_art):
                            for action in ("verify", "run-smc"):
                                checked = command([C0, action, artifact])
                                self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)
                    else:
                        self.assertTrue(stdout.startswith("error type "), stdout[:350])
                        self.assertNotEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)

    def test_q25_operator_and_builtin_real_sema_lowering(self):
        self._run_real_semantic_fixture("quad_builtin_operator_internal.sm")

    def test_q25_operators_and_nonmap_builtins_c0_parity(self):
        cases = (
            ("binary_arith_ok", "fn main() { let x: i32 = 2 + 3; assert(x == 5); return; }", True),
            ("binary_arith_mixed", "fn main() { let x = 2 + true; return; }", False),
            ("binary_arith_bool", "fn main() { let x = true + false; return; }", False),
            ("unary_i32", "fn main() { let x: i32 = -2; assert(x == -2); return; }", True),
            ("unary_bad", "fn main() { let x = -true; return; }", False),
            ("unary_bool", "fn main() { let x: bool = !true; assert(!x); return; }", True),
            ("eq_i32", "fn main() { assert(3 == 3); return; }", True),
            ("ordered_u32", "fn main() { assert(3u32 > 2u32); return; }", True),
            ("quad_join", "fn main() { let x: quad = T || F; assert(x == S); return; }", True),
            ("len_seq", "fn main() { let x: Sequence(i32) = [1, 2]; assert(len(x) == 2); return; }", True),
            ("len_bad", "fn main() { let n = len(1); return; }", False),
            ("push_ok", "fn main() { let x: Sequence(i32) = [1]; let y: Sequence(i32) = push(x, 2); assert(len(y) == 2); return; }", True),
            ("push_bad", "fn main() { let x: Sequence(i32) = [1]; let y = push(x, true); return; }", False),
            ("to_text", "fn main() { let t: text = to_text(23); assert(text_len(t) == 2u32); return; }", True),
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, source, should_pass in cases:
                with self.subTest(name=name):
                    (root / "main.sm").write_text(source + "\n", encoding="utf-8", newline="\n")
                    stdout, _ = self.run_native(root, "main.sm")
                    ref = root / (name + ".c0.smc")
                    c0 = command([C0, "compile", root / "main.sm", "-o", ref])
                    if should_pass:
                        self.assertTrue(stdout.startswith("hex "), stdout[:300])
                        self.assertEqual(c0.returncode, 0, c0.stdout + c0.stderr)
                        dev = root / (name + ".native.smc")
                        dev.write_bytes(bytes.fromhex(stdout[4:].strip()))
                        for artifact in (dev, ref):
                            for action in ("verify", "run-smc"):
                                result = command([C0, action, artifact])
                                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    else:
                        self.assertTrue(stdout.startswith("error type "), stdout[:300])
                        self.assertNotEqual(c0.returncode, 0, c0.stdout + c0.stderr)

    def test_q26_map_dependent_evidence_and_lowering(self):
        # The actual Sema->Lower fixture exercises N/F/T, exact map proof
        # identity/source, missing key/value and deleted callee metadata.
        self._run_real_semantic_fixture("quad_map_dependency_internal.sm",
                                        max_steps="2000000")

    def test_q26_map_legacy_c0_admission_and_execution(self):
        names = (
            "m01_map_i32", "m02_map_text", "m03_map_bool",
            "m04_map_u32", "m05_map_signature", "m06_map_overwrite",
            "m07_wrong_key", "m08_wrong_value", "m09_contains_arity",
            "m10_empty_arity", "m11_wrong_default", "m12_missing_context",
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for index, name in enumerate(names):
                with self.subTest(name=name):
                    path = REPO / "tests/compiler/cases" / (name + ".sm")
                    (root / "main.sm").write_bytes(path.read_bytes())
                    stdout, _ = self.run_native(root, "main.sm")
                    oracle_art = root / (name + ".c0.smc")
                    oracle = command([C0, "compile", root / "main.sm", "-o", oracle_art])
                    if index < 6:
                        self.assertTrue(stdout.startswith("hex "), stdout[:350])
                        self.assertEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)
                        native = root / (name + ".native.smc")
                        native.write_bytes(bytes.fromhex(stdout[4:].strip()))
                        for artifact in (native, oracle_art):
                            for action in ("verify", "run-smc"):
                                checked = command([C0, action, artifact])
                                self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)
                    else:
                        self.assertTrue(stdout.startswith("error "), stdout[:350])
                        self.assertNotEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)

    def test_q27_tuple_shape_assignment_and_inference_origins(self):
        self._run_real_semantic_fixture("quad_tuple_inference_internal.sm",
                                        max_steps="2500000")

    def test_q27_legacy_tuple_c0_parity(self):
        cases = (
            ("a03_tuples_and_destructuring", True),
            ("r15_tuple_statement", True),
            ("r17_tuple_assignment_arity", False),
            ("r18_tuple_assignment_type", False),
            ("r19_tuple_assignment_unknown", False),
            ("r20_tuple_assignment_target", False),
            ("r25_tuple_annotation_type", False),
            ("r26_tuple_annotation_arity", False),
            ("r31_tuple_binding_in_value_block", True),
            ("r34_tuple_expression_call", True),
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, should_pass in cases:
                with self.subTest(name=name):
                    origin = REPO / "tests/compiler/cases" / (name + ".sm")
                    (root / "main.sm").write_bytes(origin.read_bytes())
                    stdout, _ = self.run_native(root, "main.sm")
                    ref = root / (name + ".reference.smc")
                    oracle = command([C0, "compile", root / "main.sm", "-o", ref])
                    if should_pass:
                        self.assertTrue(stdout.startswith("hex "), stdout[:350])
                        self.assertEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)
                        native = root / (name + ".native.smc")
                        native.write_bytes(bytes.fromhex(stdout[4:].strip()))
                        for compiled in (native, ref):
                            for action in ("verify", "run-smc"):
                                checked = command([C0, action, compiled])
                                if name == "r34_tuple_expression_call" and action == "run-smc":
                                    self.assertNotEqual(checked.returncode, 0, checked.stdout + checked.stderr)
                                    self.assertIn("assertion failed", checked.stdout + checked.stderr)
                                else:
                                    self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)
                    else:
                        self.assertTrue(stdout.startswith("error "), stdout[:350])
                        self.assertNotEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)

    def test_q28_control_conditions_inference_and_provenance(self):
        self._run_real_semantic_fixture("quad_control_inference_internal.sm",
                                        max_steps="2200000")

    def test_q28_control_and_inference_c0_parity(self):
        cases = (
            ("if_bool", "fn main() { if true { assert(true); } return; }", True),
            ("if_int", "fn main() { if 1 { return; } return; }", False),
            ("while_bool", "fn main() { while false { break; } return; }", True),
            ("while_int", "fn main() { while 1 { break; } return; }", False),
            ("assert_bool", "fn main() { assert(true); return; }", True),
            ("assert_int", "fn main() { assert(1); return; }", False),
            ("for_sequence", "fn main() { for x in [1, 2] { assert(x > 0); } return; }", True),
            ("for_nonsequence", "fn main() { for x in 1 { assert(true); } return; }", False),
            ("inferred_bool", "fn main() { let v = true; assert(v); return; }", True),
            ("inferred_text", 'fn main() { let v = "hello"; assert(text_len(v) == 5u32); return; }', True),
            ("inferred_sequence", "fn main() { let v = [1, 2]; assert(len(v) == 2); return; }", True),
            ("inferred_call", "fn g() -> i32 { return 5; } fn main() { let v = g(); assert(v == 5); return; }", True),
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, source, should_pass in cases:
                with self.subTest(name=name):
                    (root / "main.sm").write_text(source + "\n", encoding="utf-8", newline="\n")
                    stdout, _ = self.run_native(root, "main.sm")
                    reference = root / (name + ".c0.smc")
                    oracle = command([C0, "compile", root / "main.sm", "-o", reference])
                    if should_pass:
                        self.assertTrue(stdout.startswith("hex "), stdout[:400])
                        self.assertEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)
                        native = root / (name + ".native.smc")
                        native.write_bytes(bytes.fromhex(stdout[4:].strip()))
                        for artifact in (native, reference):
                            for action in ("verify", "run-smc"):
                                checked = command([C0, action, artifact])
                                self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)
                    else:
                        self.assertTrue(stdout.startswith("error type "), stdout[:400])
                        self.assertNotEqual(oracle.returncode, 0, oracle.stdout + oracle.stderr)

    def test_q39_canonical_sema_replay_fail_closed(self):
        self._run_real_semantic_fixture("quad_trusted_admission_internal.sm",
                                        max_steps="2500000")

    def test_q39_compile_source_replay_c0_admission(self):
        cases = (
            ("simple", "fn main() { let v = 1; assert(v == 1); return; }", True),
            ("map", "fn main() { let m: Map(i32, i32) = map_empty(); let n: Map(i32, i32) = map_set(m, 1, 3); assert(map_get(n, 1, 0) == 3); return; }", True),
            ("tuple", "fn main() { let (a, b) = (1, true); assert(a == 1 && b); return; }", True),
            ("condition", "fn main() { let v: i32 = if true { 1 } else { 2 }; assert(v == 1); return; }", True),
            ("forin", "fn main() { for x in [1, 2] { assert(x > 0); } return; }", True),
            ("bad_operator", "fn main() { let x = true + false; return; }", False),
            ("bad_if", "fn main() { if 1 { return; } return; }", False),
            ("bad_map", "fn main() { let m: Map(i32, i32) = map_empty(); let n = map_set(m, true, 3); return; }", False),
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, source, accepted in cases:
                with self.subTest(name=name):
                    (root / "main.sm").write_text(source + "\n", encoding="utf-8", newline="\n")
                    stdout, _ = self.run_native(root, "main.sm")
                    artifact = root / (name + ".frozen.smc")
                    reference = command([C0, "compile", root / "main.sm", "-o", artifact])
                    if accepted:
                        self.assertTrue(stdout.startswith("hex "), stdout[:500])
                        self.assertEqual(reference.returncode, 0, reference.stdout + reference.stderr)
                        native = root / (name + ".native.smc")
                        native.write_bytes(bytes.fromhex(stdout[4:].strip()))
                        for output in (native, artifact):
                            for action in ("verify", "run-smc"):
                                checked = command([C0, action, output])
                                self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)
                    else:
                        self.assertTrue(stdout.startswith("error type "), stdout[:500])
                        self.assertNotEqual(reference.returncode, 0, reference.stdout + reference.stderr)

    def test_q49_source_file_proof_chain_tamper(self):
        self._run_real_semantic_fixture("quad_source_provenance_internal.sm",
                                        max_steps="4500000")

    def test_q49_nested_import_source_identity_parity(self):
        cases = (
            ("direct", {
                "helper.sm": b"fn inc(x: i32) -> i32 { return x + 1; }",
                "main.sm": b'Import "helper.sm"\nfn main() { assert(inc(1) == 2); return; }',
            }),
            ("nested", {
                "leaf.sm": b"fn inc(x: i32) -> i32 { return x + 1; }\r\n",
                "helper.sm": b'Import "leaf.sm"\r\nfn twice(x: i32) -> i32 { return inc(inc(x)); }',
                "main.sm": b'Import "helper.sm"\r\nfn main() { assert(twice(1) == 3); return; }',
            }),
            ("comment_boundary", {
                "helper.sm": b"// helper without newline",
                "main.sm": b'Import "helper.sm"\nfn main() { return; }',
            }),
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, sources in cases:
                with self.subTest(case=name):
                    for filename, payload in sources.items():
                        (root / filename).write_bytes(payload)
                    output, _ = self.run_native(root, "main.sm")
                    second, _ = self.run_native(root, "main.sm")
                    self.assertTrue(output.startswith("hex "), output[:350])
                    self.assertEqual(output, second)
                    artifact = root / (name + ".smc")
                    artifact.write_bytes(bytes.fromhex(output[4:].strip()))
                    for command_name in ("verify", "run-smc"):
                        check = command([C0, command_name, artifact])
                        self.assertEqual(check.returncode, 0, check.stdout + check.stderr)

    def test_terminal_unit_match_has_valid_jump_target(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "main.sm").write_text(
                "fn main() { match 1 { 1 => { return; } _ => { return; } } }\n",
                encoding="utf-8", newline="\n")
            stdout, _ = self.run_native(root, "main.sm")
            self.assertTrue(stdout.startswith("hex "), stdout[:200])
            artifact = root / "terminal_match.smc"
            artifact.write_bytes(bytes.fromhex(stdout[4:].strip()))
            for action in ("verify", "run-smc"):
                checked = command([C0, action, artifact])
                self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)

    def test_negative_boundaries(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, src in NEGATIVES.items():
                with self.subTest(name=name):
                    (root / "main.sm").write_text(src, encoding="utf-8", newline="\n")
                    stdout, metrics = self.run_native(root, "main.sm")
                    self.assertFalse(stdout.startswith("hex "), f"unexpected admission: {name}")
                    self.assertTrue(stdout.startswith("error "), stdout[:200])
                    self.assertIsNone(metrics.get("quota_exceeded"))


if __name__ == "__main__":
    unittest.main()
