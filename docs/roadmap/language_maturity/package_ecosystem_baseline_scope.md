# Package Ecosystem Baseline Scope

Status: completed M8.2 first-wave post-stable subtrack
Related roadmap package:
`docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`

## Goal

Introduce the first package-level contract for Semantic without silently
widening the published `v1.1.1` line and without turning the repository into a
full registry or package-manager project.

This is a forward-only language-maturity subtrack for current `main`. It is not
a claim that package manifests or dependency resolution already exist on the
published stable line.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- the module/import contract is file-based and module-identifier based
- current imports resolve through the active module provider rather than
  through a package manifest boundary
- the public source contract does not yet expose package manifests
- the public source contract does not yet expose registries, semver dependency
  solving, or lockfiles

That baseline remains the source of truth until this subtrack explicitly lands
its widened contract on `main`.

## Included In This Track

- explicit ownership of a package manifest surface
- deterministic package identity and package-root rules
- local path dependency declaration and validation
- explicit mapping between package roots and existing module resolution
- deterministic package graph loading for admitted first-wave dependencies
- docs/spec/tests/compatibility wording for the widened contract

## Explicit Non-Goals

- remote registries
- package publishing workflow
- semver range solving
- lockfiles as part of the first-wave public contract
- vendoring or global cache design
- build scripts or native dependency toolchains
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance

- scope checkpoint
- roadmap/milestone/plan linkage

### Wave 1 — Owner Layer

- manifest schema ownership
- package identity ownership
- package-root and dependency inventory

### Wave 2 — Source Admission

- manifest parsing
- dependency declaration validation
- module/package relationship admission

### Wave 3 — Resolution Path

- deterministic local path dependency loading
- graph validation and narrow CLI/module-provider integration
- explicit non-commitment for lockfiles in the first-wave baseline

### Wave 4 — Freeze

- docs/spec/tests/compatibility freeze

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint
2. PR 2: manifest/package identity owner-layer surface
3. PR 3: dependency declaration and module/package admission
4. PR 4: deterministic local path resolution baseline
5. PR 5: freeze and close-out

## Close-Out Reading

`M8.2` is now completed on current `main` as a first-wave package baseline.

Completed first-wave contract:

- canonical `Semantic.package` parsing and validation
- explicit package identity, package-root, and dependency inventory ownership
- package entry-module admission against admitted `module_root`
- deterministic local-path dependency loading for package-qualified imports
  such as `"alias::module/path.sm"`
- one canonical resolution path shared by `smc-cli`, `sm-sema`, and the
  incremental module graph

Still intentionally not included after close-out:

- registry or publishing semantics
- semver or lockfile ownership
- workspace/package-manager orchestration beyond deterministic local-path
  loading
- package solver heuristics or alternate dependency sources

## Acceptance Reading

This subtrack is done only when:

- package identity and manifest semantics are explicit and inspectable
- package dependency declarations and module loading agree on one admitted
  first-wave model
- the first admitted dependency graph is deterministic and reproducible
- published `v1.1.1` and widened `main` are explicitly distinguished

## Non-Commitments After Close-Out

Even after this first wave lands, the repository still does not claim:

- remote package registries
- semver dependency solving
- lockfile stability guarantees
- build-script or native-toolchain package hooks
- package publishing/distribution workflows
