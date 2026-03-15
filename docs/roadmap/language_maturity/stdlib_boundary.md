# Builtin Versus Standard Library Boundary

Status: proposed v0

## Purpose

This document defines the intended boundary between language builtins and
standard-library modules in Semantic.

Without this boundary, the language risks turning every useful helper into a
builtin and every host interaction into accidental "library convenience".

## Builtins

Builtins should stay very small and language-adjacent.

A function belongs in the builtin set only when at least one of these is true:

- it is tightly coupled to core expression typing
- it is required for the minimal executable language to stay usable
- verifier/VM support is intentionally explicit for that operation

Current examples:

- `sin`
- `cos`
- `tan`
- `sqrt`
- `abs`
- `pow`

These are not a general library; they are part of the currently stabilized
numeric call surface.

## Standard Library Modules

A capability belongs in stdlib rather than in builtins when it is:

- reusable across many programs
- not syntax-level in nature
- better expressed as an imported API than as a magical reserved function
- able to evolve as a module family with documented compatibility rules

Typical examples:

- string helpers
- collection helpers
- serialization helpers
- higher-level math helpers

## Host And Effect Boundaries

Not every helpful function should be stdlib.

Some capabilities remain outside ordinary stdlib until the repository defines a
stable effect and capability story for them.

Examples:

- file I/O
- clock access
- network access
- environment and process APIs

These are not banned forever, but they should not silently appear as if they
were ordinary pure helpers.

## Decision Rules

When deciding whether something should be a builtin or a stdlib module, use
this order:

1. If it changes expression typing or minimal execution meaning, prefer builtin.
2. If it is reusable library functionality with no special syntax role, prefer
   stdlib.
3. If it crosses a host/effect boundary, require an explicit capability-aware
   design before calling it stdlib.

## Non-Goals

This boundary is not trying to:

- freeze the entire future builtin set forever
- forbid future host-aware libraries
- force every helper into a module before the language is ready

It is only trying to prevent category drift while the language matures.

## Immediate Consequence

For the current maturity wave:

- keep the existing math builtins narrow
- treat broader numeric helpers as future `std.math`
- treat strings, collections, and serde as stdlib candidates
- delay effectful modules until their capability story is explicit
