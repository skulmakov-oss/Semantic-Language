# Semantic Stable Release Policy

Status: beta-to-stable release rule

This document defines how the current beta line is allowed to move to a stable release without reopening scope.

## Scope Freeze

While the repository is on an active beta line:

- do not add new ABI calls
- do not widen the current PROMETHEUS `v1` scope
- do not expand runtime semantics beyond the current narrow contract
- do not turn post-`v1` items into release blockers

Allowed changes during the beta-to-stable window:

- release-facing docs sync
- release asset validation
- packaging fixes
- emergency correctness fixes that are rerun through the full validation contour

## Stable Tag Preconditions

A stable tag is allowed only if all of the following remain true on `main`:

- `cargo test --workspace` is green
- boundary and ownership guard tests are green
- public API inventory is green
- runtime matrix, goldens, negative goldens, and compatibility matrix are green
- `pwsh -File scripts/verify_release_bundle.ps1` is green
- published release assets pass the smoke matrix in `docs/roadmap/release_asset_smoke_matrix.md`
- readiness and compatibility documents match actual repository behavior

## Tag Rule

- do not rewrite or force-move published stable tags
- beta tags may advance as forward-only prerelease markers
- the first stable tag after a beta line must use a non-conflicting stable version
- if any older stable tag already exists, choose the next forward version rather than rewriting history

## Publish Rule

The stable release notes should include:

- the exact source commit on `main`
- the validated asset set
- current ready surfaces
- explicit known limits that remain outside the stable commitment

## Non-Commitments

The following remain outside the stable-release critical path unless explicitly promoted by a separate decision:

- richer `fx` arithmetic beyond the current value path
- wider PROMETHEUS host-call families
- persistence and replay backends
- richer rule-side effect execution semantics
- broad naming or branding rewrites
