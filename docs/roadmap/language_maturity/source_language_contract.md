# Source Language Contract Freeze

Status: proposed

## Goal

Turn the current Semantic source surface into a single canonical language contract rather than a set of scattered implementation notes.

## Why

Semantic already has a strong execution contract. What it still lacks is a fully frozen source-language contract that a user can treat as the primary truth for writing programs.

## Scope

- define canonical source syntax and grammar boundaries
- define expression and statement semantics
- define type rules for `quad`, `bool`, `i32`, `u32`, `f64`, and `fx`
- define import and export behavior as part of the language contract
- define user-facing diagnostics expectations for source-level contract violations

## Non-Goals

- adding new runtime semantics
- widening the PROMETHEUS boundary
- changing bytecode or VM behavior beyond what is required to document the source contract honestly

## Acceptance Criteria

- one canonical source-language specification bundle exists and is linked from the main spec index
- syntax, typing, and module semantics are documented in one intentional contract family
- examples and diagnostics match the documented source surface
- no major source feature remains described only by implementation behavior
