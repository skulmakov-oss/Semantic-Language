# Semantic v1 Readiness

Status: draft v0

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

This means the repository has crossed from architecture-only planning into release-shaped validation for the current contract surfaces.

Current `v1` boundary decision:

- the official `v1` PROMETHEUS scope is the existing narrow ABI/capability/gate boundary
- wider planned host calls are not part of the current `v1` commitment

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
- spec bundle
  - `docs/spec/`
- CLI/tooling surface
  - `smc`
  - `svm`
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

- richer `fx` arithmetic is not yet aligned with the broader `f64` numeric surface, even though the canonical `fx` value path is now end-to-end
- wider planned PROMETHEUS calls such as `StateQuery`, `StateUpdate`, `EventPost`, and `ClockRead` are intentionally excluded from the current `v1`
- semantic runtime covers activation/orchestration glue, but not full rule-side effect execution
- persistence backends are not part of the current runtime/audit contract
- rollback persistence semantics are not yet formalized beyond current orchestration notes
- release packaging is not yet assembled into a final bundled distribution process

## Current Release Gate

The repository should be treated as v1-candidate only if all of the following stay green:

- `cargo test --workspace`
- `pwsh -File scripts/verify_release_bundle.ps1 -ManifestPath <path>`
- semantic runtime matrix tests
- semantic runtime golden tests
- semantic runtime negative golden tests
- semantic runtime compatibility matrix tests

## Next Remaining v1 Steps

Current highest-signal remaining work before a final v1 tag:

1. tighten remaining `fx` and `i32` numeric contract notes after landing the canonical `fx` value path
2. complete ownership implementation alignment after the owner decisions already taken for `v1`
3. raise CI and release enforcement to match the planned `M6` strength
4. tighten release bundle and compatibility statement
5. formalize any remaining runtime rollback/replay constraints that must be public
6. freeze artifact and CLI release packaging expectations

## Contract Rule

No document in this readiness summary should be used to silently overstate completeness.

If a surface is only partially complete, it must remain listed under `Current Known Limits` until tests, docs, and behavior all align.
