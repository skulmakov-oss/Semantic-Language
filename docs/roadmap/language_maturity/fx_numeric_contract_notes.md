# FX Numeric Contract Notes

Status: completed stable-line honesty checkpoint
Related milestone: `M2 Language Completion`

## Purpose

Freeze the honest `v1.1.1` reading of the current `fx` contract now that the
canonical value path is end-to-end.

This checkpoint does not widen the published stable line. It freezes the
release-facing distinction between:

- the published stable `v1.1.1` `fx` contract
- the forward-only first-wave plain `fx` arithmetic widening now admitted on
  current `main`

## Published Stable `v1.1.1` Reading

- `fx` has a canonical source -> IR -> VM value path.
- explicit `fx` literals are supported and lower through the fixed-point
  carrier directly.
- contextual literal-to-`fx` admission is supported where the expected type is
  already `fx`.
- existing `fx`-typed values may flow through ordinary locals, parameters,
  returns, calls, and equality.

## Honest `v1.1.1` Limits

- `fx` is currently a value-transport and equality family, not a full arithmetic
  family in the Rust-like source contract.
- binary `+`, `-`, `*`, and `/` on `fx` are not part of the stable contract.
- unary `+` and unary `-` are only admitted when forming an `fx` literal payload
  in the canonical Rust-like path; they are not general-purpose `fx` operators.
- coercion from non-literal non-`fx` numeric expressions into `fx` is not part
  of the stable contract.

## Current `main` Forward-Only Widening

Current `main` now admits one completed first-wave widening beyond the
published stable line:

- deterministic plain unary `+` / unary `-` over already-typed `fx`
  expressions
- deterministic plain binary `+`, `-`, `*`, `/` between already-typed `fx`
  operands
- canonical lowering and verified execution for that admitted arithmetic
  surface under `SEMCODE3`

That widening is forward-only. It does not retroactively change the published
`v1.1.1` release promise.

The completed widening checkpoint is:

- `docs/roadmap/language_maturity/fx_arithmetic_full_scope.md`

## Still Outside The Current Contract

Even after the completed first-wave widening on current `main`, the repository
still does not claim:

- implicit coercion from non-literal non-`fx` numeric expressions into `fx`
- `fx[unit]` arithmetic
- full arithmetic parity between `fx` and `f64`
- any host/runtime/ABI widening justified only by the presence of `fx`
  arithmetic on current `main`

## Release Honesty Rule

Release-facing docs should describe `fx` as:

- stable for explicit fixed-point values and transport through the published
  `v1.1.1` execution path
- widened on current `main` only for the admitted first-wave plain arithmetic
  slice documented above
- still intentionally narrower than `f64` for arithmetic semantics
- not a justification for silently widening host/runtime/operator boundaries on
  either the published stable line or the forward-only first-wave widening
