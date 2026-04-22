# Package Manifest

Status: historical design note, not current baseline

## Purpose

This document records one future design candidate for a broader package-manager
manifest layer above the landed first-wave package baseline.

Current-main truth:

- the admitted current-main package baseline is `Semantic.package`, not
  `Semantic.toml`
- this document does not describe the landed `M8.2` package baseline
- if a broader manifest layer is reopened, it requires a new explicit scope
  decision from fresh `main`

## Proposed Manifest Name

One future canonical manifest candidate would be:

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

One future broader manifest wave could include:

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

The first expected value in this design sketch is:

- `v1`

### `entry`

Purpose:

- explicit package entry source for executable packages

Current honest limit:

- library-only packages may eventually use a dedicated library root, but this
  first document does not freeze that shape yet

## `dependencies` Table

This future design sketch would support explicit dependency entries.

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

These tables may come later, but should not be treated as part of the initial
broader manifest wave if it is ever reopened:

- `[dev-dependencies]`
- `[features]`
- `[workspace]`
- target-specific configuration tables

## Non-Goals

This future manifest design does not yet try to define:

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
