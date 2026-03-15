# Standard Library Module Families

Status: proposed v0

## Purpose

This document defines the first intended module-family map for the Semantic
standard library.

It is a roadmap contract for library direction, not a claim that all of these
modules already exist.

## Priority Tiers

### Tier 1: Core Deterministic Utility

These are the first families worth making intentional.

#### `std.math`

Purpose:

- reusable numeric helpers beyond the minimal builtin set
- deterministic wrappers for common numeric operations
- future home for higher-level math helpers that should not become language
  keywords

Examples:

- clamping
- interpolation
- normalization helpers
- aggregate numeric utilities

#### `std.text`

Purpose:

- string and textual-data surface
- formatting, splitting, and simple normalization

Why it matters:

- without text, the language remains too narrow for ordinary general-purpose
  workflows

#### `std.collections`

Purpose:

- canonical collection families once the language has enough data-model support
- likely future home for sequences, maps, and set-like containers

Why it matters:

- collections should be a library story layered on top of the language data
  model, not accidental syntax

#### `std.serde`

Purpose:

- serialization and deserialization for user data
- explicit structured encoding boundaries

Why it matters:

- structured data becomes much more useful once users can exchange it across
  files, tools, and services

### Tier 2: Controlled Runtime Utility

These families are useful, but they require stronger host/effect discipline.

#### `std.time`

Purpose:

- durations
- timestamps
- stable time arithmetic helpers

#### `std.path`

Purpose:

- path manipulation that does not itself perform effects

#### `std.fs`

Purpose:

- file-system access with explicit capability/effect rules

Why Tier 2:

- these modules are more dangerous to standardize casually because they can
  blur the boundary between deterministic library code and host access

### Tier 3: Domain-Oriented Library Families

These families should come later.

Possible examples:

- `std.policy`
- `std.semantic`
- `std.prometheus`

Why later:

- they are valuable, but they should grow from a stable core library model
  rather than define it prematurely

## First Recommended Public Contracts

The first family that should move from roadmap to detailed contract is probably
`std.math`, because:

- the language already has a small numeric builtin set
- the difference between builtin math and library math needs a clear line
- this family can be documented without immediately widening host scope

The second likely family is `std.text`, because strings are one of the biggest
maturity gaps for the language today.

The concrete first-family contract is defined in:

- `docs/roadmap/language_maturity/std_math_surface.md`

## Cross-References

This family map depends on:

- `docs/roadmap/language_maturity/standard_library_surface.md`
- `docs/roadmap/language_maturity/stdlib_boundary.md`
