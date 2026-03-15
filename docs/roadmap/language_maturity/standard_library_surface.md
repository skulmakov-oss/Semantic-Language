# Standard Library Surface

Status: proposed

## Goal

Define the first intentional standard-library surface for Semantic beyond the current math- and runtime-oriented builtins.

## Why

A language cannot approach Rust or Python maturity without a usable standard library. Today Semantic has execution primitives, but not a broad user-facing library contract.

## Scope

- identify the first stable library families
- define string, collection, filesystem, time, and serialization priorities
- distinguish language builtins from standard-library modules
- define stability and compatibility rules for standard-library APIs

## Non-Goals

- shipping every possible library family in one wave
- hiding runtime limits behind implicit host effects
- treating external integrations as if they were already stable stdlib

## Acceptance Criteria

- one standard-library roadmap exists with prioritized families
- language builtins and stdlib modules are clearly separated
- at least one library family has a documented public contract and examples
- library compatibility expectations are stated explicitly
