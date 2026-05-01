# Release Version Cut Decision

Status: applied stable-tag checkpoint

## Goal

Choose the next honest public stable version after the completed `v0.1`,
`v0.2`, and `v0.3` implementation waves, without rewriting published stable
history or overstating asset validation.

## Current Validated Repository Commit

The current release-gate-validated repository commit is:

- `c9e5470`

At this commit the following gates were rerun and passed:

- `cargo test --workspace`
- `cargo test --test public_api_contracts`
- `cargo test --test golden_semcode`
- `cargo test --test prometheus_runtime_matrix`
- `cargo test --test prometheus_runtime_goldens`
- `cargo test --test prometheus_runtime_negative_goldens`
- `cargo test --test prometheus_runtime_compat_matrix`
- `pwsh -File scripts/verify_release_bundle.ps1 -ManifestPath artifacts/release/post_v03_release_bundle_manifest.json`

## Existing Published Version Markers

Relevant existing tags already in repository history:

- `v0.1.0`
- `v1.0.0`
- `v1.1.0-beta.1`
- `v1.1.1-beta2`
- `v1.1.1-beta3`
- `v1.1.1-beta4`

## Decision

The next honest forward stable tag candidate is:

- `v1.1.1`

## Why This Is The Correct Forward Version

- `v1.0.0` already exists as a published stable tag.
- the active prerelease line already advanced past `v1.1.0-beta.1` and into
  `v1.1.1-beta2..beta4`
- `docs/roadmap/stable_release_policy.md` requires:
  - no rewriting or force-moving published stable tags
  - the first stable tag after a beta line to use a non-conflicting stable
    version
  - the next forward version when an older stable tag already exists

Given those constraints, `v1.1.1` is the smallest forward stable version that
matches the current prerelease line and does not collide with published stable
history.

## Cargo Package Version Note

The workspace `Cargo.toml` package version still reads `0.1.0`.

That is not treated here as the authority for the next repository release tag.
This checkpoint covers the Git tag and published release decision for the
repository bundle, not a crates.io publication policy or a separate package
ecosystem versioning wave.

If package-manifest/version alignment is later formalized as a separate
release requirement, it should be handled explicitly instead of being silently
folded into this stable-tag decision.

## Remaining Blocking Step Before Final Stable Cut

This blocker was cleared before publish.

The release-facing blocker that remained at checkpoint time was:

- rerun `docs/roadmap/release_asset_smoke_matrix.md` against downloaded assets
  built for the exact candidate tag `v1.1.1`

That smoke validation was later completed against the published `v1.1.1`
assets before the stable release was published.

## Acceptance Reading

This checkpoint is useful only if:

- the next stable version is stated explicitly as `v1.1.1`
- the reason for rejecting earlier or conflicting stable tags is documented
- package-version mismatch is acknowledged rather than silently ignored
- downloaded-asset validation remains an explicit blocker until rerun on the
  exact candidate tag
