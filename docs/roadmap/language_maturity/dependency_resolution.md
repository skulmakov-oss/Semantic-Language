# Dependency Resolution

Status: historical design note, not current baseline

## Purpose

This document records one future design candidate for dependency resolution and
lockfile behavior above the landed first-wave package baseline.

Current-main truth:

- current `main` already admits deterministic local-path dependency loading
  through `Semantic.package`
- this document does not describe that landed baseline; it sketches a broader
  future layer that would require a new explicit scope decision
- packages and modules are still distinct concepts in both the landed baseline
  and any future expansion

## Resolution Layers

This future package-manager sketch has three distinct layers:

1. package manifest declares dependency roots
2. resolver maps those dependency roots to concrete package sources
3. existing module/import logic resolves modules inside those roots

This preserves the current source import model while giving it a real package
context.

## Lockfile

One future canonical lockfile candidate would be:

- `Semantic.lock`

Purpose:

- pin concrete dependency versions and sources
- make builds reproducible across machines and CI runs
- prevent silent dependency drift

## Resolution Rules

Any future package-manager resolver should be deterministic.

Expected rules:

- dependency roots must be declared explicitly in `Semantic.toml`
- the same manifest and lockfile must resolve to the same package graph
- local path dependencies resolve without network access
- versioned dependencies resolve through locked versions once a lockfile exists
- ordinary builds should not silently rewrite the lockfile

## Import Relationship

Packages should expose roots to the existing import surface rather than replace
it outright.

Intended rule:

- imports remain quoted module paths
- dependency aliases provide import roots for those paths

Example direction:

```sm
Import "policy_core/rules"
Import "mathx/stats"
```

This is an ecosystem-level rule layered above module resolution, not a new
claim about runtime semantics.

## Path Dependencies

The first supported dependency source in this future sketch should be local
paths.

Reason:

- path dependencies are enough to make workspace-style development real
- they avoid forcing a registry before the package model is stable
- they make the lockfile/reproducibility story testable early

## Versioned Dependencies

Versioned dependencies would belong to the broader package contract even before
a public registry is fully implemented, if this design is ever reopened.

Expected rules:

- manifests may declare version requirements
- lockfile records the concrete chosen version
- changing the resolved version without lockfile update is not allowed silently

## Conflict Policy

Any future dependency story should prefer clarity over cleverness.

Expected rules:

- duplicate package aliases in one manifest are invalid
- ambiguous dependency roots are invalid
- incompatible version requirements should fail resolution explicitly
- the package graph should remain acyclic at the package layer unless the
  repository later defines a deliberate exception

## Publishing Relationship

Resolution policy and publishing policy are related but distinct.

This document only fixes how dependencies are selected and pinned. Registry
protocol, index format, and publish workflow belong to a later stage of the
ecosystem workstream.

## Non-Goals

This future resolution contract does not yet define:

- remote registry wire protocol
- mirrors
- authenticated private registries
- feature unification
- sophisticated override tables

## Cross-References

This resolution contract depends on:

- `docs/roadmap/language_maturity/package_ecosystem.md`
- `docs/roadmap/language_maturity/package_manifest.md`
- `docs/roadmap/language_maturity/package_lockfile.md`
- `docs/roadmap/language_maturity/package_worked_example.md`
