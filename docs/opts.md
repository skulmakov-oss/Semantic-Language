# Optimization Passes (v0.2)

## StructuralCleanup

- Location: `crates/sm-ir/src/passes/cleanup.rs`
- Type: IR optimization pass (`OptPass`)
- Entry: `run_default_opt_passes()` in `crates/sm-ir/src/passes/mod.rs`

StructuralCleanup is an IR-stage cleanup pass. It is not part of lowering itself, and its
implementation must not live in `legacy_lowering.rs`.

## CrystalFold

- Location: `crates/sm-ir/src/passes/crystalfold.rs`
- Type: IR optimization pass (`OptPass`)
- Entry: `run_default_opt_passes()` in `crates/sm-ir/src/passes/mod.rs`

CrystalFold is an IR-stage pass. It is not part of parsing, smc typing, or emit.
Its frozen `v1` contract is intentionally narrow:

- name: `CrystalFold`
- version: `1`
- scope: local constant-fold and identity rewrites over the current IR instruction
  stream only
- rewrite order: linear instruction traversal in original instruction order
- barriers: `Label`, jumps, `Assert`, `Call`, `Ret`, and other explicit effect /
  control instructions clear fold state before the next instruction
- diagnostics boundary: CrystalFold emits no warnings and owns no source-span
  reasoning; semantics-owned hints such as `W0241` stay advisory

## W0241 Scope

- `W0241` is emitted by semantics as a hint/diagnostic only.
- `W0241` does not guarantee a rewrite happened.
- Materialized rewrites are performed by CrystalFold on IR.

## Guarantees

- Deterministic: linear instruction traversal, stable rewrite order.
- Idempotent: applying CrystalFold twice is equivalent to once.
- Barrier-safe: constant state does not flow across labels, jumps, calls, or
  other explicit control/effect boundaries.
- Covered by tests:
  - `crystalfold_surface_stays_frozen_at_v1`
  - `crystalfold_idempotent`
  - `crystalfold_clears_constant_state_across_barriers`
  - `crystalfold_rewrite_order_and_report_are_deterministic`

## Pipeline Order

`frontend -> semantics (warnings) -> lowering -> StructuralCleanup -> CrystalFold -> emit`

