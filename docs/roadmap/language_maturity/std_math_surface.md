# `std.math` Surface

Status: proposed v0

## Purpose

This document defines the first concrete standard-library family proposed for
Semantic: `std.math`.

It is a library-contract design target, not a claim that the current parser,
module loader, or VM already ships this module.

## Why `std.math` First

`std.math` is the best first stdlib family because:

- the language already exposes a narrow numeric builtin set
- there is a clear need for reusable numeric helpers beyond that builtin set
- it stays within deterministic, pure computation
- it does not require widening host/effect scope

This makes it the lowest-risk first family for turning "stdlib" into a real
public library surface.

## Boundary With Builtins

Current builtins such as:

- `sin`
- `cos`
- `tan`
- `sqrt`
- `abs`
- `pow`

remain language-adjacent primitives.

`std.math` should not duplicate them blindly. Its role is broader reusable
helpers that are better expressed as imported library functions than as magical
reserved names.

## Proposed Module Name

The canonical first stable name should be:

- `std.math`

Example import style:

```sm
Import "std/math" as math
```

The package/manifest story is out of scope for this document; this is only the
intended public library family name and shape.

## First-Wave Surface

The first-wave `std.math` surface should stay small and focused.

Recommended initial families:

- scalar transforms
- comparison helpers
- interpolation and normalization helpers
- aggregate helpers for small numeric sets

### Proposed First-Wave Functions

#### `clamp`

```sm
math.clamp(x: f64, min: f64, max: f64) -> f64
```

Purpose:

- bound a value into a closed numeric range

#### `min`

```sm
math.min(a: f64, b: f64) -> f64
```

#### `max`

```sm
math.max(a: f64, b: f64) -> f64
```

#### `lerp`

```sm
math.lerp(start: f64, end: f64, t: f64) -> f64
```

Purpose:

- deterministic interpolation helper for policy scoring and sensor smoothing

#### `normalize`

```sm
math.normalize(value: f64, min: f64, max: f64) -> f64
```

Purpose:

- explicit range normalization into a stable numeric interval

#### `approx_eq`

```sm
math.approx_eq(a: f64, b: f64, eps: f64) -> bool
```

Purpose:

- explicit floating comparison helper

This is preferable to encouraging users to pretend floating equality is always
exactly what they want.

#### `mean2` / `mean3`

```sm
math.mean2(a: f64, b: f64) -> f64
math.mean3(a: f64, b: f64, c: f64) -> f64
```

Purpose:

- small deterministic aggregate helpers without waiting for collection support

## Type Scope

The first `std.math` wave should be explicit about type scope.

Recommended phase-1 rule:

- `std.math` is stabilized first around `f64`

Why:

- `f64` is the broadest current numeric family with stable arithmetic and
  builtin support
- `fx` is still intentionally narrower
- generic numeric overloading would be premature

## Purity And Determinism

The `std.math` family should be pure.

Expected rules:

- no hidden host interaction
- no clock, random, filesystem, or process access
- deterministic output for the same input values
- no capability manifest requirements for the first wave

## Non-Goals

The first `std.math` wave should not attempt to include:

- random number generation
- matrix or tensor packages
- symbolic algebra
- statistics-heavy modules
- effectful numeric APIs
- broad generic numeric polymorphism

These may become later library work, but they should not be bundled into the
first stabilized family.

## Example Workloads

### Policy Scoring

```sm
let score = math.clamp(signal, 0.0, 1.0);
let confidence = math.lerp(0.2, 1.0, score);
```

### Sensor Comparison

```sm
if math.approx_eq(sensor_a, sensor_b, 0.001) {
    return T;
} else {
    return N;
}
```

### Compact Aggregate Numeric Logic

```sm
let center = math.mean3(a, b, c);
let normalized = math.normalize(center, 0.0, 100.0);
```

## Compatibility Expectations

Once `std.math` is stabilized:

- module name `std.math` should not silently drift
- published function names should remain stable
- changing argument order or return type is a compatibility break
- moving a helper from stdlib into builtin should require explicit version
  review

## Acceptance Criteria

The first concrete stdlib family should be considered properly designed when:

- the repository has a canonical `std.math` family document
- builtin-vs-stdlib math boundaries are explicit
- at least one small initial function set is proposed
- examples show clear value beyond today's builtin-only numeric surface
