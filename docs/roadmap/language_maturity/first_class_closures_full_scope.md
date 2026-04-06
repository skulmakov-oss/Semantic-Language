# First-Class Closures Full Scope

Status: active M8.4 post-stable subtrack
Related roadmap package:
`docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`

## Goal

Introduce the first admitted first-class closure surface for Semantic without
silently widening the published `v1.1.1` line and without opening generics,
traits, or async/runtime machinery ahead of schedule.

This is a forward-only language-maturity subtrack for current `main`. It is not
a claim that first-class closures already exist on the published stable line.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- short lambdas exist only as immediate-call or pipeline sugar
- short lambdas are not first-class values in the public language contract
- short lambdas are capture-free in the current stable source contract
- the published `v1.1.1` line does not claim closure values, closure types, or
  closure runtime carriers

That baseline remains the source of truth until this subtrack explicitly lands
its widened contract on `main`.

## Included In This Track

- explicit ownership of one first-wave closure value family
- a narrow closure type spelling for admitted source positions
- deterministic immutable capture policy for admitted closure values
- direct invocation of admitted closure values
- local binding, parameter, and return transport for admitted closures
- docs/spec/tests/compatibility wording for the widened contract

## Explicit Non-Goals

- multi-argument closure syntax
- async closures or coroutine semantics
- mutable capture or by-reference capture semantics
- trait/protocol-based callable abstractions
- generic closure signatures or higher-kinded callable machinery
- host-ABI widening for closure values
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance

- scope checkpoint
- roadmap/milestone/plan linkage

### Wave 1 — Owner Layer

- closure family ownership
- closure type/literal/capture metadata inventory
- deterministic closure value-model boundaries
- explicit typecheck/lowering gap diagnostics before executable admission

### Wave 2 — Source Admission

- parser admission
- sema/type admission
- explicit diagnostics for unsupported closure forms

### Wave 3 — Execution Path

- lowering/runtime/VM closure carrier path
- deterministic immutable capture materialization
- direct invocation path for admitted closure values

### Wave 4 — Freeze

- docs/spec/tests/compatibility freeze

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint
2. PR 2: owner-layer closure family surface
3. PR 3: parser/sema/type admission
4. PR 4: runtime/VM closure carrier and invocation path
5. PR 5: freeze and close-out

## Initial First-Wave Reading

The first-wave closure contract is intentionally narrow:

- one parameter only
- one closure value family only
- immutable capture only
- direct call syntax only
- no trait-based callable overloading

That keeps the track additive over the current short-lambda sugar without
turning it into a general abstraction system in one step.

Current Wave 1 reading on `main`:

- current `main` now owns one first-wave closure family in the frontend owner
  layer
- current `main` does not yet admit first-class closures in executable parser or
  source-typing positions
- current `main` surfaces explicit Wave 1 gap diagnostics instead of silently
  widening parsing or runtime behavior

## Acceptance Reading

This subtrack is done only when:

- one first-wave closure family is explicit and inspectable
- closure values, immutable capture, and direct invocation agree on one
  deterministic first-wave model
- docs/spec/tests describe the same admitted baseline
- published `v1.1.1` and widened `main` are explicitly distinguished

## Non-Commitments After Close-Out

Even after this first wave lands, the repository still does not claim:

- generic callable abstractions
- trait/protocol-based closure polymorphism
- async/generator closure semantics
- multi-argument or variadic closure families
- that first-class closures were already part of the published `v1.1.1` line
