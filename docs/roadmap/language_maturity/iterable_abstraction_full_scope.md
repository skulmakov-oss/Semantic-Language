# Iterable Abstraction Full Scope

Status: active M9.3 post-stable subtrack
Related roadmap package:
`docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`

## Goal

Introduce a narrow first-wave iterable/iterator abstraction that lets Semantic
programs iterate over `Sequence(T)` and user-defined collections without
duplicating loop logic, gated behind the M9.2 trait foundation.

This is a forward-only language-maturity subtrack for current `main`. It is not
a claim that iterable abstraction already exists on the published stable line.

## Prerequisite

M9.2 Traits/Protocols must be completed before this track begins. Iterable
abstraction depends on a trait as its contract: the `Iterable` trait is defined
in terms of the trait system introduced in M9.2. Starting this track before
M9.2 closes would require re-doing the contract surface.

## Why This Track Exists

Semantic now has generics on `main` via M9.1 and will have traits/protocols via
M9.2. The next practical barrier to expressive collection code is the inability
to iterate over user-defined types with a uniform `for x in collection` syntax.
Without iterable abstraction:

- every loop over a user-defined collection requires manual index arithmetic
- `for x in collection` syntax is unavailable for user types
- range types exist but are not user-extensible into a uniform iteration model
- there is no standard contract for a type to declare it is traversable

This track opens the minimum first-class iterable surface using the trait
system without mixing in lazy evaluation, async iteration, or combinator chains.

## Decision Check

- [x] This is a new explicit post-stable track with its own scope decision
- [x] This does not silently widen published `v1.1.1`
- [x] This is one stream, not a mixture of multiple tracks
- [x] This can be closed with a clear done-boundary

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- `Sequence(T)` exists with `expr[index]` access but no for-each style iteration
  over user types
- there is no `Iterable` trait or equivalent in the published stable baseline
- `for x in collection` desugaring is not part of the published stable contract
- range types exist but are not user-extensible into a shared iteration model
- no standard stdlib contract exists for user-defined traversable types

That baseline remains the source of truth until this subtrack explicitly lands
its widened contract on `main`.

## Activation Reading

`M9.3 Iterable Abstraction` is now the active language-maturity stream after
the completion of:

- `IR v1 contract freeze`
- `SemCode version discipline`
- `runtime boundary hardening`

This activation is a scope/governance checkpoint only. It does not itself claim
that iterable abstraction is admitted yet on current `main`.

## Included In This Track

- `Iterable` trait definition in stdlib, using the M9.2 trait system as its
  contract surface
- `for x in collection` desugaring to the `Iterable` trait call
- `Sequence(T)` implementation of `Iterable`
- range types implementation of `Iterable`
- user record and ADT definitions may implement `Iterable`
- docs/spec/tests/compatibility wording for the widened contract

## Explicit Non-Goals

- lazy iterators or adapter chains (map, filter, zip)
- async iteration
- infinite iterators
- iterator combinators beyond a basic next/has-next contract
- SIMD or vectorized iteration
- higher-order iterator pipelines
- variance or lifetime annotations on iterator types
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance

- scope checkpoint
- roadmap/milestone/plan linkage

### Wave 1 — Owner Layer

- `Iterable` trait ownership and stdlib placement
- `for x in collection` desugaring ownership
- explicit typecheck/lowering gap markers before executable admission

### Wave 2 — Source Admission

- parser admission for `for x in collection` syntax over user types
- sema/type admission for `Iterable` trait impl checking
- explicit diagnostics for unsupported iterable forms

### Wave 3 — Stdlib Impl + Typecheck

- built-in executable iterable slice for `Sequence(T)` and range values
- explicit typecheck/lowering agreement that built-in iterable loops run on
  current `main`
- explicit diagnostics that user-defined `Iterable` impl dispatch is still
  deferred

### Wave 4 — Explicit Impl Dispatch

- freeze the executable contract for explicit user-defined `Iterable` impl
  dispatch before code changes
- require one deterministic trait method shape for the first executable
  user-defined slice:
  `fn next(self: Self, index: i32) -> Option(Item)`
- treat `index` as the zero-based loop-driver cursor supplied by the lowered
  `for x in collection` execution path
- continue the loop on `Option::Some(item)` and terminate on `Option::None`
- do not introduce hidden mutable iterator state, host callbacks, or dynamic
  dispatch in this first executable user-defined slice
- executable lowering/runtime wiring for explicit user-defined `Iterable` impls
  lands only after that contract is frozen

### Wave 4A — Impl Method Executable Contract

- impl method bodies typecheck on current `main`
- impl methods lower into executable internal functions on current `main`
- trait-side `Self` now survives parser/typecheck as the narrow impl-anchored
  receiver contract on current `main`
- this prerequisite is now complete and removes the earlier dead owner-layer
  gap for trait/impl method bodies
- this does not by itself claim user-visible iterable loop execution over
  explicit impls yet

### Wave 5 — Freeze

- docs/spec/tests/compatibility freeze

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint
2. PR 2: owner-layer `Iterable` trait surface and desugaring ownership
3. PR 3: parser/sema/type admission for `for x in collection`
4. PR 4: built-in executable iterable slice for `Sequence(T)` and ranges
5. PR 5: impl method executable contract prerequisite
6. PR 6: explicit iterable impl dispatch contract freeze
7. PR 7: explicit user-defined `Iterable` impl dispatch
8. PR 8: freeze and close-out

## Initial First-Wave Reading

The first-wave iterable contract is intentionally narrow:

- one trait (`Iterable`) as the single iteration contract
- `for x in collection` desugars to that trait call only
- no combinator surface in Wave 1–3
- user types may implement `Iterable` directly; no auto-derive machinery required
- no implicit coercion across iterable boundaries

That keeps the track additive over the current concrete collection surfaces
without opening a full lazy-evaluation or combinator system in one step.

## Explicit Dispatch Contract Freeze

The next executable user-defined iterable slice must stay narrow and
deterministic.

Approved direction for the next step:

- explicit user-defined iterable loops do not use hidden iterator objects or
  host-managed mutable iteration state
- the loop driver owns the cursor and passes it explicitly as `index: i32`
- the executable trait hook is:
  `fn next(self: Self, index: i32) -> Option(Item)`
- `Option::Some(item)` yields one loop item and increments the cursor
- `Option::None` terminates the loop
- the built-in `Sequence(T)` / range path remains the already-landed separate
  executable slice on current `main`

Still out of scope for this first explicit-dispatch slice:

- ADT/schema-specific iteration contracts
- hidden mutable iterator cells
- dynamic trait dispatch
- lazy combinator pipelines
- non-`i32` driver cursors

## Acceptance Reading

This track is done only when:

- the `Iterable` trait is explicit and inspectable in stdlib
- `for x in collection` desugaring, typecheck, and execution agree on one
  deterministic first-wave model
- `Sequence(T)` and range types both satisfy the `Iterable` contract
- docs/spec/tests describe the same admitted baseline
- published `v1.1.1` and widened `main` are explicitly distinguished

## Non-Commitments After Close-Out

Even after this first wave lands, the repository still does not claim:

- lazy iterators or adapter chains
- async or concurrent iteration
- iterator combinators beyond basic next/has-next
- SIMD or vectorized traversal
- that iterable abstraction was already part of the published `v1.1.1` line

## Merge Gate

Before closing this track:

- [ ] code/tests are green
- [ ] spec/docs are synced
- [ ] public API or golden snapshots are updated if needed
- [ ] compatibility/release-facing wording is honest
