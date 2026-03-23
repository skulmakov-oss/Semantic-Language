# Record Scenarios

Status: validated against stage-1 v0

## Purpose

This document gives concrete user-data scenarios that justify records as the
first aggregate family for Semantic and tracks which parts are already covered
by the current stage-1 implementation.

The goal is to show real workloads that are awkward in the current scalar-only
surface and become clearer with nominal records.

These examples are no longer only design targets. The stage-1 contract now
supports nominal record declarations, construction, field access, pass/return,
and equality inside the stable field-equality subset.

## Why Scenarios Matter

Without scenarios, "expand user data model" can stay abstract for too long.

Records need to prove that they:

- reduce scalar sprawl
- improve readability
- preserve deterministic reasoning
- compose well with `quad`, numeric signals, and policy logic

## Scenario 1: Access Policy Context

Current scalar-only style tends to spread one logical decision across many
locals:

```sm
let camera_state: quad = T;
let badge_state: quad = N;
let override_state: quad = N;
let tamper_state: quad = F;
let quality: f64 = 0.50;
```

This works, but the source no longer has one named value that represents "the
current decision context".

The stage-1 record form is clearer:

```sm
record DecisionContext {
    camera: quad,
    badge: quad,
    override_state: quad,
    tamper: quad,
    quality: f64,
}

fn allow(ctx: DecisionContext) -> quad {
    if ctx.tamper == T || ctx.tamper == S {
        return S;
    }
    if ctx.override_state == T {
        return T;
    }
    if ctx.camera == T && ctx.badge == T {
        return T;
    }
    return N;
}
```

Why this matters:

- the policy takes one domain object instead of five unrelated parameters
- field names remain explicit
- `quad`-oriented semantics stay first-class inside the aggregate

Stage-1 validation status:

- compiles through the verified path
- runs through the VM with explicit field access and ordinary control flow

## Scenario 2: Sensor Snapshot

Current numeric and semantic signals often belong to one measurement moment.

Scalar-only code forces users to keep this grouped mentally:

```sm
let sensor_a: f64 = 0.51;
let sensor_b: f64 = 0.49;
let camera_state: quad = T;
let badge_state: quad = T;
```

A record lets the source express that this is one snapshot:

```sm
record SignalSnapshot {
    sensor_a: f64,
    sensor_b: f64,
    camera: quad,
    badge: quad,
}

fn trusted(snapshot: SignalSnapshot) -> bool {
    return snapshot.camera == T && snapshot.badge == T;
}
```

Why this matters:

- numeric and semantic fields stay grouped
- helper functions can accept one meaningful argument
- later field additions do not explode parameter lists immediately

Stage-1 validation status:

- compiles through the verified path
- field access resolves deterministically by declaration-slot order

## Scenario 3: Rule Input Envelope

Semantic is good at deterministic reasoning, but today rule inputs still need
to be unpacked manually.

A record allows a clean input envelope:

```sm
record RuleInput {
    source: quad,
    quality: quad,
    manual_override: quad,
}

fn final_state(input: RuleInput) -> quad {
    if input.manual_override == S {
        return S;
    }
    if input.source == T && input.quality == T {
        return T;
    }
    return N;
}
```

Why this matters:

- rule signatures become more stable
- grouped policy inputs read like one contract
- the code documents intent without relying on naming conventions alone

Stage-1 validation status:

- supported as a nominal input object
- compatible with pass/return through the verified execution path

## Scenario 4: Runtime Configuration Bundle

Not every structured value is a policy object. Some are just grouped execution
inputs.

Example:

```sm
record RuntimeConfig {
    max_steps: u32,
    debug_mode: bool,
    fallback_state: quad,
}
```

Why this matters:

- this is ordinary user data, not a Logos entity
- users need a source-level way to group config values without pretending they
  are semantic declarations

Current limit:

- record values are still not part of the PROMETHEUS host ABI surface
- config bundles may participate in verified-local execution, not host calls

## Comparison With Logos Entities

It is important not to confuse records with `Entity`.

`Entity` remains:

- a declarative Logos construct
- domain metadata for the Logos surface
- part of the system/rule description layer

Records are currently:

- ordinary executable values
- usable in Rust-like functions
- suitable for passing, returning, comparing, and field access

That distinction keeps the executable language model cleaner.

## First-Stage Success Criteria

The first record stage is worthwhile when users can stop writing
policy-shaped scalar packs like:

- `camera_state`
- `badge_state`
- `override_state`
- `tamper_state`
- `quality`

and instead pass one explicit domain value with named fields.

## Explicit Non-Goals Still In Force

The current stage-1 implementation still does not include:

- record destructuring
- record update / copy-with
- record punning
- methods, inheritance, or dynamic dispatch
- host ABI passage for record values

## Cross-References

This scenario set supports:

- `docs/roadmap/language_maturity/user_data_model.md`
- `docs/roadmap/language_maturity/record_data_model.md`
