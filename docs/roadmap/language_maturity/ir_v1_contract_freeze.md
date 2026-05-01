# IR v1 Contract Freeze

Status: completed post-stable anti-drift checkpoint

This checkpoint is completed and now serves as frozen baseline history on
current `main`.

## Goal

Freeze the current IR v1 slice as the canonical lowered execution contract for
Semantic `main`.

This checkpoint exists to stop IR from drifting implicitly between:

- source/frontend semantics
- optimizer ownership
- SemCode transport
- VM/runtime execution

The point of this track is contract freeze, not new execution behavior.

## Canonical Reading

IR is the explicit execution-contract boundary in the staged pipeline:

`frontend -> semantics -> lowering -> IR passes -> emit -> VM`

In that reading:

- frontend owns source syntax and source typing rules
- lowering owns translation into execution-shaped IR
- `sm-ir` owns IR structure and optimizer pass ownership
- SemCode remains a later binary contract, not the IR itself
- VM executes admitted SemCode, not raw IR

## Current Landed State

The current `main` already includes:

- canonical IR ownership in `sm-ir`
- canonical top-level IR units:
  - `IrFunction`
  - `IrInstr`
  - `ImmutableIrProgram`
- explicit lowering into execution-shaped control flow
- explicit optimizer ownership in `sm-ir::passes`
- deterministic default pass ordering:
  - `StructuralCleanup v1`
  - `CrystalFold v1`
- admitted ownership-path IR metadata used for runtime ownership transport

That is enough to freeze the owner boundary and the narrow v1 slice.

## Included In This Freeze

- IR as the canonical lowered execution contract
- `sm-ir` as the sole owner of:
  - IR structure
  - lowering-facing IR construction
  - optimizer pass ownership
- explicit boundary between IR and SemCode
- explicit boundary between IR and VM/runtime semantics
- deterministic pass ordering as part of the contract
- honest out-of-scope list for what is not frozen yet

## Explicit Non-Goals

This checkpoint does not include:

- new IR instruction families
- new optimizer behavior
- SemCode version changes
- verifier widening
- VM/runtime widening
- a separate `sm-opt` owner crate
- canonical serialized textual IR format
- CFG notation freeze beyond the current narrow execution-oriented reading

## Freeze Rules

- no frontend or AST structures leak across the IR boundary
- IR remains execution-oriented, not syntax-oriented
- optimizer ownership remains in `sm-ir`
- pass ordering stays explicit and deterministic
- SemCode binary ownership is downstream from IR and must not be folded back
  into the IR contract
- changes to IR structure or admitted pass behavior require explicit contract
  review, not silent drift

## Completed Reading

This checkpoint is now complete because:

- `docs/spec/ir.md` reflects the same staged-contract reading
- architecture docs treat IR as the lowered execution-contract boundary
- roadmap docs point to this file as the completed IR freeze checkpoint
- supported scope and out-of-scope items are explicit
- no document implies a separate `sm-opt` owner or a broader frozen IR surface

