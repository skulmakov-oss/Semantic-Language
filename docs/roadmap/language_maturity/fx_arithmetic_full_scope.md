# FX Arithmetic Full Scope

Status: completed post-stable first-wave track
Related backlog item: `richer fx arithmetic beyond the current value path`
Related stable baseline note:
`docs/roadmap/language_maturity/fx_numeric_contract_notes.md`

## Goal

Extend `fx` from the current stable value-transport and equality family into a
deterministic arithmetic family without widening host/runtime boundaries or
reopening the published `v1.1.1` contract by implication.

This is a post-stable language/runtime expansion track, not a stable-line
correction.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- `fx` has an end-to-end canonical source -> IR -> VM value path
- explicit `fx` literals are stable
- contextual literal admission into `fx` is stable only where the expected type
  is already `fx`
- ordinary `fx` value transport through locals, calls, returns, and equality is
  stable
- binary `+`, `-`, `*`, and `/` on `fx` are not part of the published stable
  contract

That stable reading remains the source of truth for the published stable line.
Current `main` now carries the widened first-wave `fx` arithmetic contract
described below; it does not retroactively change the published `v1.1.1`
boundary.

## Included In This Track

- explicit ownership of general-purpose `fx` unary/binary arithmetic semantics
- deterministic source typing for `fx` arithmetic over already-typed `fx`
  operands
- matching IR lowering and verified VM execution for the admitted `fx`
  arithmetic surface
- diagnostics/spec/tests/goldens sync for the widened `fx` arithmetic contract

## Explicit Non-Goals

- implicit coercion from non-`fx` numeric expressions into `fx`
- widening host ABI, `prom-*`, or runtime integration boundaries
- new floating-point policy or changes to `f64`
- unit-carrying `fx[unit]` arithmetic beyond the already frozen first-wave
  units contract
- generic numeric overloading or operator traits
- reinterpreting historical stable docs as if `v1.1.1` already shipped full
  `fx` arithmetic

## Intended Slice Order

1. docs/governance checkpoint
2. source typing + diagnostics for plain `fx` unary/binary arithmetic
3. IR/lowering/VM admission for the same plain `fx` arithmetic surface
4. docs/spec/test/golden freeze for the widened post-stable contract

## Acceptance Reading

This track is done only when:

- the admitted `fx` arithmetic operator set is explicit and deterministic
- typing/lowering/VM behavior agree on the same `fx` arithmetic surface
- release-facing docs distinguish the published `v1.1.1` baseline from the new
  post-stable widened contract
- no part of the work quietly widens units, host ABI, or general numeric
  coercion rules

## Slice History

- docs/governance checkpoint opened this post-stable track
- plain `fx` unary/binary arithmetic was admitted by source typing
- canonical lowering/verified execution landed for the same plain `fx`
  arithmetic surface under a promoted `SEMCODE3` line
- docs/spec/test freeze landed for the widened post-stable contract

## Close-Out Reading

This first-wave `fx` arithmetic track is now complete on `main`.

Completed first-wave surface:

- deterministic plain unary `+` / `-` on already-typed `fx` expressions
- deterministic plain binary `+`, `-`, `*`, `/` between already-typed `fx`
  operands
- matching source typing, lowering, verifier, VM, and SemCode header behavior
- release-facing docs that distinguish published `v1.1.1` from widened `main`

Still intentionally outside this first wave:

- implicit coercion from non-`fx` numeric expressions into `fx`
- unit-carrying `fx[unit]` arithmetic
- generic numeric overloading or operator traits
- any widening of host ABI, `prom-*`, or runtime integration boundaries
- any claim that `v1.1.1` already shipped full `fx` arithmetic parity with
  `f64`
