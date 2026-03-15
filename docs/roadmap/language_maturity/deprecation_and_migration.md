# Deprecation And Migration Policy

Status: proposed v0

## Purpose

This document defines the intended deprecation and migration policy for
Semantic public surfaces.

The goal is to make platform evolution explicit instead of forcing users to
discover breaking changes accidentally.

## Deprecation Rule

Once a surface is marked `stable`, removal should not be the first step.

Expected rule:

- stable surfaces should be deprecated before they are removed or
  fundamentally reinterpreted

The deprecation notice should identify:

- what is deprecated
- what replaces it, if anything
- when removal or stronger change is expected

## Migration Rule

Stable-surface changes should ship with migration guidance.

Expected forms:

- release notes
- compatibility statement updates
- docs updates with before/after examples
- explicit mention in language/library/package policy docs

## Draft And Experimental Surfaces

Draft and experimental surfaces may evolve faster, but they are not exempt from
honesty.

Required rule:

- if a surface is unstable, it must be labeled unstable rather than treated as
  implicitly stable

This is how the platform avoids overpromising while still moving quickly.

## Version Review Rule

Some changes are not merely deprecations; they require explicit version review.

Examples:

- SemCode meaning changes
- profile schema meaning changes
- stable manifest or lockfile meaning changes
- stable stdlib API redefinitions

These changes should not hide behind a vague "cleanup" label.

## Release Notes Rule

Every compatibility-relevant release should explain:

- what changed
- which stability layer it affects
- whether migration is needed
- whether any stable surface was deprecated

## Example Policy Outcomes

### Allowed Without Deprecation

- refining a draft feature that is still clearly labeled draft
- adding a new stable surface alongside existing ones
- expanding experimental tooling without implying stable support

### Requires Deprecation Or Migration Note

- changing stable source syntax meaning
- changing stable JSON CLI output fields
- renaming a stable stdlib API
- changing stable package manifest semantics

## Non-Goals

This first policy does not yet define:

- exact time-based deprecation windows
- LTS release trains
- automated migration tooling

Those may come later, but the repository should first agree on the policy
principle before it promises stronger operational machinery.

## Cross-References

This policy works together with:

- `docs/roadmap/language_maturity/stability_and_compatibility.md`
- `docs/roadmap/language_maturity/compatibility_policy_stack.md`
