"""PR #46 P2 follow-up regressions: interspersed imports and evidence scaling.

Every accepted result is emitted by the real Semantic compiler under pinned C0.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[2]
COMPILER = REPO / "compiler"
C0 = os.environ.get("SEMANTIC_SMC", "")


def call(*args, cwd=COMPILER):
    return subprocess.run([str(x) for x in args], cwd=cwd, capture_output=True,
                          text=True, errors="replace", timeout=180)


@unittest.skipUnless(C0 and Path(C0).is_file(), "SEMANTIC_SMC must identify pinned C0")
class PR46FollowupTests(unittest.TestCase):
    def compile_native(self, root, main="main.sm"):
        result = call(C0, "run", "main.sm", "--profile", "cli-read-only",
                      "--root", root, "--envelope", "trusted-compiler",
                      "--max-steps", "20000000", "--max-calls", "50000",
                      "--max-frames", "256", "--metrics", "json", "--", main)
        metrics = [json.loads(line) for line in result.stderr.splitlines()
                   if line.startswith('{"schema":"semantic.run.metrics')]
        self.assertEqual(result.returncode, 0, result.stdout[-200:] + result.stderr[-1500:])
        self.assertTrue(result.stdout.startswith("hex "), result.stdout[:250])
        self.assertTrue(metrics, result.stderr[-500:])
        self.assertEqual(metrics[-1]["outcome"], "ok", metrics)
        self.assertIsNone(metrics[-1]["quota_exceeded"], metrics)
        return bytes.fromhex(result.stdout[4:].strip()), metrics[-1]

    def check_artifact(self, root, data):
        artifact = root / "pr46-followup.smc"
        artifact.write_bytes(data)
        for action in ("verify", "run-smc"):
            result = call(C0, action, artifact)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_interspersed_imports_preserve_source_lineage(self):
        cases = {
            "after_declaration": {
                "main.sm": b'fn before() { return; }\nImport "helper.sm"\nfn main() { before(); after(); return; }\n',
                "helper.sm": b'fn after() { return; }\n',
            },
            "between_two_imports": {
                "main.sm": b'Import "first.sm"\nfn between() { return; }\nImport "second.sm"\nfn main() { first(); between(); second(); return; }\n',
                "first.sm": b'fn first() { return; }\n',
                "second.sm": b'fn second() { return; }\n',
            },
            "nested_mixed_crlf": {
                "main.sm": b'fn before() { return; }\r\nImport "helper.sm"\r\nfn main() { before(); from_helper(); from_leaf(); return; }\r\n',
                "helper.sm": b'fn from_helper() { return; }\r\nImport "leaf.sm"\r\n',
                "leaf.sm": b'fn from_leaf() { return; }\r\n',
            },
        }
        for name, files in cases.items():
            with self.subTest(case=name), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                for filename, content in files.items():
                    (root / filename).write_bytes(content)
                artifact, _ = self.compile_native(root)
                self.check_artifact(root, artifact)

    def test_large_sequence_does_not_exhaust_steps(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            elements = ", ".join("1" for _ in range(1000))
            source = (f"fn main() {{ let items: Sequence(i32) = [{elements}]; "
                      "assert(len(items) == 1000); return; }\n")
            (root / "main.sm").write_text(source, encoding="utf-8", newline="\n")
            artifact, metrics = self.compile_native(root)
            self.assertLess(metrics["counters"]["steps"], 20000000)
            self.check_artifact(root, artifact)
            print("PR46 witnesses 1000: ", metrics["counters"])

    def test_duplicate_witness_rejected_in_incremental_accumulator(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for src in COMPILER.glob("*.sm"):
                if src.name != "main.sm":
                    shutil.copy2(src, root / src.name)
            fixture = REPO / "tests/compiler/programs/quad_evidence_incremental_internal.sm"
            (root / "main.sm").write_bytes(fixture.read_bytes())
            artifact = root / "witness-check.smc"
            built = call(C0, "compile", root / "main.sm", "-o", artifact)
            self.assertEqual(built.returncode, 0, built.stdout + built.stderr)
            self.check_artifact(root, artifact.read_bytes())


if __name__ == "__main__":
    unittest.main()
