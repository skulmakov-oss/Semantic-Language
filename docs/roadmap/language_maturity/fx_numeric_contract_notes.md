# FX Numeric Contract Notes

Status: completed stable-line honesty checkpoint
Related milestone: `M2 Language Completion`

## Purpose

Freeze the honest `v1.1.1` reading of the current `fx` contract now that the
canonical value path is end-to-end.

This checkpoint does not widen `fx` semantics. It narrows the wording around
what is already stable.

## Current Stable Reading

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
- richer `fx` arithmetic remains post-`v1` work and must not be implied by the
  current end-to-end value path.

## Release Honesty Rule

Release-facing docs should describe `fx` as:

- stable for explicit fixed-point values and transport through the current
  canonical execution path
- intentionally narrower than `f64` for arithmetic semantics
- not a justification for widening host/runtime/operator boundaries on the
  stable line
