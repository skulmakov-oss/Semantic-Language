# Traits Protocols Full Scope

Status: proposed M9.2 post-stable subtrack
Related roadmap package:
`docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`

## Goal

Introduce the first admitted structural trait/protocol surface for Semantic —
named behavior contracts with explicit impl blocks, static dispatch only in
first wave — without opening vtable dispatch, async traits, or higher-ranked
abstractions ahead of schedule.

This is a forward-only language-maturity subtrack for current `main`. It is not
a claim that traits or protocols already exist on the published stable line.

## Why This Track Exists

Semantic now has generics/parametric polymorphism on `main` (M9.1 completed).
The current state has type-parametric definitions but no mechanism to constrain
type parameters to a named behavior contract. Without traits/protocols:

- `<T>` type parameters carry no static guarantees beyond what the call site
  can check structurally
- there is no named abstraction boundary for reusable behavior sets
- generic code cannot express "T must support these operations" in a
  checkable, doc-able form
- polymorphism remains purely structural with no behavior abstraction layer

This track opens the minimum first-class trait/protocol surface without mixing
in vtable dispatch, async traits, or higher-ranked bounds.

## Decision Check

- [ ] This is a new explicit post-stable track with its own scope decision
- [ ] This does not silently widen published `v1.1.1`
- [ ] This is one stream, not a mixture of multiple tracks
- [ ] This can be closed with a clear done-boundary

## Stable Baseline Before This Track

The current `main` baseline (after M9.1) already freezes these facts:

- generics/parametric polymorphism are admitted for functions and record/ADT
  definitions on `main`
- all polymorphism remains structural; there is no trait or protocol syntax in
  the published language contract
- type parameters carry no named behavior bounds in the current baseline
- there is no `trait`, `protocol`, or `impl` keyword in the admitted source
  grammar
- `dyn Trait` vtable dispatch does not exist and is not claimed
- published `v1.1.1` does not claim any behavior-contract or trait-based
  abstraction

That baseline remains the source of truth until this subtrack explicitly lands
its widened contract on `main`.

## Included In This Track

- `trait Foo { fn method(&self) -> T; }` syntax for declaring named behavior
  contracts
- `impl Foo for MyType { ... }` syntax for explicit, named impl blocks
- static dispatch only: call-site monomorphic resolution through trait bounds
- trait bounds on type parameters: `<T: Foo>` spelling admitted in first wave
- basic trait object syntax `dyn Foo` is DEFERRED to a later wave
- docs/spec/tests/compatibility wording for the widened contract

## Explicit Non-Goals

- `dyn Trait` / vtable dispatch (deferred to a later wave)
- async traits or `async fn` in trait definitions
- associated types or associated type families
- default method bodies with complex generic constraints
- higher-ranked trait bounds (`for<'a> Fn(&'a T)` style)
- derive macros or procedural macro integration
- orphan rules enforcement in first wave
- trait aliases
- negative impls
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance

- scope checkpoint
- roadmap/milestone/plan linkage

### Wave 1 — Owner Layer

- trait and impl syntax ownership
- behavior-contract and impl-block metadata inventory
- static dispatch and bound-resolution policy boundaries
- explicit typecheck/lowering gap markers before executable admission

### Wave 2 — Source Admission

- parser admission for `trait`, `impl`, and `dyn` keyword stubs
- sema/type admission for trait definitions and impl blocks
- explicit diagnostics for unsupported trait forms

### Wave 3 — Typecheck

- impl resolution at call sites
- trait bound checking on type parameters
- coherence checking (single impl per type/trait pair in first wave)
- verifier compatibility for statically dispatched trait calls

### Wave 4 — Freeze

- docs/spec/tests/compatibility freeze

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint
2. PR 2: owner-layer trait/impl surface
3. PR 3: parser/sema/type admission
4. PR 4: typecheck, impl resolution, and bound checking
5. PR 5: freeze and close-out

## Initial First-Wave Reading

The first-wave trait/protocol contract is intentionally narrow:

- named behavior contracts only; structural polymorphism remains valid outside
  trait bounds
- static dispatch only; no vtable or dynamic dispatch in Wave 1–3
- no `dyn Trait` in first wave
- trait bounds on type parameters admitted; higher-ranked bounds deferred
- one impl per type/trait pair enforced; orphan rules deferred to a later wave
- no default method bodies with complex constraints

That keeps the track additive over the current generic surface without opening
a full abstraction system in one step.

## Acceptance Reading

This track is done only when:

- one first-wave trait/protocol surface is explicit and inspectable
- `trait` definitions, `impl` blocks, and bound-checked call sites agree on
  one deterministic first-wave model
- docs/spec/tests describe the same admitted baseline
- published `v1.1.1` and widened `main` are explicitly distinguished

## Non-Commitments After Close-Out

Even after this first wave lands, the repository still does not claim:

- `dyn Trait` / vtable dispatch or dynamic polymorphism
- async traits or `async fn` in trait definitions
- higher-ranked trait bounds or lifetime-parameterised traits
- associated types, type families, or generic associated types
- derive macros or procedural macro trait expansion
- that traits were already part of the published `v1.1.1` line

## Merge Gate

Before closing this track:

- [ ] code/tests are green
- [ ] spec/docs are synced
- [ ] public API or golden snapshots are updated if needed
- [ ] compatibility/release-facing wording is honest
