# Collections Surface Full Scope

Status: active M8.3 post-stable subtrack
Related roadmap package:
`docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`

## Goal

Introduce the first admitted collection surface for Semantic without silently
widening the published `v1.1.1` line and without opening full abstraction
machinery ahead of schedule.

This is a forward-only language-maturity subtrack for current `main`. It is not
a claim that first-class collections already exist on the published stable
line.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- there are no first-class collection carriers in the public language contract
- tuple and record surfaces exist, but they do not imply general-purpose
  sequence or lookup collections
- current syntax/types docs still treat collections as outside the admitted
  stable subset
- published `v1.1.1` does not claim list, map, set, or iterable abstractions

That baseline remains the source of truth until this subtrack explicitly lands
its widened contract on `main`.

## Included In This Track

- explicit ownership of one first-wave ordered sequence collection family
- deterministic construction/literal surface for that collection family
- narrow indexing and same-family equality semantics
- deterministic runtime carrier for the admitted sequence family
- docs/spec/tests/compatibility wording for the widened contract

## Explicit Non-Goals

- map/dictionary carriers
- set carriers
- user-defined generic collections
- collection traits, protocols, or iterator frameworks
- comprehensions, lazy pipelines, or query-language syntax
- mutation-heavy collection APIs beyond the first admitted baseline
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance

- scope checkpoint
- roadmap/milestone/plan linkage

### Wave 1 — Owner Layer

- collection family ownership
- literal and operation inventory
- deterministic value-model metadata

### Wave 2 — Source Admission

- parser admission
- sema/type admission
- explicit diagnostics for unsupported collection forms

### Wave 3 — Execution Path

- IR/lowering/VM carrier path
- deterministic iteration/index behavior
- docs/tests/goldens for the admitted baseline

### Wave 4 — Freeze

- docs/spec/tests/compatibility freeze

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint
2. PR 2: owner-layer collection family surface
3. PR 3: parser/sema/type admission
4. PR 4: runtime/VM iteration and indexing baseline
5. PR 5: freeze and close-out

## Current Wave Reading

Current branch scope for Wave 2:

- admit `Sequence(type)` in declared source type positions
- admit bracketed ordered sequence literals in the Rust-like source path
- admit same-family equality for ordered sequence values when the item type
  already supports stable equality
- keep runtime lowering/indexing/iteration explicitly outside the Wave 2 slice

Still intentionally not included in Wave 2:

- indexing or length operations
- iteration syntax or iterable loops
- runtime carrier details or VM lowering
- collection mutation policy beyond the first admitted baseline

Current branch scope for Wave 3:

- canonical runtime carrier for admitted ordered sequence values
- deterministic `expr[index]` execution with `i32` index operands
- same-family verified runtime equality for admitted ordered sequence values
- promoted `SEMCODE9` / `CAP_SEQUENCE_VALUES` contract for programs that
  actually require the widened sequence carrier

Still intentionally not included in Wave 3:

- `len` / `is_empty` surface
- `for value in sequence` or any general iterable loop story
- maps, sets, or generic collection abstractions
- host-ABI widening for sequence values

## Acceptance Reading

This subtrack is done only when:

- one ordered sequence collection family is explicit and inspectable
- collection construction, indexing, and equality agree on one deterministic
  first-wave model
- docs/spec/tests describe the same admitted baseline
- published `v1.1.1` and widened `main` are explicitly distinguished

## Non-Commitments After Close-Out

Even after this first wave lands, the repository still does not claim:

- general-purpose map/set ecosystems
- user-defined parametric collection types
- generic iterator/protocol frameworks
- lazy collection pipelines or comprehensions
- that collection support was already part of the published `v1.1.1` line
