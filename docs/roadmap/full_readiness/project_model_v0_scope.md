# FR-4 Project Model v0 Scope

Status: proposed readiness scope  
Parent: Semantic Full Readiness — Non-UI Track

## Goal

Define a minimal Semantic project model so a user can create, check, and run a small multi-file Semantic project through a documented path.

This document scopes the model. It does not implement the manifest, CLI commands, or resolver changes.

## Minimal layout

```text
semantic.toml
src/
  main.sm
  lib.sm
examples/
tests/
```

## Design principles

- Project identity must be explicit.
- Entry behavior must be deterministic.
- Local module resolution must not depend on ambient working-directory accidents.
- The model must not conflict with the existing import/export surface.
- Package/registry behavior is out of scope for v0.

## Work packages

### FR-4.1 — define `semantic.toml` v0

Scope:

- package name;
- package version or explicit non-commitment;
- source root;
- entrypoint;
- feature/capability profile hook if needed;
- toolchain/version expectations if needed.

Acceptance:

- manifest fields are documented;
- unknown field policy is specified;
- invalid manifest diagnostics are deterministic.

### FR-4.2 — define package identity

Acceptance:

- package identity is stable enough for diagnostics, cache keys, and examples;
- identity does not imply registry publication.

### FR-4.3 — define `src/main.sm` and `src/lib.sm`

Acceptance:

- executable project entrypoint behavior is explicit;
- library root behavior is explicit;
- missing/duplicate entrypoints fail predictably.

### FR-4.4 — define local dependency paths

Acceptance:

- local imports resolve relative to project/package rules;
- path normalization policy is explicit;
- resolver behavior remains deterministic.

### FR-4.5 — define examples discovery

Acceptance:

- examples can be discovered or are explicitly manual;
- example entrypoints are documented.

### FR-4.6 — define tests discovery

Acceptance:

- test directory semantics are either admitted or explicitly deferred;
- no accidental test framework is implied.

### FR-4.7 — implement `smc new` follow-up

Acceptance for follow-up:

- creates the minimal layout;
- generated project checks successfully;
- no UI/workbench files are generated.

### FR-4.8 — implement `smc check project` follow-up

Acceptance for follow-up:

- checks manifest and project sources;
- reports manifest/source diagnostics separately.

### FR-4.9 — implement `smc run project` follow-up

Acceptance for follow-up:

- runs the declared entrypoint through verified execution where applicable;
- rejects ambiguous entrypoints.

### FR-4.10 — document project model

Acceptance:

- project guide exists;
- Getting Started can reference one canonical path.

## Out of scope

- package registry;
- dependency lockfile unless separately scoped;
- remote dependencies;
- version solver;
- build scripts;
- UI application model;
- Workbench-specific project metadata.

## Definition of Done

FR-4 is complete when a user can follow one documented project layout and the CLI has a clear path to create, check, and run that project without relying on undocumented conventions.
