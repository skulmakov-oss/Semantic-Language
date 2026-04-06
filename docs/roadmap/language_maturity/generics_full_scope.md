# Generics Full Scope

Status: proposed M9.1 post-stable subtrack
Related roadmap package:
`docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`

## Goal

Introduce the first admitted parametric polymorphism surface for Semantic
without silently widening the published `v1.1.1` line and without opening
trait-based abstraction, async, or runtime machinery ahead of schedule.

This is a forward-only language-maturity subtrack for current `main`. It is not
a claim that generics already exist on the published stable line.

## Why This Track Exists

Semantic now has text, packages, collections, and first-class closures on
`main`. All four foundations are completed first-wave baselines. The next
barrier to practical code reuse is the inability to write type-parametric
definitions. Without generics:

- `Sequence(T)` cannot be written generically over user-defined types
- `Closure(T -> U)` cannot be composed generically
- `Option(T)` and `Result(T, E)` cannot be opened to user-defined type families
- record and ADT definitions must be duplicated for each concrete type

This track opens the minimum first-class generic surface without mixing in
trait dispatch, async abstractions, or higher-kinded types.

## Decision Check

- [ ] This is a new explicit post-stable track with its own scope decision
- [ ] This does not silently widen published `v1.1.1`
- [ ] This is one stream, not a mixture of multiple tracks
- [ ] This can be closed with a clear done-boundary

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- there are no type parameters in the public language contract
- all concrete types (`i32`, `u32`, `f64`, `fx`, `bool`, `quad`, `text`,
  `Sequence(T)`, `Closure(T -> U)`) use fixed or structurally admitted forms
- `Option(T)` and `Result(T, E)` exist as standard library forms but are not
  user-parameterisable in the published stable baseline
- record and ADT definitions take no type parameters in the published stable
  line
- published `v1.1.1` does not claim user-defined generic types or generic
  functions

That baseline remains the source of truth until this subtrack explicitly lands
its widened contract on `main`.

## Included In This Track

- one first-wave type-parameter family for functions and record/ADT definitions
- a narrow type-parameter spelling for admitted source positions
- deterministic monomorphisation policy
- generic function definitions and call-site instantiation
- generic record and ADT definitions
- docs/spec/tests/compatibility wording for the widened contract

## Explicit Non-Goals

- higher-kinded types
- variance annotations (covariance, contravariance)
- trait/protocol bounds on type parameters (deferred to M9.2)
- associated types or type families
- generic closures beyond what first-wave monomorphisation admits
- specialisation or template-based optimisation
- implicit type-class dispatch
- variadic generics
- lifetime or region annotations
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance

- scope checkpoint
- roadmap/milestone/plan linkage

### Wave 1 — Owner Layer

- type-parameter syntax ownership
- generic definition and instantiation metadata inventory
- monomorphisation policy boundaries
- explicit typecheck/lowering gap markers before executable admission

### Wave 2 — Source Admission

- parser admission for type-parameter syntax
- sema/type admission for generic definitions and call-site instantiation
- explicit diagnostics for unsupported generic forms

### Wave 3 — Lowering Path

- IR monomorphisation pass
- lowering of generic definitions to concrete SemCode paths
- verifier and VM compatibility for monomorphised output

### Wave 4 — Freeze

- docs/spec/tests/compatibility freeze

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint
2. PR 2: owner-layer type-parameter surface
3. PR 3: parser/sema/type admission
4. PR 4: IR monomorphisation and lowering path
5. PR 5: freeze and close-out

## Initial First-Wave Reading

The first-wave generic contract is intentionally narrow:

- one type-parameter per definition site only
- monomorphisation only (no runtime generic dispatch)
- no trait/protocol bounds in Wave 1–3
- generic functions, records, and ADTs admitted; generic closures follow from
  monomorphisation automatically
- no implicit coercion across generic boundaries

That keeps the track additive over the current concrete type surfaces without
opening a full abstraction system in one step.

## Acceptance Reading

This track is done only when:

- one first-wave type-parameter family is explicit and inspectable
- generic definitions, monomorphisation, and call-site instantiation agree on
  one deterministic first-wave model
- docs/spec/tests describe the same admitted baseline
- published `v1.1.1` and widened `main` are explicitly distinguished

## Non-Commitments After Close-Out

Even after this first wave lands, the repository still does not claim:

- trait/protocol-based generic bounds or dispatch
- higher-kinded types or type constructors
- variance, lifetimes, or region-based memory semantics
- specialisation or template metaprogramming
- that generics were already part of the published `v1.1.1` line

## Merge Gate

Before closing this track:

- [ ] code/tests are green
- [ ] spec/docs are synced
- [ ] public API or golden snapshots are updated if needed
- [ ] compatibility/release-facing wording is honest
