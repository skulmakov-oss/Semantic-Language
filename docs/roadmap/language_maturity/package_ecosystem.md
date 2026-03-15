# Package And Dependency Ecosystem

Status: proposed v0

## Goal

Move Semantic from file- and module-level imports toward an intentional package
and dependency story.

This workstream is about reproducible project structure, dependency
declaration, and publishing discipline. It is not about replacing the current
module/import contract.

## Current Baseline

Today Semantic has:

- source-level file/module imports
- deterministic module resolution and validation
- import/export/re-export policy
- no package manifest
- no lockfile
- no dependency resolver as a public user-facing contract

Current honest constraint:

- file imports are useful, but they are not yet a package ecosystem

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
- `docs/roadmap/language_maturity/dependency_resolution.md`

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

This workstream should be considered materially started only when:

- one canonical package model exists in docs
- import/module behavior is clearly separated from package/dependency behavior
- project metadata and lockfile expectations are documented
- publishing and version-compatibility policy are stated explicitly

## Immediate Next Slice

The immediate next slice for this PR is to define:

- the package manifest
- dependency-resolution rules
- lockfile expectations

because those three pieces are the minimum coherent package story.
