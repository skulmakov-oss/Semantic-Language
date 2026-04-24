# Semantic Stable Release Policy

Status: release-governance policy for the published stable line

Read this document using the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

Read this policy together with:

- `docs/release_artifact_model.md`

This policy governs:

- the published stable line

It does not automatically promote:

- landed-on-`main` behavior
- or the current qualified limited-release contour

## Current Stable Reading

The current published stable line is:

- `v1.1.1`

Current practical-programming qualification is separate and remains:

- `qualified limited release`

Those are distinct decisions.

## Scope Freeze Rule

While maintaining or validating the stable line:

- do not silently widen the stable promise
- do not treat landed-on-`main` work as stable by default
- do not reopen broader feature scope through release-maintenance PRs

Allowed stable-line work:

- release-facing docs sync
- release asset validation
- packaging fixes
- narrow correctness fixes that are rerun through the full validation contour

## Stable Tag Preconditions

A stable tag or stable-line refresh is allowed only if all relevant release
validation remains green, including:

- workspace tests
- boundary and ownership guards
- public API compatibility checks
- release bundle verification
- release asset smoke verification
- release-facing docs matching actual repository behavior

## Promotion Rule

Behavior should be described as `published stable` only when:

- the stable line explicitly promises it
- supporting release assets and validation cover it

Landed behavior on current `main` remains unpromoted until an explicit later
decision promotes it.

## Publish Rule

Stable release notes should state:

- the exact released commit
- the artifact model for the published asset set
- the validated asset set
- the stable-ready surfaces
- the known limits that remain outside the stable promise

## Non-Commitments

The following remain outside the stable-release critical path unless explicitly
promoted later:

- broader practical-programming widening beyond the current stable promise
- broader executable-module authoring
- UI
- broader runtime and ecosystem work already landed on `main` but not yet
  promoted
