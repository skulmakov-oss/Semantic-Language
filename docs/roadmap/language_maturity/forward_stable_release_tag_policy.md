# Forward Stable Release Tag Policy

Status: completed stable-line governance checkpoint
Related milestone: `M6 v1 Lockdown`

## Purpose

Keep the forward-only stable release and tag policy explicit after the published
`v1.1.1` release, without reopening release scope or rewriting history.

## Current Stable Reading

- published stable tags must remain immutable
- prerelease and stable lines move forward only
- the next stable tag after a beta line must use the next non-conflicting
  forward version
- release/tag policy is governed by:
  - `docs/roadmap/stable_release_policy.md`
  - `docs/roadmap/language_maturity/release_version_cut_decision.md`

## Why This Checkpoint Exists

The `v1.1.1` release already applied the policy in practice, but the roadmap
layer should state explicitly that this is now frozen release discipline rather
than a remaining follow-up item.

## Honest Boundary

This checkpoint does not:

- create a new release
- widen package/versioning scope beyond the documented repository release model
- reopen beta-only wording
- relax the no-history-rewrite rule
