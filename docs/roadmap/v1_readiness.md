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

- the canonical `fx` value path is end-to-end, but `fx` remains a narrow
  value-transport/equality family rather than a full arithmetic peer of `f64`
- wider planned PROMETHEUS calls such as `StateQuery`, `StateUpdate`, `EventPost`, and `ClockRead` are intentionally excluded from the current `v1`
- semantic runtime covers activation/orchestration glue, but not full rule-side effect execution
- persistence backends are not part of the current runtime/audit contract
- rollback persistence semantics are not yet formalized beyond current orchestration notes
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
