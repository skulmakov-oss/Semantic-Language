# Record Data Model

Status: implemented stage-1 v0

## Purpose

This document defines the first stabilized aggregate family currently available
in the Semantic executable language surface: nominal records.

It is no longer only a design target. It records the narrow stage-1 contract
that now exists across parser, sema, IR, verifier, and VM.

## Why Records First

Records are the right first aggregate family because they:

- model ordinary domain state directly
- preserve explicit field naming
- remain deterministic to lower
- do not require hidden dynamic dispatch
- compose naturally with existing `quad` and numeric values

## Canonical Stage-1 Surface

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

Current rules:

- record types are nominal, not structural
- record field order is part of layout, but not the public identity of the
  type
- field names must be unique within one record declaration
- an empty record type is not part of the stabilized stage-1 surface

## Value Semantics

Current stage-1 rules:

- record values are immutable after construction
- record construction requires every declared field exactly once
- no implicit default field values
- no field mutation syntax in the first stage
- passing a record to a function passes one nominal logical value through the
  verified execution path

## Equality

Stage-1 equality is explicit and narrow.

Current rules:

- record equality is allowed only when every field type already supports
  stable equality
- record equality returns `bool`
- there is no custom user-defined equality hook in the first stage

## Pattern And Match Scope

Records do not yet reopen a full pattern system.

Current stage-1 rule:

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

The current stage-1 record implementation preserves the deterministic VM story.

Current strategy:

- lower each record value into a fixed compile-time field layout
- rewrite source field order into canonical declaration-slot order
- keep record layout statically known to the compiler
- avoid introducing a general heap object model

That means the first record family behaves more like a named product type than
like a dynamic object.

## IR Implications

The stage-1 implementation keeps the IR change set narrow.

Current direction:

- explicit record-type metadata exists in the frontend and sema layers
- record construction lowers through canonical `MakeRecord`
- field access lowers through deterministic `RecordGet`
- generic dictionary-like runtime operations remain out of scope

The contract goal remains explicit: records lower to predictable slots, not
opaque runtime blobs.

## VM And Verifier Implications

Stage-1 records do not abandon the verified execution model.

Current rules:

- verifier validates record-related layout references
- VM executes record access through deterministic, bounded operations
- records do not become a hidden escape hatch around SemCode validation

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

## Example Workloads

The record family already supports stage-1 scenarios such as:

- access-policy contexts
- structured sensor snapshots
- semantic decision envelopes
- grouped runtime configuration inputs

These are more representative of real user data than scalar-only local packs,
while staying within the current verified-path contract.

## Acceptance Criteria

The Stage-1 record family should be considered properly frozen for v0 when the
repository has:

- a stable canonical source form
- explicit construction and access rules
- explicit equality and non-goals
- a documented deterministic lowering direction
- examples showing why records are better than scalar-only decomposition
- scenario workloads that compile through the verified path
