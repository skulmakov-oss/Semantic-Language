# Record Data Model

Status: proposed v0

## Purpose

This document defines the first intentional aggregate family proposed for the
Semantic executable language surface: nominal records.

It is a design target for the user-data-model workstream, not a claim that the
current parser or VM already implements this feature.

## Why Records First

Records are the right first aggregate family because they:

- model ordinary domain state directly
- preserve explicit field naming
- remain deterministic to lower
- do not require hidden dynamic dispatch
- compose naturally with existing `quad` and numeric values

## Proposed Surface

The proposed source form is a nominal `record` declaration:

```sm
record DecisionContext {
    camera: quad,
    badge: quad,
    quality: f64,
    override: quad,
}
```

Construction is explicit:

```sm
let ctx = DecisionContext {
    camera: T,
    badge: N,
    quality: 0.50,
    override: N,
};
```

Field access is explicit:

```sm
ctx.camera
ctx.quality
```

## Type Identity

Proposed rules:

- record types are nominal, not structural
- record field order is part of layout, but not the public identity of the
  type
- field names must be unique within one record declaration
- an empty record type should not be part of the first stabilized surface

## Value Semantics

Proposed phase-1 rules:

- record values are immutable after construction
- record construction requires every declared field exactly once
- no implicit default field values
- no field mutation syntax in the first stage
- passing a record to a function passes the value as one logical source object

## Equality

Phase-1 equality should be explicit and narrow.

Proposed rules:

- record equality is allowed only when every field type already supports
  stable equality
- record equality returns `bool`
- there is no custom user-defined equality hook in the first stage

## Pattern And Match Scope

Records should not immediately force a full pattern system.

Phase-1 rule:

- record destructuring and record-pattern matching are out of scope
- users access fields explicitly rather than through destructuring syntax

This keeps the first aggregate family coherent without forcing a broad match
expansion in the same change wave.

## Record Punning Gate

`record punning` is intentionally not part of the first canonical record layer.

For the `Density-20 Plus` backlog this means `D20P-D01` stays blocked until all
of the following are true:

- nominal `record` declarations are part of the executable source contract
- record construction with explicit field names is implemented end-to-end
- explicit field access lowers through a stable deterministic slot model
- the repository has a separate decision reopening record destructuring syntax

Until then, forms such as:

```sm
Point { x, y }
let { x, y } = point;
```

must remain out of scope. The first record wave should keep field names explicit
and should not reopen pattern/destructuring ergonomics by accident.

## Lowering Strategy

The first record implementation should preserve the current deterministic VM
story.

Preferred initial strategy:

- lower each record value into a fixed compile-time field layout
- flatten field storage during IR lowering rather than introducing a general
  heap object model
- keep record layout statically known to the compiler

That means the first record family should behave more like a named product type
than like a dynamic object.

## IR Implications

The first implementation should aim for the smallest IR change compatible with
clear semantics.

Preferred direction:

- add explicit record-type metadata in the frontend and sema layers
- lower field access into deterministic field-slot operations
- avoid generic dictionary-like runtime operations

The exact IR opcode strategy is intentionally left open, but the contract goal
is clear: records should lower to predictable slots, not opaque runtime blobs.

## VM And Verifier Implications

Phase-1 records should not require abandoning the verified execution model.

Expected rules:

- verifier must be able to validate record-related layout references
- VM should execute record access through deterministic, bounded operations
- records must not become a hidden escape hatch around SemCode validation

## Quad Composition

Records should work naturally with the language's core semantic strengths.

Example:

```sm
record SignalSnapshot {
    camera: quad,
    badge: quad,
    quality: f64,
}

fn trusted(snapshot: SignalSnapshot) -> bool {
    return snapshot.camera == T && snapshot.badge == T;
}
```

This keeps `quad` as a first-class field family instead of forcing users to
spread semantically related values across parallel locals.

Concrete motivating workloads are collected in:

- `docs/roadmap/language_maturity/record_scenarios.md`

## Non-Goals

This phase does not attempt to provide:

- mutable structs
- inheritance
- methods or dynamic dispatch
- record destructuring patterns
- record-specific generics
- heap allocation semantics as a user-visible contract

## Example Workload

The record family should unlock examples like:

- access-policy contexts
- structured sensor snapshots
- semantic decision envelopes
- grouped runtime configuration inputs

These are more representative of real user data than today's scalar-only local
packs.

## Acceptance Criteria

The Stage 1 record family should be considered properly designed when the
repository has:

- a stable proposed source form
- explicit construction and access rules
- explicit equality and non-goals
- a documented deterministic lowering direction
- examples showing why records are better than scalar-only decomposition
