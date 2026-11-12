"""Qualify fs_read_text typing and named CALL emission against frozen C0."""
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


class FsReadTextTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.smc = Path(os.environ["SEMANTIC_SMC"]).resolve()
        cls.c0 = Path(os.environ["SEMANTIC_C0_SMC"]).resolve()
        print(json.dumps(qualify_reference(REPO, cls.smc, cls.c0, os.environ)), flush=True)

    def setUp(self):
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory(prefix="semantic-i2-")))

    def run_tool(self, binary, *args):
        command = [str(binary), *map(str, args)]
        run = subprocess.run(command, cwd=self.root, capture_output=True, encoding="utf-8", timeout=120)
        print(json.dumps(dict(command=command, exit=run.returncode, stdout=run.stdout,
                              stderr=run.stderr)), flush=True)
        return run

    def compile_pair(self, source):
        fixture = self.root / "probe.sm"
        fixture.write_text(source, encoding="utf-8")
        c0_artifact = self.root / "c0.smc"
        c0_artifact.unlink(missing_ok=True)
        c0 = self.run_tool(self.c0, "compile", fixture, "-o", c0_artifact, "--opt-level", "O0")
        native = self.run_tool(
            self.smc, "run", REPO / "compiler/main.sm", "--profile", "cli-read-only",
            "--root", self.root, "--envelope", "trusted-compiler", "--max-steps", 20000000,
            "--max-calls", 50000, "--max-frames", 256, "--metrics", "json", "--", fixture.name)
        self.assertEqual(native.returncode, 0, native.stderr)
        return c0, c0_artifact, native.stdout.strip()

    def verified_pair(self, source):
        c0, ref, native = self.compile_pair(source)
        self.assertEqual(c0.returncode, 0, c0.stderr)
        self.assertRegex(native, r"^hex [0-9a-f]+$")
        artifact = self.root / "semantic.smc"
        artifact.write_bytes(bytes.fromhex(native[4:]))
        for path in (ref, artifact):
            check = self.run_tool(self.c0, "verify", path)
            self.assertEqual(check.returncode, 0, check.stderr)
        return ref, artifact

    def test_named_call(self):
        for body in ('let content: text = fs_read_text("sample.txt");',
                     'let path: text = "nested/sample.txt"; let content: text = fs_read_text(path);'):
            with self.subTest(body=body):
                for artifact in self.verified_pair(f"fn main() {{ {body} return; }}"):
                    disasm = self.run_tool(self.c0, "disasm", artifact)
                    self.assertEqual(disasm.returncode, 0, disasm.stderr)
                    size = re.search(r"fn main: code=(\d+) bytes", disasm.stdout)
                    self.assertIsNotNone(size)
                    code = artifact.read_bytes()[-int(size[1]):]
                    count, = struct.unpack_from("<H", code)
                    offset = 2
                    symbols = []
                    for _ in range(count):
                        length, = struct.unpack_from("<H", code, offset)
                        offset += 2
                        symbols.append(code[offset:offset + length])
                        offset += length
                    call_line = re.search(r"^\s*([0-9a-f]+): CALL\b", disasm.stdout, re.M)
                    ret = re.search(r"^\s*([0-9a-f]+): RET\s*$", disasm.stdout, re.M)
                    self.assertIsNotNone(call_line)
                    self.assertIsNotNone(ret)
                    instructions = code[-(int(ret[1], 16) + 2):]
                    call, has_dst, dst, symbol, argc, arg = struct.unpack_from(
                        "<BBHHHH", instructions, int(call_line[1], 16))
                    self.assertEqual((call, has_dst, symbols[symbol], argc), (0x40, 1, b"fs_read_text", 1))
                    self.assertRegex(disasm.stdout, rf", r{arg}\b|\br{arg},")
                    self.assertRegex(disasm.stdout, rf"STORE_VAR s\d+, r{dst}\b")

    def test_utf8_source_execution(self):
        (self.root / "nested").mkdir()
        (self.root / "nested/sample.txt").write_text("Привет", encoding="utf-8")
        source = ('fn main() { let path: text = "nested/sample.txt"; '
                  'let content: text = fs_read_text(path); assert(content == "Привет"); return; }')
        self.verified_pair(source)
        run = self.run_tool(self.c0, "run", self.root / "probe.sm", "--profile",
                            "cli-read-only", "--root", self.root, "--metrics", "json")
        self.assertEqual(run.returncode, 0, run.stderr)
        self.assertIn("capability=fs.read decision=allow bytes=12", run.stderr)

    def test_rejections(self):
        cases = (
            ("let x: text = fs_read_text();", r"expects 1 argument\(s\), got 0", r"expects 1 args, got 0"),
            ('let x: text = fs_read_text("a.txt", "b.txt");', r"expects 1 argument\(s\), got 2", r"expects 1 args, got 2"),
            ("let x: text = fs_read_text(true);", r"invalid arguments for builtin", r"expected Text"),
            ("let x: text = fs_read_text(0u32);", r"invalid arguments for builtin", r"expected Text"),
            ('let x: u32 = fs_read_text("sample.txt");', r"declared u32 but initialized with text", r"type mismatch"),
            ("unrelated_unknown();", r"unknown function 'unrelated_unknown'", r"(?i)unknown function"),
        )
        for body, semantic_error, c0_error in cases:
            with self.subTest(body=body):
                self.assert_rejected(f"fn main() {{ {body} return; }}", semantic_error, c0_error)
        self.assert_rejected('fn fs_read_text(path: text) -> text { return path; } fn main() { return; }',
                             r"reserved builtin name", r"reserved for the application boundary")

    def assert_rejected(self, source, semantic_error, c0_error):
        c0, artifact, native = self.compile_pair(source)
        self.assertEqual(c0.returncode, 1)
        self.assertRegex(c0.stderr, c0_error)
        self.assertFalse(artifact.exists())
        self.assertTrue(native.startswith("error "), native)
        self.assertRegex(native, semantic_error)

    def test_existing_calls(self):
        self.verified_pair("fn main() { let arg: text = args_read(0u32); return; }")
        source = ('fn size(s: text) -> u32 { return text_len(s); } '
                  'fn main() { let m: Map(u32, text) = map_empty(); '
                  'let n: Map(u32, text) = map_set(m, 0u32, "x"); '
                  'assert(map_contains(n, 0u32)); assert(size(map_get(n, 0u32, "")) == 1u32); return; }')
        for artifact in self.verified_pair(source):
            run = self.run_tool(self.c0, "run-smc", artifact)
            self.assertEqual(run.returncode, 0, run.stderr)
