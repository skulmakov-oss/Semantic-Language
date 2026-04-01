# Module And Import Specification

Status: draft v0
Primary semantic owners: `sm-sema`, `sm-front`

## Purpose

This document defines the current public module, import, and re-export contract
for Semantic source files.

It supersedes scattered import/export notes as the canonical source-contract
entry for module linkage behavior.

## Current Module Unit

The current module unit is a source file loaded by module identifier.

Current rule:

- imports resolve modules through the active module provider
- module linkage is deterministic and checked before execution

This document does not yet define a package ecosystem or registry story. It
describes only the current file- and module-level source contract.

## Supported Import Forms

Current supported forms:

1. namespace import with implicit alias

```sm
Import "a/b/c"
```

2. namespace import with explicit alias

```sm
Import "a/b/c" as X
```

3. selected symbol import with optional aliases

```sm
Import "a/b/c" { Foo, Bar as Baz }
```

4. wildcard import

```sm
Import "a/b/c" *
```

5. public re-export import

```sm
Import pub "a/b/c" { Foo }
```

## Name Resolution Order

Current effective resolution order:

1. local symbols
2. explicit selected imports as direct local bindings
3. namespace-qualified access such as `X.Foo`
4. wildcard imports in declaration order as fallback for unresolved names

Clarifications:

- local/import alias conflicts are rejected with `E0241` instead of being
  resolved by shadowing
- every `Import` still creates one namespace alias, using either explicit
  `as X` or the default file-stem alias
- selected imports participate in unqualified lookup before wildcard imports
- wildcard imports do not remove namespace-qualified access to the same module

## Export Surface

The current export surface is centered on top-level Logos declarations:

- `System`
- `Entity`
- `Law`

Re-export is supported through `Import pub ...`.

Current export provenance model distinguishes:

- local declarations
- imported declarations
- re-exported declarations

## Determinism Rules

Current determinism rules:

- local export ordering is deterministic by declaration order
- re-exports are appended after local exports in import declaration order
- dependency export order is preserved within each re-exported set
- wildcard resolution follows import declaration order
- symbol-cycle detection is explicit rather than best-effort

## Validation Rules

Current module-surface validation includes:

- duplicate namespace alias rejection (`E0241`)
- missing selected symbol rejection (`E0244`)
- duplicate selected alias rejection (`E0245`)
- selected-import kind mismatch rejection (`E0245`)
- public re-export collision rejection (`E0242`)
- symbol-level re-export cycle rejection (`E0243`)
- invalid wildcard/select combination rejection (`E0245`)

## Current Limits

The current module contract does not yet claim stable support for:

- package manifests
- external registries
- semantic versioned dependency resolution
- lockfiles as part of the public language contract

Those concerns belong to the future package ecosystem surface rather than the
current source module baseline.

## Validation Evidence

Current repository fixtures cover this surface in:

- `tests/fixtures/imports/`
- `tests/import_export_docs_fixtures.rs`

## Contract Rule

Any public change to import syntax, export behavior, resolution order, or
collision/cycle policy should update this document in the same change series.
