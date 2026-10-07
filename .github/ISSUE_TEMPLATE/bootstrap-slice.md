---
name: SHF work item
description: Define one bootstrap work item under the SHF self-hosting roadmap
title: "SHF-?: "
labels: []
assignees: []
---

## SHF stage

- Parent stage (SHF-0 … SHF-17):
- Owner repository: `Semantic-Language` / `Semantic` / both
- Upstream authority: `skulmakov-oss/Semantic#1910`

## Objective

<!-- One concrete outcome. -->

## Reference

- Repository: `skulmakov-oss/Semantic`
- Exact SHA (where relevant):
- Contract affected (spec path / module):

## Ownership check

- [ ] This does not change the Semantic language/runtime contract (otherwise it belongs upstream first)
- [ ] No compiler logic is placed in host/bootstrap glue

## Migration state

Current: REFERENCE_ONLY / MIRRORED / DIFFERENTIAL / QUALIFIED / CANONICAL
Target:

## Comparison form

<!-- Tier from docs/QUALIFICATION.md §1 and exact schema. -->

## Evidence required

- [ ] Positive cases
- [ ] Negative cases where contractual
- [ ] Boundary cases
- [ ] Mutation proof (gate fails on a meaningful defect)
- [ ] Unexplained deltas = 0
- [ ] Limitations recorded

## Non-goals

-

## Stop conditions

Stop if the reference contract is ambiguous, the reference pin cannot be verified, or a required
capability is missing upstream.
