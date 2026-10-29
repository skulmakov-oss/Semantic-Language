"""Focused OWN0 qualification for the Semantic-written compiler.

The test is intentionally fail-closed: it runs only with the approved compiler
role supplied through SEMANTIC_SMC and never substitutes the host Rust compiler.
"""

import os
import pathlib
import subprocess
import tempfile
import unittest


HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent.parent
SMC = os.environ.get("SEMANTIC_SMC")
INVALID_FIXTURE = HERE / "programs" / "ownership_invalid.sm"


def u16(data, offset):
    if offset < 0 or offset + 2 > len(data):
        raise ValueError("truncated u16")
    return int.from_bytes(data[offset:offset + 2], "little"), offset + 2


def u32(data, offset):
    if offset < 0 or offset + 4 > len(data):
        raise ValueError("truncated u32")
    return int.from_bytes(data[offset:offset + 4], "little"), offset + 4


def str16(data, offset):
    size, offset = u16(data, offset)
    end = offset + size
    if end > len(data):
        raise ValueError("truncated str16")
    return data[offset:end].decode("utf-8"), end


def decode_own0(data):
    """Decode the frozen function envelope and OWN0 structurally."""
    if len(data) < 8 or data[:8] not in (b"SEMCOD22", b"SEMCOD23"):
        raise ValueError("invalid SemCode header")
    offset = 8
    if data[offset:offset + 4] != b"ADT0":
        raise ValueError("missing ADT0")
    offset += 4
    adt_count, offset = u16(data, offset)
    for _ in range(adt_count):
        _, offset = str16(data, offset)
        variants, offset = u16(data, offset)
        for _ in range(variants):
            _, offset = str16(data, offset)
            _, offset = u16(data, offset)
    functions = []
    while offset < len(data):
        name, offset = str16(data, offset)
        code_len, offset = u32(data, offset)
        end = offset + code_len
        if end > len(data):
            raise ValueError("truncated function envelope")
        code = data[offset:end]
        cursor = 0
        string_count, cursor = u16(code, cursor)
        strings = []
        for _ in range(string_count):
            value, cursor = str16(code, cursor)
            strings.append(value)
        if cursor + 4 > len(code) or code[cursor:cursor + 4] != b"OWN0":
            raise ValueError("missing OWN0")
        cursor += 4
        own_count, cursor = u16(code, cursor)
        events = []
        for _ in range(own_count):
            if cursor + 2 > len(code):
                raise ValueError("truncated OWN0 event header")
            kind = code[cursor]
            mode = code[cursor + 1]
            cursor += 2
            anchor, cursor = u32(code, cursor)
            root, cursor = u32(code, cursor)
            component_count, cursor = u16(code, cursor)
            components = []
            for _ in range(component_count):
                if cursor >= len(code):
                    raise ValueError("truncated OWN0 component kind")
                component_kind = code[cursor]
                cursor += 1
                if component_kind == 1:
                    field, cursor = u32(code, cursor)
                    components.append((component_kind, field))
                elif component_kind == 0:
                    index, cursor = u16(code, cursor)
                    components.append((component_kind, index))
                else:
                    raise ValueError("unsupported OWN0 component")
            events.append(dict(kind=kind, mode=mode, anchor=anchor, root=root,
                               components=components))
        if cursor + 4 > len(code) or code[cursor:cursor + 4] != b"SIG0":
            raise ValueError("missing SIG0")
        cursor += 4
        sig_count, cursor = u16(code, cursor)
        cursor += sig_count
        if cursor > len(code):
            raise ValueError("truncated SIG0")
        functions.append(dict(name=name, strings=strings, events=events, code=code[cursor:]))
        offset = end
    return functions


def instruction_starts(code):
    starts = {}
    offset = 0
    while offset < len(code):
        starts[offset] = code[offset]
        op = code[offset]
        if op in (2,):
            size = 4
        elif op in (3, 6):
            size = 7
        elif op in (4, 5, 22, 90, 95, 107):
            size = 5
        elif op in (48, 49):
            size = 5 if op == 48 else 7
        elif op in (66, 112):
            size = 3
        elif op in (114, 115, 70, 73):
            size = 9
        elif op == 68:
            size = 7
        elif op == 64:
            if offset + 8 > len(code):
                raise ValueError("truncated CALL")
            argc = int.from_bytes(code[offset + 6:offset + 8], "little")
            size = 8 + 2 * argc
        elif op in (67, 91, 69):
            if offset + 7 > len(code):
                raise ValueError("truncated aggregate instruction")
            argc = int.from_bytes(code[offset + 5:offset + 7], "little")
            size = 7 + 2 * argc if op == 69 else 5 + 2 * argc
        elif op == 65:
            if offset + 2 > len(code):
                raise ValueError("truncated RET")
            size = 4 if code[offset + 1] == 1 else 2
        else:
            size = 7
        if offset + size > len(code):
            raise ValueError("truncated instruction")
        offset += size
    return starts


