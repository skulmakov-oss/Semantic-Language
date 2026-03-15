# Package And Dependency Ecosystem

Status: proposed

## Goal

Move Semantic from file- and module-level imports toward an intentional package and dependency story.

## Why

Rust and Python are ecosystems as much as languages. Without package resolution, versioning, publishing, and reproducible dependency management, Semantic remains a toolchain rather than a full language platform.

## Scope

- define the package unit for Semantic projects
- define dependency resolution and lockfile expectations
- define publishing and registry expectations
- define compatibility rules for third-party packages

## Non-Goals

- shipping a public registry immediately
- pretending file imports are already equivalent to a package ecosystem
- coupling package resolution to unstable runtime semantics

## Acceptance Criteria

- one canonical package model exists in docs
- import/module behavior is clearly separated from package/dependency behavior
- project metadata and lockfile expectations are documented
- publishing and version-compatibility policy are stated explicitly
