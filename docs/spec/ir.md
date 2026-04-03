# IR Specification

Status: draft v0
Owner crate: `sm-ir`

## Purpose

This document defines the current intermediate-representation contract between semantic analysis and SemCode production.

IR exists to make execution-relevant structure explicit before bytecode emission.

Current pipeline role:

`source -> AST -> sema -> IR -> opt -> SemCode`

## Current Ownership

`sm-ir` owns:

- IR instruction families
- function-level IR structure
- lowering from frontend and sema-facing programs
- the current optimizer pass surface

`sm-ir` does not own:

- source grammar
- semantic typing rules
- SemCode binary format ownership
- VM execution semantics

Current owner decision:

- optimizer ownership is fixed to `sm-ir` for the current `v1` baseline
- `sm-opt` is not a canonical owner crate in the repository baseline
- any future `sm-opt` split requires an explicit architecture decision and code movement, not just naming changes in docs

## Current IR Shapes

Current top-level IR units:

- `IrFunction`
- `IrInstr`
- `ImmutableIrProgram`

Current `IrInstr` family includes explicit forms for:

- labels and jumps
- loads for quad, bool, i32, f64
- variable load/store
- quad and bool operations
- comparisons
- `f64` arithmetic
- calls and returns
- gate and pulse-oriented effect instructions

The IR is intentionally closer to execution form than frontend AST.

## Lowering Rule

Lowering is responsible for:

- flattening structured frontend control flow
- materializing explicit labels and jumps
- introducing register-indexed operations
- preparing code for deterministic optimization and emission

Current public lowering entrypoints include:

- source to IR
- profile-aware source to IR
- IR and source to SemCode through `sm-ir`

## Optimizer Surface

Current optimizer surface lives in `sm-ir::passes`.

Current canonical pass API includes:

- `OptPass`
- `OptReport`
- `IrModule`
- `run_default_opt_passes`

Current default pipeline:

- `StructuralCleanup` version `1`
- `CrystalFold` version `1`

Contract rule:

- pass ordering is explicit
- optimizer ownership is not split across a second crate in the current baseline
- structural cleanup and fold logic must live in `sm-ir::passes`
- default pipeline must stay discoverable without hidden lowering-only behavior
- `CrystalFold v1` keeps a frozen narrow contract:
  - local instruction-stream constant / identity rewrites only
  - linear deterministic rewrite order
  - barrier clears at labels, jumps, asserts, calls, returns, and other
    explicit control/effect instructions
  - no warning ownership and no source-span ownership

## Boundary Rule

IR may represent effect-oriented instructions such as gate operations, but it does not own:

- host ABI semantics
- capability policy
- runtime state semantics

Those concerns are validated later through producer policy, emitted contract, verifier checks, and execution boundaries.

## Out of Scope for This Draft

This draft does not yet fully formalize:

- every individual `IrInstr` operand invariant
- full CFG notation
- richer optimizer passes beyond `StructuralCleanup v1` and `CrystalFold v1`
- a separate `sm-opt` owner model
- canonical serialized textual IR form
