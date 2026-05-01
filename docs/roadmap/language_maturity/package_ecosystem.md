# Package And Dependency Ecosystem

Status: historical design note, not current baseline

Current-main truth:

- the first-wave package baseline is already completed on current `main` in
  `docs/roadmap/language_maturity/package_ecosystem_baseline_scope.md`
- the admitted current-main package contract is the landed `Semantic.package`
  baseline with deterministic local-path dependency loading
- this document is retained only as a broader future design note; it does not
  describe the current admitted package baseline or the published stable line

## Goal

Sketch one possible broader package-manager direction beyond the landed
`Semantic.package` baseline.

This document is about a future manifest/lockfile/versioning/publishing layer.
It is not about replacing the current module/import contract, and it is not a
claim that this broader package-manager layer is active today.

## Current Baseline

Today current `main` has:

- source-level file/module imports
- deterministic module resolution and validation
- import/export/re-export policy
- canonical `Semantic.package` parsing and validation
- package entry-module admission
- deterministic local-path dependency loading for package-qualified imports
- no lockfile
- no public registry/publishing contract
- no lockfile-backed package-manager contract

Current honest constraint:

- current `main` has a first-wave package baseline, but it is not yet a full
  package-manager ecosystem

## Why This Matters

Rust and Python are ecosystems as much as languages.

Without package resolution, versioning, publishing, and reproducible dependency
management, Semantic remains a toolchain rather than a full language platform.

Users currently lack a stable answer to questions like:

- what is one project/package unit
- how a project declares external dependencies
- how dependency versions are pinned
- how published libraries are named and consumed

## Design Principles

The package model should preserve the current repository discipline.

Required principles:

- deterministic resolution
- explicit dependency declaration
- lockfile-backed reproducibility
- separation between source module syntax and package metadata
- no silent network dependency during ordinary local builds
- compatibility with the existing import/module contract

## Staged Plan

### Stage 1: Local Package Model

The first package wave should define:

- one package manifest format
- one lockfile format
- local path dependencies
- workspace-style multi-package development

This is the smallest useful step because it makes projects reproducible before
inventing a public registry.

### Stage 2: Published Package Identity

After local packages, the next step should define:

- canonical package naming
- semantic version expectations
- publishable package metadata
- package compatibility policy

### Stage 3: Registry And Remote Resolution

Only after package identity is stable should the repository define:

- registry indexing
- remote fetch policy
- cache behavior
- trust/security model for published packages

## Core Documents

The immediate package-ecosystem design is anchored in:

- `docs/roadmap/language_maturity/package_manifest.md`
- `docs/roadmap/language_maturity/package_lockfile.md`
- `docs/roadmap/language_maturity/dependency_resolution.md`
- `docs/roadmap/language_maturity/package_worked_example.md`

## Relationship To Imports

Packages do not replace modules.

Current intended rule:

- module imports remain the source-level linkage form
- package metadata determines which dependency roots are available to the
  module resolver
- package and dependency semantics sit above the existing import surface rather
  than erasing it

That keeps `docs/imports.md` and the future package model complementary instead
of contradictory.

## Non-Goals

This workstream is not intended to:

- ship a public registry immediately
- pretend file imports are already equivalent to a package ecosystem
- couple package resolution to unstable runtime semantics
- make remote networking mandatory for normal project builds

## Acceptance Criteria

Any future reopening of this broader package-manager design should be
considered materially started only when:

- one canonical package model exists in docs
- import/module behavior is clearly separated from package/dependency behavior
- project metadata and lockfile expectations are documented
- publishing and version-compatibility policy are stated explicitly

## Current Decision

- no active implementation slice is currently open from this document
- any reopening of manifest/lockfile/versioning/registry work requires a new
  explicit scope decision from fresh `main`
- until that happens, the current admitted package truth remains the completed
  `Semantic.package` baseline and its close-out docs
