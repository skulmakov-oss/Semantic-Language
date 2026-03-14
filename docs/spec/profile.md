# ParserProfile Specification

Status: draft v0
Owner crate: `sm-profile`
Primary consumers: `sm-front`, `sm-sema`, `sm-ir`, `smc`

## Purpose

`ParserProfile` is the canonical policy contract for language-surface
acceptance and producer policy.

Contract rule:

- profile defines what is allowed to be parsed and produced
- SemCode header defines what was actually produced
- verifier proves the produced artifact is consistent with that contract

`ParserProfile` is not an embedded runtime contract and is not a second SemCode
metadata layer.

## Canonical Ownership

`ParserProfile` lives in `sm-profile`.

The following are architectural violations:

- duplicate profile schema outside `sm-profile`
- hidden parser defaults deep inside frontend or sema entrypoints
- support or legacy modules acting as a second profile owner

## Current Schema

Current public fields:

- `identity`
- `version`
- `abi`
- `compatibility`
- `features`
- `capabilities`
- `aliases`

Current policy subdomains:

- `AbiProfile`
- `CompatibilityMode`
- `FeaturePolicy`
- `CapabilityExpectations`

## Current Version Policy

Current default public profile version:

- major `1`
- minor `0`

Contract rule:

- incompatible schema or meaning changes require a major version bump
- backward-compatible additions or clarifications require a minor version bump

## Current Default Profile

The current public baseline profile is `ParserProfile::foundation_default()`.

Important rule:

- defaults must be chosen at the public entry boundary
- deeper parser and semantic stages must not silently invent their own hidden
  default profile

## Policy Scope

Current profile policy governs:

- whether debug-symbol-oriented emission is allowed
- whether `f64` surface is allowed
- whether gate-surface constructs are allowed
- whether Logos surface is allowed
- whether legacy-compatible surface branches are accepted

This is compile-time policy, not runtime authority.

## Capability Expectations

`CapabilityExpectations` expresses profile-level expectations and restrictions.

Important rule:

- profile expectations do not replace the actual SemCode capability contract
- capability bits in the produced artifact must still be derived from actual
  usage

That means:

- profile may allow more than a specific program uses
- producer must not emit extra capability claims just because the profile
  allows them

## Compatibility Mode Rule

`CompatibilityMode` affects surface acceptance only.

It may permit legacy syntax or aliases.

It must not weaken:

- verifier admission
- runtime isolation
- capability enforcement
- SemCode safety guarantees

## Serialization Rule

The canonical serialized form is the JSON schema used by `sm-profile`.

Serialization requirements:

- deterministic roundtrip
- no silent semantic field loss
- version fields preserved
- alias map preserved

Any schema change requires:

1. update this specification
2. update `sm-profile` roundtrip tests
3. update user-facing validation behavior if public semantics changed

## No Silent Mutation Rule

The following are forbidden without version review:

- changing the meaning of an existing feature flag
- changing the meaning of an existing compatibility mode
- changing ABI policy interpretation
- changing serialized schema semantics while pretending the version is unchanged
