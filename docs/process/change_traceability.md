# Semantic Change Traceability

Status: active and mandatory

This document defines the minimum documentation trail required for project work.

## Required Trail For Every Logical Step

Each logical step must leave a written record containing:

- `Request`
- `Why now`
- `In scope`
- `Out of scope`
- `Backups created`
- `Files or areas touched`
- `Tests added or updated`
- `Docs updated`
- `Release or milestone claim impact`
- `Status`

The trail may live in one or more repository artifacts, but it must exist by the time the step is ready for review.

## Request Discipline

- every request that changes code, contracts, release posture, or roadmap must be recorded
- the request record must describe the narrow intent of the step, not a vague future ambition
- if the request is document-only, mark it explicitly as document-only
- if the request is behavioral, identify the expected verification contour

## Milestone Discipline

- every milestone must have a scope document
- every milestone must state what is admitted, what remains outside scope, and what counts as done
- every milestone close-out must record whether the result is frozen, qualified, limited, or still experimental
- after a milestone is closed, do not continue widening the same surface without opening a new scope decision

## PR Discipline

- every PR must state the exact logical step it implements
- every PR must state that two reserve backups were created before edits began
- every PR must identify test evidence or state why the change is document-only
- every PR must identify which docs were updated
- every PR must state whether release claims or readiness claims changed
- every PR must state whether the first backup was deleted and why the second backup is being kept or removed

## Checkpoint Discipline

- after a meaningful project phase, create a dated checkpoint that records current canonical branch truth
- each checkpoint should include:
  - canonical commit
  - current admitted surfaces
  - current non-admitted surfaces
  - qualification status
  - active or completed milestone state
  - workspace warnings if local trees are dirty or unsuitable as clean bases
- checkpoints are continuity artifacts for future sessions and new work windows
