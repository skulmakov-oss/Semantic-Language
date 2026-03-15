# Package Manifest

Status: proposed v0

## Purpose

This document defines the first intended package manifest contract for
Semantic projects.

It is a design target for the package ecosystem workstream, not a claim that
the current CLI already parses this file.

## Proposed Manifest Name

The first canonical manifest file should be:

- `Semantic.toml`

Reason:

- it is explicit and user-facing
- it follows the repository naming style of using full product identity for
  top-level user-facing artifacts
- it avoids overloading source-module files with project metadata concerns

## Package Unit

A package is the smallest publishable and dependency-addressable Semantic
project unit.

A package should contain:

- one manifest file
- one source root
- zero or more local modules
- optional examples/tests/docs

## Proposed Top-Level Shape

```toml
[package]
name = "access-policy"
version = "0.1.0"
edition = "v1"
entry = "src/main.sm"

[dependencies]
mathx = { path = "../mathx" }
policy_core = { version = "^0.2.0" }
```

## `package` Table

The first manifest wave should include:

- `name`
- `version`
- `edition`
- `entry`

Optional later metadata may include:

- `description`
- `license`
- `authors`
- `repository`

### `name`

Rules:

- package names are globally meaningful user-facing identifiers
- names should be lowercase with `-` separators
- dependency aliases may differ from package names at use sites

### `version`

Rules:

- package versions follow semantic versioning expectations
- published compatibility promises are made against package version

### `edition`

Purpose:

- edition marks the intended language/library compatibility wave for the
  package surface

The first expected value is:

- `v1`

### `entry`

Purpose:

- explicit package entry source for executable packages

Current honest limit:

- library-only packages may eventually use a dedicated library root, but this
  first document does not freeze that shape yet

## `dependencies` Table

The first manifest wave should support explicit dependency entries.

Proposed dependency forms:

- local path dependency
- versioned published dependency

Example:

```toml
[dependencies]
mathx = { path = "../mathx" }
policy_core = { version = "^0.2.0" }
```

The dependency key is the local package alias exposed to the resolver.

## Future Tables

These tables may come later, but should not be treated as part of the first
stabilized manifest wave:

- `[dev-dependencies]`
- `[features]`
- `[workspace]`
- target-specific configuration tables

## Non-Goals

This first manifest design does not yet try to define:

- a full workspace grammar
- registry authentication
- source replacement or mirrors
- feature resolution
- build scripts

## Cross-References

This manifest design works together with:

- `docs/roadmap/language_maturity/package_ecosystem.md`
- `docs/roadmap/language_maturity/dependency_resolution.md`
- `docs/roadmap/language_maturity/package_lockfile.md`
- `docs/roadmap/language_maturity/package_worked_example.md`
