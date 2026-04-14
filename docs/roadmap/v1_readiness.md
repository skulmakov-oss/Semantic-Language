# Semantic v1 Readiness

Status: published stable release line

This document summarizes the current release-facing readiness state for Semantic v1.

## Current Readiness Position

Current repository state has working coverage for:

- repository governance and ownership rules
- verified SemCode execution
- runtime quota contract
- canonical profile contract
- IR verification and minimum optimizer pipeline
- PROMETHEUS ABI, capability, and gate boundaries
- PROMETHEUS runtime, state, rules, and audit owner crates
- semantic runtime validation matrix and golden baselines
- tuple + direct record-field runtime ownership, including deterministic
  `BorrowWriteConflict` enforcement and end-to-end regression coverage
- CI-enforced boundary, public API, runtime, and release-bundle gates

This means the repository has crossed from architecture-only planning into a
published stable release line for the current contract surfaces.

Current `v1` boundary decision:

- the official `v1` PROMETHEUS scope is the existing narrow ABI/capability/gate boundary
- wider planned host calls are not part of the current `v1` commitment
- ownership alignment for optimizer, SemCode, and CLI is already implemented in code
- the active stable `v1.1.1` line is published from `main`

## Current Artifact List

Current v1-facing artifact families in the repository:

- architecture bundle
  - `docs/architecture/`
- roadmap bundle
  - `docs/roadmap/milestones.md`
  - `docs/roadmap/type_completeness_matrix.md`
  - `docs/roadmap/runtime_validation_policy.md`
  - `docs/roadmap/release_bundle_checklist.md`
  - `docs/roadmap/compatibility_statement.md`
  - `docs/roadmap/release_asset_smoke_matrix.md`
  - `docs/roadmap/stable_release_policy.md`
- spec bundle
  - `docs/spec/`
- CLI/tooling surface
  - `smc`
  - `svm`
- published stable assets
  - `smc.exe`
  - `svm.exe`
  - Windows release zip
- semantic runtime validation
  - `tests/prometheus_runtime_matrix.rs`
  - `tests/prometheus_runtime_goldens.rs`
  - `tests/prometheus_runtime_negative_goldens.rs`
  - `tests/prometheus_runtime_compat_matrix.rs`

## Current Ready Surfaces

Currently ready or substantially stabilized surfaces:

- `sm-verify`
- verified-only VM execution path
- `sm-runtime-core`
- runtime ownership pipeline for tuple + direct record-field paths
- `sm-profile`
- `sm-ir` verification and minimum optimizer contract
- `prom-abi`
- `prom-cap`
- `prom-gates`
- `prom-runtime`
- `prom-state`
- `prom-rules`
- `prom-audit`

## Current Known Limits

The following limits remain explicit and should be treated as release-facing honesty requirements:

- the published `v1.1.1` line intentionally excludes first-wave plain `fx`
  unary/binary arithmetic, even though current `main` now admits deterministic
  plain `fx` arithmetic with canonical lowering/verified execution under
  `SEMCODE3`
- the published `v1.1.1` line intentionally excludes post-stable PROMETHEUS
  calls such as `StateQuery`, `StateUpdate`, `EventPost`, and `ClockRead`,
  even though current `main` now admits them as a forward-only widened boundary
- the published `v1.1.1` line intentionally excludes first-wave rule-side
  effect execution, even though current `main` now admits narrow declared
  `StateWrite` and `AuditNote` execution
- the published `v1.1.1` line intentionally excludes post-stable persisted
  archive materialization/loading, even though current `main` now admits narrow
  `StateSnapshotArchive` and `AuditReplayArchive` ownership/materialization
- the published `v1.1.1` line intentionally excludes multi-session replay
  archives, even though current `main` now admits narrow
  `MultiSessionReplayArchive` ownership/materialization
- the published `v1.1.1` line intentionally excludes rollback persistence
  semantics, even though current `main` now admits narrow
  `StateRollbackArtifact` ownership and deterministic
  `SemanticStateStore::apply_rollback(...)`
- the published `v1.1.1` line intentionally excludes executable `text`, even
  though current `main` now admits first-wave text literals/equality through
  canonical `SEMCODE8`, verifier admission, and VM execution
- the published `v1.1.1` line intentionally excludes the first-wave package
  ecosystem baseline, even though current `main` now admits `Semantic.package`
  parsing, package entry-module admission, and deterministic local-path
  dependency loading for package-qualified imports
- the published `v1.1.1` line intentionally excludes the first-wave ordered
  sequence collection surface, even though current `main` now admits
  `Sequence(type)`, bracketed literals, same-family equality, `expr[index]`,
  and canonical verified execution through `SEMCODE9`
- the published `v1.1.1` line intentionally excludes the first-wave first-class
  closure surface, even though current `main` now admits `Closure(T -> U)`,
  standalone closure literals, immutable capture, direct invocation, and
  canonical verified execution through `SEMCODE10`
- the published `v1.1.1` line intentionally excludes the first-wave generics
  surface, even though current `main` now admits type-parameter syntax for
  functions, records, and ADTs, and deterministic call-site monomorphisation
  under the narrow `TypeVar`-to-concrete substitution model
- the published `v1.1.1` line intentionally excludes the completed runtime
  ownership track on current `main`, even though `main` now admits tuple +
  direct record-field `AccessPath` transport, verifier admission, frame-local
  borrow tracking, deterministic `BorrowWriteConflict` rejection, and e2e
  regression coverage; unsupported scope remains explicit for ADT payload
  paths, schema paths, partial release, aliasing graphs, inter-frame
  persistence, and indirect projections
- the published `v1.1.1` line intentionally excludes the first-wave UI
  application boundary, even though current `main` now admits single-window
  session ownership, deterministic event polling, frame-token ownership, and
  the minimal `DrawCommand`/`DrawFrame` family as exercised by `prom-ui-demo`
- current `main` still does not claim rollback, retry/compensation, or generic
  mixed-family rule-effect execution semantics
- current `main` still does not claim rollback, migration, recovery, or
  runtime replay engine semantics for persisted archives
- current `main` still does not claim implicit coercion into `fx`, `fx[unit]`
  arithmetic, or full arithmetic parity between `fx` and `f64`
- current `main` still does not claim rollback artifact text materialization,
  crash-resume, inter-session repair, or generic transaction semantics
- final stable packaging and tag policy remain narrower than the long-term distribution plan

## Current Release Gate

The repository should be treated as release-valid only if all of the following stay green:

- `cargo test --workspace`
- boundary and ownership guard tests
- `cargo test --test public_api_contracts`
- `pwsh -File scripts/verify_release_bundle.ps1 -ManifestPath <path>`
- semantic runtime matrix tests
- semantic runtime golden tests
- semantic runtime negative golden tests
- semantic runtime compatibility matrix tests

## Next Release Maintenance Steps

Current highest-signal remaining work after the first stable `v1.1.1` tag:

1. keep release-facing docs aligned with the published stable line on `main`
2. rerun representative asset smoke for every forward release tag
3. keep narrow `v1` limits explicit unless a separate scope decision promotes them
4. treat any future widening as a forward versioned release, not silent drift

## Contract Rule

No document in this readiness summary should be used to silently overstate completeness.

If a surface is only partially complete, it must remain listed under `Current Known Limits` until tests, docs, and behavior all align.
