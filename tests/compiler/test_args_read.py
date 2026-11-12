"""I1 differential checks using the attested C0 and the Semantic compiler."""
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import tempfile
import unittest

from reference_identity import qualify_reference


REPO = Path(__file__).resolve().parents[2]


class ArgsReadTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.smc = Path(os.environ["SEMANTIC_SMC"]).resolve()
        cls.c0 = Path(os.environ["SEMANTIC_C0_SMC"]).resolve()
        print(json.dumps(qualify_reference(REPO, cls.smc, cls.c0, os.environ)), flush=True)

    def setUp(self):
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory(prefix="semantic-i1-")))
        self.source = self.root / "probe.sm"

    def run_tool(self, binary, *args):
        cmd = [str(binary), *map(str, args)]
        result = subprocess.run(cmd, cwd=self.root, capture_output=True, text=True,
                                encoding="utf-8", timeout=120)
        print(json.dumps(dict(command=cmd, exit=result.returncode, stdout=result.stdout,
                              stderr=result.stderr)), flush=True)
        return result

    def compile_pair(self, source):
        self.source.write_text(source, encoding="utf-8")
        c0_artifact = self.root / "c0.smc"
        c0_artifact.unlink(missing_ok=True)
        c0 = self.run_tool(self.c0, "compile", self.source, "-o", c0_artifact, "--opt-level", "O0")
        native = self.run_tool(
            self.smc, "run", REPO / "compiler/main.sm", "--profile", "cli-read-only",
            "--root", self.root, "--envelope", "trusted-compiler", "--max-steps", 20000000,
            "--max-calls", 50000, "--max-frames", 256, "--metrics", "json", "--", self.source.name)
        self.assertEqual(native.returncode, 0, native.stderr)
        return c0, c0_artifact, native.stdout.strip()

    def verify_pair(self, c0, c0_artifact, native):
        self.assertEqual(c0.returncode, 0, c0.stderr)
        self.assertRegex(native, r"^hex [0-9a-f]+$")
        artifact = self.root / "native.smc"
        artifact.write_bytes(bytes.fromhex(native[4:]))
        for path in (c0_artifact, artifact):
            verified = self.run_tool(self.c0, "verify", path)
            self.assertEqual(verified.returncode, 0, verified.stderr)
        return artifact

    def assert_args_read_call(self, artifact, index):
        disasm = self.run_tool(self.c0, "disasm", artifact)
        self.assertEqual(disasm.returncode, 0, disasm.stderr)
        code_size = re.search(r"fn main: code=(\d+) bytes", disasm.stdout)
        self.assertIsNotNone(code_size)
        code = artifact.read_bytes()[-int(code_size[1]):]
        count, = struct.unpack_from("<H", code)
        offset = 2
        symbols = []
        for _ in range(count):
            size, = struct.unpack_from("<H", code, offset)
            offset += 2
            symbols.append(code[offset:offset + size])
            offset += size
        ret = re.search(r"^\s*([0-9a-f]+): RET\s*$", disasm.stdout, re.M)
        self.assertIsNotNone(ret)
        instructions = code[-(int(ret[1], 16) + 2):]
        load, arg, value = struct.unpack_from("<BHI", instructions)
        call, has_dst, dst, symbol, argc, call_arg = struct.unpack_from("<BBHHHH", instructions, 7)
        self.assertEqual((load, value), (6, index))
        self.assertEqual((call, has_dst, symbols[symbol], argc, call_arg),
                         (0x40, 1, b"args_read", 1, arg))
        self.assertRegex(disasm.stdout, rf"STORE_VAR s\d+, r{dst}\b")

    def test_positive_calls_emit_verified_named_call(self):
        for index in (0, 1):
            with self.subTest(index=index):
                c0, ref, native = self.compile_pair(
                    f"fn main() {{\n    let argument: text = args_read({index}u32);\n    return;\n}}\n")
                artifact = self.verify_pair(c0, ref, native)
                for path in (ref, artifact):
                    self.assert_args_read_call(path, index)
                    unavailable = self.run_tool(self.c0, "run-smc", path)
                    self.assertEqual(unavailable.returncode, 1)
                    self.assertIn("ABI ArgsRead failure [Unavailable]", unavailable.stderr)
                unsupported = self.run_tool(self.c0, "run-smc", artifact, "--profile", "cli-read-only")
                self.assertEqual(unsupported.returncode, 1)
                self.assertIn("usage: smc run-smc <input.smc>", unsupported.stderr)

    def test_negative_calls_reject_before_lowering(self):
        for body, native_error, c0_error in (
            ("let a: text = args_read();", r"type .*expects 1 argument\(s\), got 0", r"expects 1 args, got 0"),
            ("let a: text = args_read(0u32, 1u32);", r"type .*expects 1 argument\(s\), got 2", r"expects 1 args, got 2"),
            ("let a: text = args_read(true);", r"type .*invalid arguments for builtin 'args_read'", r"expected U32"),
            ('let a: text = args_read("0");', r"type .*invalid arguments for builtin 'args_read'", r"expected U32"),
            ("let a: text = args_read(-1);", r"type .*invalid arguments for builtin 'args_read'", r"expected U32"),
            ("let a: u32 = args_read(0u32);", r"type .*declared u32 but initialized with text", r"type mismatch"),
            ("unrelated_unknown();", r"name .*unknown function 'unrelated_unknown'", r"(?i)unknown function"),
        ):
            with self.subTest(body=body):
                c0, ref, native = self.compile_pair(f"fn main() {{ {body} return; }}\n")
                self.assertEqual(c0.returncode, 1)
                self.assertRegex(c0.stderr, c0_error)
                self.assertFalse(ref.exists())
                self.assertRegex(native, "^error " + native_error)

    def test_reserved_name(self):
        c0, ref, native = self.compile_pair(
            'fn args_read(i: u32) -> text { return "shadow"; }\nfn main() { return; }\n')
        self.assertEqual(c0.returncode, 1)
        self.assertIn("reserved for the application boundary", c0.stderr)
        self.assertFalse(ref.exists())
        self.assertRegex(native, r"^error name .*reserved builtin name")

    def test_map_and_text_control(self):
        c0, ref, native = self.compile_pair(
            'fn main() { let m: Map(u32, text) = map_empty(); '
            'let n: Map(u32, text) = map_set(m, 0u32, "x"); '
            'assert(text_len(map_get(n, 0u32, "")) == 1u32); return; }\n')
        artifact = self.verify_pair(c0, ref, native)
        for path in (ref, artifact):
            result = self.run_tool(self.c0, "run-smc", path)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_c0_source_application_boundary(self):
        self.source.write_text(
            'fn main() { let a: text = args_read(1u32); assert(a == "second"); return; }\n',
            encoding="utf-8")
        for profile, args, expected_exit, outcome, effect_calls, diagnostic in (
            ("cli-read-only", ("first", "second"), 0, "ok", 1, "decision=allow"),
            ("cli-read-only", ("first",), 1, "runtime_error", 1, "application argument 1 is unavailable"),
            ("pure", ("first", "second"), 1, "capability_denied", 0, "MissingCapability"),
        ):
            with self.subTest(profile=profile, args=args):
                result = self.run_tool(self.c0, "run", self.source, "--profile", profile,
                                       "--root", self.root, "--metrics", "json", "--", *args)
                self.assertEqual(result.returncode, expected_exit, result.stderr)
                self.assertIn(diagnostic, result.stderr)
                metrics = next(json.loads(line) for line in result.stderr.splitlines()
                               if line.startswith('{"schema":"semantic.run.metrics'))
                self.assertEqual(metrics["outcome"], outcome)
                self.assertEqual(metrics["counters"]["effect_calls"], effect_calls)