@unittest.skipUnless(SMC and pathlib.Path(SMC).is_file(), "SEMANTIC_SMC is required")
class OwnershipSemcodeTests(unittest.TestCase):
    def test_record_updates_emit_nonempty_own0_and_verify(self):
        source = """record R { x: i32, y: i32 }

fn main() {
    let x: i32 = 1;
    if true { x = 2; } else { x = 4; }
    let r: R = R { x: 1, y: 2 };
    let s: R = r with { x: 3, y: 4 };
    assert(s.x == 3);
    assert(s.y == 4);
    return;
}
"""
        with tempfile.TemporaryDirectory(prefix="semantic-own0-") as tmp:
            root = pathlib.Path(tmp)
            (root / "ownership.sm").write_text(source, encoding="utf-8", newline="")
            result = subprocess.run(
                [SMC, "run", "main.sm", "--profile", "cli-read-only",
                 "--root", root, "--envelope", "trusted-compiler",
                 "--max-steps", "20000000", "--max-calls", "50000",
                 "--max-frames", "256", "--", "ownership.sm"],
                cwd=REPO / "compiler", capture_output=True, text=True,
                encoding="utf-8", timeout=1800,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            line = next((line for line in result.stdout.splitlines() if line.startswith("hex ")), "")
            self.assertTrue(line, result.stdout + result.stderr)
            artifact = bytes.fromhex(line[4:].strip())
            functions = decode_own0(artifact)
            self.assertEqual(len(functions), 1)
            events = functions[0]["events"]
            self.assertEqual([event["kind"] for event in events], [1, 1, 1, 1], events)
            self.assertEqual([event["mode"] for event in events], [0, 0, 1, 1], events)
            starts = instruction_starts(functions[0]["code"])
            self.assertTrue(all(event["anchor"] in starts for event in events))
            self.assertEqual(starts[events[0]["anchor"]], 5)
            self.assertEqual(starts[events[2]["anchor"]], 69)
            self.assertEqual(starts[events[3]["anchor"]], 69)
            self.assertEqual(events[2]["anchor"], events[3]["anchor"])
            self.assertEqual([event["components"][0][0] for event in events[2:]], [1, 1])
            self.assertNotEqual(events[2]["components"], events[3]["components"])
            field_names = {
                functions[0]["strings"][event["components"][0][1]]
                for event in events[2:]
            }
            self.assertEqual(field_names, {"x", "y"})
            self.assertEqual(events[0]["root"], 0)
            self.assertEqual(events[1]["root"], 0)
            self.assertEqual(events[2]["root"], 1)
            self.assertEqual(events[3]["root"], 1)
            self.assertEqual(functions[0]["strings"][events[0]["root"]], "x#0")
            self.assertEqual(functions[0]["strings"][events[2]["root"]], "r#1")

    def test_instruction_boundary_vectors_are_exact(self):
        self.assertEqual(instruction_starts(bytes([64, 0, 0, 0, 0, 0, 1, 0, 1, 0, 65, 0])), {0: 64, 10: 65})
        self.assertEqual(instruction_starts(bytes([67, 0, 0, 1, 0, 1, 0, 65, 0])), {0: 67, 7: 65})
        self.assertEqual(instruction_starts(bytes([69, 0, 0, 0, 0, 1, 0, 0, 0, 65, 0])), {0: 69, 9: 65})
        self.assertEqual(instruction_starts(bytes([91, 0, 0, 1, 0, 1, 0, 65, 0])), {0: 91, 7: 65})
        with self.assertRaises(ValueError):
            instruction_starts(bytes([69, 0, 0, 0, 0, 1, 0, 0]))

    def test_truncated_own0_fields_fail_closed(self):
        with self.assertRaises(ValueError):
            decode_own0(b"SEMCOD22")

            out = root / "ownership.smc"
            out.write_bytes(artifact)
            verified = subprocess.run([SMC, "verify", out], capture_output=True,
                                      text=True, encoding="utf-8", timeout=1800)
            self.assertEqual(verified.returncode, 0, verified.stdout + verified.stderr)
            executed = subprocess.run([SMC, "run-smc", out], capture_output=True,
                                       text=True, encoding="utf-8", timeout=1800)
            self.assertEqual(executed.returncode, 0, executed.stdout + executed.stderr)

    def test_invalid_ownership_anchors_fail_before_artifact(self):
        for mode in ("missing", "wrong_store", "wrong_record"):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory(prefix="semantic-own0-invalid-") as tmp:
                root = pathlib.Path(tmp)
                for name in ("semcode.sm", "bytes.sm"):
                    source_file = REPO / "compiler" / name
                    (root / name).write_text(source_file.read_text(encoding="utf-8"), encoding="utf-8")
                (root / "lower.sm").write_text(
                    """record Ins { op: i32, a: i32, b: i32, c: i32, imm: i32, args: Sequence(i32) }
record OwnPath { kind: i32, mode: i32, root: i32, anchor: i32, comp_kind: i32, comp_a: i32, comp_b: i32 }
record FnIr { name: text, families: Sequence(i32), strings: Sequence(text), ins: Sequence(Ins), owns: Sequence(OwnPath) }
""", encoding="utf-8")
                fixture = INVALID_FIXTURE.read_text(encoding="utf-8").replace('"missing"', f'"{mode}"')
                (root / "invalid.sm").write_text(fixture, encoding="utf-8")
                result = subprocess.run(
                    [SMC, "run", "main.sm", "--profile", "cli-read-only", "--root", root,
                     "--envelope", "trusted-compiler", "--max-steps", "20000000",
                     "--max-calls", "50000", "--max-frames", "256", "--", "invalid.sm", mode],
                    cwd=REPO / "compiler", capture_output=True, text=True,
                    encoding="utf-8", timeout=1800,
                )
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                line = next((line for line in result.stdout.splitlines() if line.startswith("hex ")), "")
                self.assertTrue(line, result.stdout + result.stderr)
                artifact = root / "invalid.smc"
                artifact.write_bytes(bytes.fromhex(line[4:].strip()))
                executed = subprocess.run(
                    [SMC, "run-smc", artifact], capture_output=True, text=True,
                    encoding="utf-8", timeout=1800,
                )
                self.assertEqual(executed.returncode, 0, executed.stdout + executed.stderr)


if __name__ == "__main__":
    unittest.main()
