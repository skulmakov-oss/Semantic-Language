# Record Data Model

Status: implemented stage-2 v0

## Purpose

This document defines the first stabilized aggregate family currently available
in the Semantic executable language surface: nominal records.

It is no longer only a design target. It records the narrow stage-2 contract
that now exists across parser, sema, IR, verifier, and VM.

## Why Records First

Records are the right first aggregate family because they:

- model ordinary domain state directly
- preserve explicit field naming
- remain deterministic to lower
- do not require hidden dynamic dispatch
- compose naturally with existing `quad` and numeric values

## Canonical Stage-2 Surface

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

Stage-2 ergonomics are now available as sugar over the same nominal slot-based
carrier:

```sm
let ctx = DecisionContext { camera, badge, quality, override_state };
let next = ctx with { quality };
let DecisionContext { camera, quality: _ } = ctx;
let DecisionContext { camera: T, quality } = next else return;
```

## Type Identity

Current rules:

- record types are nominal, not structural
- record field order is part of layout, but not the public identity of the
  type
- field names must be unique within one record declaration
- an empty record type is not part of the stabilized stage-1 surface

## Value Semantics

Current stage-2 rules:

- record values are immutable after construction
- record construction requires every declared field exactly once
- no implicit default field values
- copy-with rebuilds a value of the same nominal record type
- no field mutation syntax exists in the stable surface
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

Records still do not reopen a full pattern system.

Current stage-2 rule:

- statement-level record destructuring is supported through nominal
  `RecordName { field: target }`
- record `let-else` is supported through nominal
  `RecordName { field: target } = value else return ...;`
- record punning is sugar only inside canonical nominal record forms
- record-pattern matching in `match` arms remains out of scope
- anonymous brace-only record forms such as `let { x, y } = value;` remain out
  of scope

## Lowering Strategy

The current stage-2 record implementation preserves the deterministic VM story.

Current strategy:

- lower each record value into a fixed compile-time field layout
- rewrite source field order into canonical declaration-slot order
- keep record layout statically known to the compiler
- avoid introducing a general heap object model

That means the first record family behaves more like a named product type than
like a dynamic object.

## IR Implications

The stage-2 implementation keeps the IR change set narrow.

Current direction:

- explicit record-type metadata exists in the frontend and sema layers
- record construction and copy-with lower through canonical `MakeRecord`
- field access and destructuring lower through deterministic `RecordGet`
- generic dictionary-like runtime operations remain out of scope

The contract goal remains explicit: records lower to predictable slots, not
opaque runtime blobs.

## VM And Verifier Implications

Stage-2 records do not abandon the verified execution model.

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
- record pattern matching in `match`
- anonymous brace-only record forms
- nested record patterns
- record-specific generics
- heap allocation semantics as a user-visible contract

## Example Workloads

The record family already supports stage-2 scenarios such as:

- access-policy contexts
- structured sensor snapshots
- semantic decision envelopes
- grouped runtime configuration inputs

These are more representative of real user data than scalar-only local packs,
while staying within the current verified-path contract.

## Acceptance Criteria

The Stage-2 record family should be considered properly frozen for v0 when the
repository has:

- a stable canonical source form
- explicit construction, access, destructuring, copy-with, and punning-shorthand rules
- explicit equality and non-goals
- a documented deterministic lowering direction
- examples showing why records are better than scalar-only decomposition
- scenario workloads that compile through the verified path
