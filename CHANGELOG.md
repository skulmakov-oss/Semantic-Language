# Changelog

All notable changes to this project are documented in this file.

## Unreleased

### Added
- `v0.1` density-surface wave landed in `main`, including expression-valued
  control, guarded control, composition/call density, flow primitives,
  assertion contracts, const declarations, and expanded numeric literal forms.
- `v0.2` contract/data-core wave landed in `main`, including:
  - tuple literals and tuple types
  - tuple destructuring bind/assignment and tuple `let-else`
  - nominal ADT declarations and constructors
  - ADT match core plus exhaustiveness enforcement
  - `Option(T)` / `Result(T, E)` standard forms and match ergonomics
  - first-wave function contracts: `requires`, `ensures`, and narrow
    `invariant`
  - first-wave units of measure for supported numeric families
- record-layer waves landed in `main`, including canonical nominal records,
  field access, pass/return, equality-safe comparisons, destructuring, narrow
  record `let-else`, immutable copy-with, and shorthand/punning ergonomics.
- `v0.3` schema/boundary-core wave landed in `main`, including:
  - canonical schema declarations with record/tagged-union forms, role markers,
    and version metadata
  - deterministic validation-plan ownership and derived validation checks
  - canonical config-contract parsing and validation paths
  - deterministic generated API-contract artifacts
  - deterministic schema compatibility classification and migration metadata
  - deterministic generated wire-contract artifacts for tagged wire unions and
    record patch types

### Changed
- GitHub roadmap hygiene was normalized so implemented `v0.1`, `v0.2`, `v0.3`,
  density, and record-layer milestones no longer remain open after landing in
  `main`.
- The repository now carries an explicit post-`v0.3` release-freeze checkpoint
  in `docs/roadmap/language_maturity/release_freeze_post_v03_checkpoint.md`.

### Notes
- This section records the post-`v0.3` freeze state only; it does not yet cut a
  new version or tag.
- The next honest release step is asset/release-note/version-cut housekeeping
  on top of the current `main`, not another feature wave.

## v1.0.0 - 2026-02-14

### Added
- `f64` type support in frontend, type checker, AST, and IR.
- Float literals and unary `+` / `-`.
- `f64` arithmetic operators: `+`, `-`, `*`, `/` with precedence parsing.
- SemCode v1 format marker: `SEMCODE1`.
- New math opcodes: `LOAD_F64`, `ADD_F64`, `SUB_F64`, `MUL_F64`, `DIV_F64`.
- VM builtin dispatch for `sin`, `cos`, `tan`, `sqrt`, `abs`, `pow`.
- New v1 golden fixture: `tests/golden_v1/calculator.sm` + `.smc`.
- New example program: `examples/calculator.sm`.

### Changed
- SemCode parser/VM now accept both `SEMCODE0` and `SEMCODE1`.
- Bytecode emitter automatically selects:
  - `SEMCODE0` for v0-compatible programs.
  - `SEMCODE1` when v1 math opcodes are used.
- Golden tests extended to include v1 fixture.

### Compatibility
- Backward compatibility preserved: existing `SEMCODE0` binaries remain executable.

## v0.1.0 - 2026-02-14

### Added
- Toolchain v0 baseline: parser, type-checker, IR lowering, SemCode emitter, VM, CLI.
- SemCode v0 bytecode format (`SEMCODE0`) and golden byte-for-byte tests.

### Notes
- v0 is frozen on branch `release/v0` and tag `v0.1.0`.
