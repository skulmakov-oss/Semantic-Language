# Semantic v1 Release Bundle Checklist

Status: draft v0

Use this checklist before assembling a release-facing v1 bundle.

## Required Documentation Bundle

Verify the bundle includes:

- `docs/architecture/`
- `docs/spec/`
- `docs/roadmap/v1_readiness.md`
- `docs/roadmap/runtime_validation_policy.md`
- `docs/roadmap/compatibility_statement.md`

## Required Contract Surfaces

Verify the release documents the current state of:

- SemCode header family and verifier rule
- ParserProfile contract and hash semantics
- VM quota and verified-only execution rule
- PROMETHEUS ABI, capability, and gate boundaries
- semantic runtime orchestration, state, rules, and audit owner split

## Required Test Gates

Verify these are green before the bundle is considered releasable:

- `cargo test --workspace`
- `cargo test --test public_api_contracts`
- `cargo test --test golden_semcode`
- `cargo test --test prometheus_runtime_matrix`
- `cargo test --test prometheus_runtime_goldens`
- `cargo test --test prometheus_runtime_negative_goldens`
- `cargo test --test prometheus_runtime_compat_matrix`

## Required Artifact Notes

Verify the release notes include:

- currently stabilized surfaces
- known limits that still block a final v1 tag
- explicit snapshot regeneration rule
- compatibility-sensitive contract families

## Blocking Rule

Do not mark the bundle release-ready if:

- any known limit was silently dropped from the docs
- compatibility-sensitive tests were not run
- runtime snapshots were regenerated without review
- readiness and compatibility documents disagree with actual repository behavior
