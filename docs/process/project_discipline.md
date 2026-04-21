# Semantic Project Discipline

Status: active and mandatory

Project motto:

- discipline in maximum form

Priority statement:

- project tempo must be driven by discipline, not by chaotic acceleration
- when speed and discipline conflict, discipline wins
- repeat this rule in scope setting, PR review, checkpointing, and merge decisions

This document defines the repository operating contract.
It applies to every request, every milestone, every PR, and every merge decision.

## Canonical Source Of Truth

- treat `origin/main` as the canonical source of truth for merge-ready work
- do not treat a dirty local worktree as a clean basis for new PRs
- if a local workspace contains unrelated WIP, create a fresh worktree from the canonical branch before starting merge-ready work
- do not make release or readiness claims from local-only state

## Scope Discipline

- every logical step must have an explicit scope before implementation begins
- every scope must state `in-scope`, `out-of-scope`, and done criteria
- do not silently widen language semantics, runtime behavior, verifier behavior, import behavior, or release claims
- if a surface needs to widen beyond current admitted behavior, open a new explicit scope first
- one PR equals one logical step

## Documentation Discipline

- every step must leave a durable documentation trail
- every request that changes code, behavior, contracts, release posture, or roadmap must be reflected in repository documentation
- every milestone must record scope, status, admitted contour, and close-out state
- every PR must leave an explicit note of what changed and why
- do not rely on chat history as the sole record of project decisions

## Test Discipline

- every behavioral change must be covered by tests or updated verification evidence
- document-only changes are the only class of change allowed to skip tests
- if a change touches behavior and the existing tests are insufficient, add or extend tests before considering the step complete
- do not merge with failing relevant tests
- if tests fail, the work is not complete until the cause is understood and the state is returned to green

## Backup And Recovery Discipline

- before making changes, create two reserve backups of the relevant working state
- the backups may be filesystem snapshots, worktree snapshots, or another explicit recovery form, but both copies must be real and restorable
- if the change cannot be brought back to a green state before merge readiness, roll the step back rather than carrying broken state forward
- after the change reaches a verified green state, delete the first backup
- keep the second backup until it is genuinely no longer needed for recovery, audit, or comparison
- do not remove the final backup copy just because the branch is green; remove it only when the recovery value is truly exhausted

## Claim Discipline

- documentation claims must match actual repository behavior
- milestone and release claims must match qualified evidence, not intent
- do not call a surface stable, complete, qualified, or released unless the repository evidence supports that claim
- if behavior is narrower than desired, state the narrow truth explicitly

## Execution Rule

- scope first
- implementation second
- tests and verification third
- documentation sync before completion
- merge only after the full relevant validation contour is green
