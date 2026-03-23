# Function Contract Invariant Scope

Status: proposed checkpoint

## Purpose

Freeze the decision boundary for the remaining `V02-08` work after
`requires` and `ensures` have already landed in `main`.

This document exists so the next step is not "write more contract code" in the
abstract. The next step must be one of two explicit choices:

- implement a narrow first-wave `invariant` slice
- or reshape `#119` so `invariant` moves out of the current done-boundary

## Current Landed State

The current first-wave function contract model already includes:

- entry-side `requires(condition)`
- exit-side `ensures(condition)`
- explicit lowering to core `Assert` instructions
- canonical parser, typecheck, lowering, VM, docs, and tests

This means `#119` is no longer blocked on pre/postcondition symmetry.

## Why A Separate Scope Checkpoint Is Needed

`invariant` is not just another spelling variant of `requires` or `ensures`.
It risks widening the checking model in ways that affect:

- scope visibility
- mutation/update points
- loop and early-return behavior
- the number of execution points that must carry contract checks

Without an explicit checkpoint, it is too easy to either:

- close `#119` too early, pretending `requires + ensures` are enough
- or pull too much semantic machinery into one last "small" slice

## Recommended First-Wave Invariant Shape

If `invariant` stays inside `#119`, keep it narrow.

Recommended first slice:

- declaration-level `invariant(condition)` on ordinary user-defined functions
- the condition uses the same narrow expression subset as `requires/ensures`
- scope is limited to function parameters and, for non-unit returns, optional
  synthetic `result`
- lowering is explicit and deterministic
- checks run only at function entry and function exit

This makes first-wave `invariant` a deliberate sugar layer:

- entry check is equivalent to `requires(condition)`
- exit check is equivalent to `ensures(condition)`

That keeps the slice inside the current contract engine instead of inventing a
new one.

## Explicit Non-Goals For First-Wave Invariant

Do not include any of the following in the first invariant slice:

- statement-local invariants
- loop invariants
- block invariants
- mutation-point rechecking
- call expressions inside invariant conditions
- host or `prom-*` widening
- proof obligations beyond explicit runtime-visible assertions

If any of those become necessary, they should be treated as a later expansion
issue, not as part of the first `V02-08` close-out.

## Binary Decision Rule For `#119`

Close `#119` only if one of these becomes true:

1. a narrow declaration-level `invariant` slice lands in `main`
2. `#119` is explicitly reshaped so the accepted done-boundary is only
   `requires + ensures`

Anything in between is roadmap drift.

## Acceptance Criteria For A Narrow Invariant Slice

- parser accepts declaration-level `invariant(condition)` after
  `requires/ensures`
- sema enforces `bool` type and the same narrow expression subset
- lowering reuses the current explicit contract-check path
- no hidden runtime semantics are introduced
- docs and diagnostics describe the exact first-wave boundary
- tests prove entry and exit checks on the verified path

## Recommended Next Action

Before opening another code PR, decide intentionally whether `#119` keeps
`invariant` inside its current acceptance boundary.

If the answer is yes, the next implementation branch should be a narrow
`invariant-only` slice.

If the answer is no, reshape `#119` first and then close it on the already
landed `requires + ensures` model.
