# `Semantic.lock`

Status: historical design note, not current baseline

## Purpose

This document records one future design candidate for a package-manager
lockfile above the landed first-wave package baseline.

Current-main truth:

- the admitted current-main package baseline has no public lockfile contract
- this document does not describe current `main`; it describes a broader future
  design that would require a new explicit scope decision

It is the reproducibility companion to `Semantic.toml`. The manifest declares
intent; the lockfile records the concrete resolved graph used for one build.

## Canonical Name

One future canonical lockfile candidate would be:

- `Semantic.lock`

The lockfile should live at the package root next to `Semantic.toml`.

## Why A Lockfile Exists

The first package story should be reproducible before it becomes networked or
ecosystem-heavy.

The lockfile exists to:

- pin concrete dependency selections
- make CI and local builds agree on the same graph
- make version drift explicit rather than silent
- separate user intent from resolver output

## Manifest Versus Lockfile

The contract between the two files should be explicit:

- `Semantic.toml` declares package metadata and dependency requirements
- `Semantic.lock` records the concrete resolved dependency graph
- ordinary builds consume the lockfile rather than silently changing it
- lockfile updates should happen only through explicit resolver/update flows

## Proposed Top-Level Shape

```toml
version = 1
root = "access-policy"

[[package]]
name = "access-policy"
version = "0.1.0"
source = "path:."

[[package]]
name = "mathx"
version = "0.1.0"
source = "path:../mathx"

[[package]]
name = "policy-core"
version = "0.2.3"
source = "registry:semantic"
checksum = "sha256:abc123"
```

This shape is intentionally small. It is not trying to freeze a workspace,
feature, or registry wire protocol yet.

## Required Fields

One future lockfile wave could include:

- top-level `version`
- top-level `root`
- one `[[package]]` entry for every resolved package in the graph

Each `[[package]]` entry should include:

- `name`
- `version`
- `source`

Optional later fields may include:

- `checksum`
- `dependencies`
- `edition`

## Source Forms

One future lockfile wave could support these source forms:

- `path:.`
- `path:../relative-path`
- `registry:<name>`

The purpose is to make source origin explicit without freezing a registry
transport too early.

## Determinism Rules

Any future lockfile contract should obey these rules:

- the same manifest plus lockfile resolves to the same package graph
- the same lockfile may be used on different machines without changing package
  identity
- path dependencies remain local and explicit
- registry dependencies are pinned to concrete versions in the lockfile

## Mutation Rules

If this future lockfile layer is ever admitted, its stable behavior should be
conservative:

- ordinary compile/check/run commands should not silently rewrite
  `Semantic.lock`
- explicit dependency-update flows may rewrite the lockfile
- manual edits to the lockfile are not considered a supported workflow

## Conflict And Validation Rules

The first lockfile layer in this design sketch should reject:

- duplicate package entries with the same identity
- missing root package entry
- malformed or unknown source forms
- dependency graphs that disagree with manifest aliases or package identity

## Non-Goals

This future lockfile document does not define:

- a workspace lockfile grammar
- feature activation state
- registry authentication data
- mirrors or source overrides
- full transitive dependency edge syntax

## Cross-References

This lockfile contract works together with:

- `docs/roadmap/language_maturity/package_ecosystem.md`
- `docs/roadmap/language_maturity/package_manifest.md`
- `docs/roadmap/language_maturity/dependency_resolution.md`
- `docs/roadmap/language_maturity/package_worked_example.md`
