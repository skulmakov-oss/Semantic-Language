# FR-7 / FR-8 / FR-9 Examples, Onboarding, and Release Qualification Scope

Status: proposed readiness scope  
Parent: Semantic Full Readiness — Non-UI Track

## Goal

Close the public-facing proof path for Semantic readiness through canonical examples, external onboarding, and reproducible release qualification.

This document scopes the final readiness gates. It does not implement examples, docs, tests, or release automation.

## FR-7 — Canonical Examples

### Purpose

Prove the language through small, stable, documented programs.

### Required example families

- hello/text;
- quad decision;
- record and match;
- ADT / Option / Result;
- sequence processing;
- map lookup if map is admitted;
- module imports;
- contracts with `requires` / `ensures` / `invariant`;
- deterministic pseudo-random flow if `rand` is admitted;
- rule/state decision;
- one benchmark-class non-UI program.

### Work packages

#### FR-7.1 — define canonical examples policy

Acceptance:

- each example has status: stable, qualified limited, current-main, or experimental;
- examples avoid unsupported forms unless intentionally negative;
- examples are small enough for onboarding.

#### FR-7.2 — create canonical examples set

Acceptance:

- 10 to 12 example families are present or explicitly deferred;
- each example has a README or short explanation;
- each example maps to one or more language features.

#### FR-7.3 — add check/run smoke plan

Acceptance:

- each runnable example has expected command path;
- check/run status is explicit;
- failing examples are not presented as canonical positives.

#### FR-7.4 — add expected output / golden plan

Acceptance:

- deterministic outputs are captured where useful;
- examples do not depend on nondeterministic host state.

## FR-8 — External Onboarding

### Purpose

Allow a strong external engineer to clone, understand, build, and run Semantic without private project knowledge.

### Required docs

- Getting Started;
- Language Tour;
- Semantic by Example;
- Project Model Guide;
- CLI Guide;
- Diagnostics / explain guide;
- Troubleshooting;
- Release Status.

### Work packages

#### FR-8.1 — Getting Started

Acceptance:

- clone/build/check/run path is documented;
- required toolchain is explicit;
- failure modes are listed.

#### FR-8.2 — Language Tour

Acceptance:

- major surface forms are introduced in a practical order;
- stable vs current-main status is not blurred.

#### FR-8.3 — Semantic by Example

Acceptance:

- examples are linked from one entrypoint;
- each example explains purpose, command, expected behavior.

#### FR-8.4 — CLI Guide

Acceptance:

- check/build/run/verify/explain/doctor or their admitted equivalents are documented;
- unsupported commands are not implied.

#### FR-8.5 — Diagnostics and troubleshooting

Acceptance:

- common diagnostics have human-readable explanations;
- troubleshooting separates source errors, verifier rejection, runtime traps, and host/capability denial.

## FR-9 — Release Qualification

### Purpose

Convert readiness into a reproducible release candidate.

### Required gates

- workspace build;
- standard tests;
- no-std checks where applicable;
- verifier tests;
- runtime gates;
- public API guard;
- boundary enforcement;
- examples smoke;
- docs consistency;
- release bundle process.

### Work packages

#### FR-9.1 — define release qualification checklist

Acceptance:

- gate list is explicit;
- command list is explicit;
- failure response is defined.

#### FR-9.2 — run full CI matrix

Acceptance:

- all required jobs are green;
- known non-blocking jobs are explicitly labeled.

#### FR-9.3 — run examples smoke

Acceptance:

- canonical examples are checked or run by script/manual checklist;
- outputs are stable or documented as non-output examples.

#### FR-9.4 — run release bundle process

Acceptance:

- release artifact path is reproducible;
- release notes match shipped behavior.

#### FR-9.5 — freeze compatibility statement

Acceptance:

- stable/current-main distinction remains honest;
- release promise does not absorb experimental surfaces.

#### FR-9.6 — tag release candidate follow-up

Acceptance:

- tag policy is documented;
- release candidate can be reproduced from tag and instructions.

## Out of scope

- UI application demos;
- Workbench readiness;
- GitHub Linguist PR;
- public release claim widening without qualification evidence;
- new feature implementation.

## Definition of Done

FR-7/FR-8/FR-9 are complete when Semantic has a canonical example set, a first-contact onboarding path, and a reproducible release qualification checklist that can honestly support either strong limited release or public release candidate posture.
