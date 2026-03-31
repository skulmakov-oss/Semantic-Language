# Semantic v1 Release Bundle Checklist

Status: active stable release baseline

Use this checklist before assembling or publishing a release-facing stable or
prerelease `v1` bundle.

## Required Documentation Bundle

Verify the bundle includes:

- `docs/architecture/`
- `docs/spec/`
- `docs/roadmap/v1_readiness.md`
- `docs/roadmap/runtime_validation_policy.md`
- `docs/roadmap/compatibility_statement.md`
- `docs/roadmap/release_asset_smoke_matrix.md`
- `docs/roadmap/stable_release_policy.md`
- published asset notes for `smc.exe`, `svm.exe`, and the Windows zip when a GitHub release is cut

Reproducible check command:

- `pwsh -File scripts/verify_release_bundle.ps1 -ManifestPath <path>`

## Required Contract Surfaces

Verify the release documents the current state of:

- SemCode header family and verifier rule
- ParserProfile contract and hash semantics
- VM quota and verified-only execution rule
- PROMETHEUS ABI, capability, and gate boundaries
- semantic runtime orchestration, state, rules, and audit owner split

## Required Test Gates

Verify these are green before the bundle is considered releasable:

- CI must run these as explicit release-facing jobs, not only via broad workspace test aggregation
- `.github/workflows/ci.yml` should include `boundary-enforcement`, `public-api-guard`, `runtime-release-gates`, and `release-bundle-process`
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
- known limits that remain explicit non-commitments for the current narrow `v1`
- explicit snapshot regeneration rule
- compatibility-sensitive contract families
- which packaged assets were published for the current tag

## Required Asset Smoke

Verify published assets are checked against at least:

- one minimal compile-run-disasm source
- one verified-path `f64` builtin case
- one representative semantic policy example from `examples/`

## Blocking Rule

Do not mark the bundle release-ready if:

- any known limit was silently dropped from the docs
- compatibility-sensitive tests were not run
- runtime snapshots were regenerated without review
- readiness and compatibility documents disagree with actual repository behavior
