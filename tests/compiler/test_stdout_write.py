"""Qualify stdout_write typing and named CALL emission against frozen C0."""
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


class StdoutWriteTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.smc = Path(os.environ["SEMANTIC_SMC"]).resolve()
        cls.c0 = Path(os.environ["SEMANTIC_C0_SMC"]).resolve()
        print(json.dumps(qualify_reference(REPO, cls.smc, cls.c0, os.environ)), flush=True)

    def setUp(self):
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory(prefix="semantic-i3-")))

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

    def test_named_unit_call(self):
        cases = (
            'stdout_write("hello");',
            'stdout_write("");',
            'stdout_write("Привет");',
            'let message: text = "variable"; stdout_write(message);',
            'let message: text = args_read(0u32); stdout_write(message);',
            'let message: text = fs_read_text("fixture.txt"); stdout_write(message);',
        )
        for body in cases:
            with self.subTest(body=body):
                for artifact in self.verified_pair(f"fn main() {{ {body} return; }}"):
                    self.assert_named_unit_call(artifact)

    def assert_named_unit_call(self, artifact):
        disasm = self.run_tool(self.c0, "disasm", artifact)
        self.assertEqual(disasm.returncode, 0, disasm.stderr)
        self.assertRegex(disasm.stdout, r"^SEMCOD22 epoch=0\.23\b")
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
        ret = re.search(r"^\s*([0-9a-f]+): RET\s*$", disasm.stdout, re.M)
        self.assertIsNotNone(ret)
        instructions = code[-(int(ret[1], 16) + 2):]
        calls = list(re.finditer(r"^\s*([0-9a-f]+): CALL\b.*$", disasm.stdout, re.M))
        self.assertTrue(calls)
        position = int(calls[-1][1], 16)
        call, has_dst, _, symbol, argc, arg = struct.unpack_from("<BBHHHH", instructions, position)
        self.assertEqual((call, has_dst, symbols[symbol], argc), (0x40, 0, b"stdout_write", 1))
        load = re.search(r"^\s*[0-9a-f]+: LOAD_(?:TEXT|VAR) r(\d+),.*\n\s*[0-9a-f]+: CALL dst\?0",
                         disasm.stdout, re.M)
        self.assertIsNotNone(load)
        self.assertEqual(arg, int(load[1]))
        self.assertEqual(position + 10, int(ret[1], 16))

    def test_rejections(self):
        cases = (
            ("stdout_write();", r"expects 1 argument\(s\), got 0", r"expects 1 args, got 0"),
            ('stdout_write("a", "b");', r"expects 1 argument\(s\), got 2", r"expects 1 args, got 2"),
            ("stdout_write(true);", r"invalid arguments for builtin", r"expected Text"),
            ("stdout_write(0u32);", r"invalid arguments for builtin", r"expected Text"),
            ('let result: text = stdout_write("a");', r"unit", r"(?i)unit|void"),
            ('let result: u32 = stdout_write("a");', r"unit", r"(?i)unit|void"),
            ("unrelated_unknown();", r"unknown function 'unrelated_unknown'", r"(?i)unknown function"),
        )
        for body, semantic_error, c0_error in cases:
            with self.subTest(body=body):
                self.assert_rejected(f"fn main() {{ {body} return; }}", semantic_error, c0_error)
        self.assert_rejected("fn stdout_write(message: text) { return; } fn main() { return; }",
                             r"reserved builtin name", r"reserved for the application boundary")


    def assert_rejected(self, source, semantic_error, c0_error):
        c0, artifact, native = self.compile_pair(source)
        self.assertEqual(c0.returncode, 1)
        self.assertRegex(c0.stderr, c0_error)
        self.assertFalse(artifact.exists())
        self.assertTrue(native.startswith("error "), native)
        self.assertRegex(native, semantic_error)

    def test_existing_calls(self):
        self.verified_pair('fn main() { let arg: text = args_read(0u32); let file: text = fs_read_text("fixture.txt"); return; }')
        source = ('fn size(s: text) -> u32 { return text_len(s); } '
                  'fn main() { let m: Map(u32, text) = map_empty(); '
                  'let n: Map(u32, text) = map_set(m, 0u32, "x"); '
                  'assert(map_contains(n, 0u32)); assert(size(map_get(n, 0u32, "")) == 1u32); return; }')
        for artifact in self.verified_pair(source):
            run = self.run_tool(self.c0, "run-smc", artifact)
            self.assertEqual(run.returncode, 0, run.stderr)
