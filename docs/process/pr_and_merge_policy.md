# Semantic PR And Merge Policy

Status: active and mandatory

This document defines the repository gate for PR readiness and merge eligibility.

## PR Ready Checklist

A PR is ready for review only if all of the following are true:

- the PR is one logical step with a narrow scope
- the scope and rationale are documented
- behavior-changing work includes tests or updated verification evidence
- relevant docs are updated in the same step
- release, readiness, and milestone claims remain honest
- the author can state exactly what is in scope and what is intentionally left out

## Test Gate

- document-only PRs may skip tests if they do not change behavior, contracts, generated artifacts, or release outputs
- every non-document-only PR must run the relevant test and verification contour for the affected layer
- partial green is not enough if the affected layer requires a wider validation contour
- if a test fails, stop and fix the cause before asking for merge
- do not normalize red tests as acceptable backlog
- if a change cannot be brought back to green, roll it back before merge

## Merge Rule

- merge only when all relevant tests are green
- merge only when CI is green
- merge only when documentation is synchronized with the actual change
- merge only when the PR does not exceed its declared scope
- merge only when the backup record is explicit: two backups before edits, first backup removed after green, second backup retained or removed with justification
- if any required gate is red, merge is blocked

## Recovery Rule

- if desired behavior is correct but tests are red, the task is still incomplete
- if tests are red because expectations are outdated, update the tests and the associated docs explicitly
- if a fix reveals broader drift, narrow the change or open a new scope rather than hiding the drift inside the same PR
- if the step cannot be stabilized to green in the current PR, revert the change to the last green state and regroup from there

## Reviewer Rule

- reject PRs with undocumented scope movement
- reject PRs with missing tests for behavioral changes
- reject PRs whose docs and claims overstate actual behavior
- reject PRs that depend on follow-up fixes to become safe to merge
