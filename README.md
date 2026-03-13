<p align="center">
  <img src="assets/brand/semantic-logo.svg" alt="Semantic" width="860">
</p>

# Semantic
Rust-like language toolchain with SemCode emitter and VM runtime.

This repository state is frozen as Toolchain v0 on branch `release/v0` and tag `v0.1.0`.

## Changelog
- See `CHANGELOG.md` for release notes (`v0.1.0`, `v1.0.0`).

## What's included
- Parser, type-checker, IR lowering, SemCode emitter.
- SemCode v0 VM executor and disassembler.
- Tools: `smc` for compile/check/lint and `svm` for VM run/disasm.
- Golden byte-format tests.

## Quickstart
Use these commands as-is from repository root.

```powershell
# 1) Build CLI
cargo build --bin smc

# 2) Create minimal source program
@'
fn main() {
    return;
}
'@ | Set-Content program.sm

# 3) Compile source -> SemCode
cargo run --bin smc -- compile program.sm -o program.smc

# 4) Run source directly (compile in memory + execute main)
cargo run --bin smc -- run program.sm

# 5) Run precompiled SemCode
cargo run --bin svm -- run program.smc

# 6) Disassemble SemCode
cargo run --bin svm -- disasm program.smc
```

Example `disasm` output:

```text
SEMCODE0
fn one: code=13 bytes, strings=0
  0000: LOAD_I32 r0, 1
  0007: RET r0
fn main: code=25 bytes, strings=2
  0000: CALL dst?1 r0 fn#0 argc=0
  0008: STORE_VAR s1, r0
  000d: RET
```

## CLI reference
- `smc compile <input.sm> -o <out.smc>`
  - Parses, type-checks, lowers, validates IR, emits SemCode file.
- `smc features`
  - Prints compile-time feature flags baked into this binary.
- `smc run <input.sm>`
  - Compiles source in memory and executes `main` in VM.
- `svm run <input.smc>`
  - Executes precompiled SemCode in VM.
- `svm disasm <input.smc>`
  - Prints decoded functions/opcodes from SemCode payload.

## SemCode v0 format (spec)
- Endianness: little-endian for all integer fields.
- Header: ASCII magic `SEMCODE0` (8 bytes).
- File body: repeated function records until EOF.
  - `u16 name_len`
  - `name_len` bytes function name (UTF-8)
  - `u32 code_len`
  - `code_len` bytes function code section
- Function `code` layout:
  - string table prefix
  - opcode stream payload
- String table layout:
  - `u16 string_count`
  - for each string: `u16 len`, then `len` bytes UTF-8
  - instructions reference strings by `u16 str_id`
- Jumps:
  - encoded as absolute function-local addresses in opcode stream.
  - `Label` nodes do not serialize into bytecode.
- Opcode and encoding source of truth:
  - `src/semcode_format.rs` (`MAGIC`, `Opcode`, read/write LE helpers).

## Language constraints (v0)
- `if` condition is `bool` only. `if quad_expr` is forbidden.
- `->` (implies) is `quad`-only.
- `match` is `quad`-only.
- `match` requires explicit default arm `_ => { ... }`.
- Unit-returning call is valid as statement, invalid as value expression.

## Tests
```powershell
cargo fmt --check
cargo test
cargo test --test golden_semcode
```

## no_std smoke-check
Core library supports `no_std` mode. Run:

```powershell
cargo check --no-default-features
```

Reference matrix: `docs/NO_STD.md`.
Naming note: `docs/NAMING.md`.

## Compile-Time Feature Flags

Default build enables:
- `std`
- `profile-rust`
- `profile-logos`
- `debug-symbols`

Optional:
- `simd`
- `bench`

## Repository layout
- `src/frontend.rs` - lexer/parser/type-checker/lowering/IR validation/emitter.
- `src/semcode_format.rs` - SemCode constants, opcodes, LE read/write helpers.
- `src/semcode_vm.rs` - SemCode parser, VM runtime, disassembler.
- `src/bin/smc.rs` - compiler/tooling entrypoint (`compile`, `check`, `lint`, `watch`, hashes).
- `src/bin/svm.rs` - VM entrypoint (`run`, `disasm`).
- `tests/golden/*` - `.sm` and `.smc` golden fixtures.
- `tests/golden_semcode.rs` - golden byte-for-byte format tests.

## Roadmap
- SemCode structural validator before VM execution.
- VM step limit / sandbox mode for deterministic safety.
- Extend arithmetic typing and lowering (`u32`, `fx` ops).
- Optimize `match` lowering (jump tables/selective chains).
- Stabilize stdlib ABI and calling convention.
- Debug info mapping (source positions -> bytecode offsets).
- SemCode versioning policy and compatibility matrix.

## v1-math (in progress)
- Add first-class `f64` type and float literals in frontend/type checker/IR.
- Add arithmetic opcodes for `f64`: `LOAD_F64`, `ADD_F64`, `SUB_F64`, `MUL_F64`, `DIV_F64`.
- Add math builtins in VM dispatch: `sin`, `cos`, `tan`, `sqrt`, `abs`, `pow`.
- Preserve backward compatibility: VM executes both `SEMCODE0` and `SEMCODE1`.
- Keep all v0 tests/golden fixtures green while extending v1.

## SemCode Versioning Policy
- `SEMCODE0`: frozen v0 format and opcode set.
- `SEMCODE1`: v1 extension format (new MAGIC) for math opcodes.
- VM reader supports both `SEMCODE0` and `SEMCODE1`, so old `.smc` files remain runnable.
