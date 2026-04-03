# FX Arithmetic Full Scope

Status: proposed post-stable expansion track
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

That stable reading remains the source of truth until this track explicitly
lands new behavior.

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

1. docs/governance checkpoint
2. plain `fx` unary/binary arithmetic admitted by source typing, with explicit
   canonical-lowering gap diagnostics until the lowering/VM slice lands
