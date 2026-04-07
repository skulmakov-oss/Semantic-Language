# Iterable Abstraction Full Scope

Status: proposed M9.3 post-stable subtrack
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

- [ ] This is a new explicit post-stable track with its own scope decision
- [ ] This does not silently widen published `v1.1.1`
- [ ] This is one stream, not a mixture of multiple tracks
- [ ] This can be closed with a clear done-boundary

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

- stdlib `Iterable` trait definition
- `Sequence(T)` impl of `Iterable`
- range types impl of `Iterable`
- typecheck enforcement that `for x in collection` requires `Iterable` impl

### Wave 4 — Freeze

- docs/spec/tests/compatibility freeze

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint
2. PR 2: owner-layer `Iterable` trait surface and desugaring ownership
3. PR 3: parser/sema/type admission for `for x in collection`
4. PR 4: stdlib impl for `Sequence(T)` and range types, typecheck enforcement
5. PR 5: freeze and close-out

## Initial First-Wave Reading

The first-wave iterable contract is intentionally narrow:

- one trait (`Iterable`) as the single iteration contract
- `for x in collection` desugars to that trait call only
- no combinator surface in Wave 1–3
- user types may implement `Iterable` directly; no auto-derive machinery required
- no implicit coercion across iterable boundaries

That keeps the track additive over the current concrete collection surfaces
without opening a full lazy-evaluation or combinator system in one step.

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
