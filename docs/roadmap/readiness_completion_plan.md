# Readiness Completion Plan

Status: proposed completion roadmap

## Purpose

This document defines the remaining work needed to move Semantic from its
current state:

- published stable release line
- completed first `Gate 1` qualification cycle
- current decision state: `limited release`

to a state that can honestly support a stronger external readiness claim.

This is not a "Wiki polish" plan. Wiki/readme packaging is only a downstream
effect of real readiness completion.

## Current Baseline

Current `main` already has:

- a deterministic staged pipeline:
  `frontend -> semantics -> lowering -> IR passes -> emit -> VM`
- mandatory verifier admission before the standard VM route
- crate-level ownership boundaries that are reflected in code and tests
- a published stable `v1.1.1` line
- a widened current-`main` surface that is broader than the published stable
  promise
- a completed first `Gate 1` qualification cycle with decision state
  `limited release`

This means the project is not in an "architecture-only" phase anymore.
The remaining work is completion and consolidation work.

## Readiness Completion Criteria

Semantic should be treated as readiness-complete only when all of the
following are true.

### R1. Technical Core Trust

- execution contour is stable
- verifier and VM are trusted on the admitted contour
- determinism is demonstrated
- benchmark baseline exists and is reproducible

### R2. Practical Programming Trust

- module-based authoring does not feel like a special-case workaround
- real-program trials no longer show a dominant practical blocker
- import/module ergonomics are strong enough for ordinary use on the admitted
  contour

### R3. Honest Release Posture

- release promise is explicit and evidence-backed
- the project can honestly remain `limited release`, graduate to a stronger
  limited contour, or justify a `public release` candidate

### R4. External Usability

- a strong external engineer can build, check, run, and inspect the system
  without author-side guidance
- examples, onboarding, and artifact expectations are sufficient for
  independent use

### R5. One Truth Model

The following layers must not conflict:

- `published stable`
- `qualified limited release`
- `landed on main, not yet promised`
- `out of scope`

## Truth Vocabulary

Every readiness-facing document should use the same four status families.

### Published Stable

Behavior and artifacts promised by the published stable line.

### Qualified Limited Release

Behavior qualified by the current `Gate 1` evidence and explicitly admitted
into the current practical-programming contour.

### Landed On `main`, Not Yet Promised

Behavior that exists on current `main` but is not part of the current release
promise until a later qualification or release decision promotes it.

### Out Of Scope

Behavior intentionally excluded from the current release contour.

## Governing Rules

### Rule 1 — No Readiness By Intuition

Readiness cannot be upgraded because the project "feels mature".
Readiness can move only when the evidence changes.

### Rule 2 — Phase A Becomes Authoritative

Once truth consolidation lands, any stale status document becomes a real
readiness defect and must be synced before further widening.

### Rule 3 — One Narrow Widening At A Time

During readiness completion, do not open multiple practical-widening waves at
once.

### Rule 4 — Landed Is Not Automatically Promised

Current-`main` behavior is not automatically part of the release contour.
Promotion requires an explicit qualification or release decision.

### Rule 5 — UI Stays Outside The Current Contour Unless Admitted

UI remains outside the readiness-critical contour unless a later release scope
decision explicitly pulls it in.

## Phase Order

### Phase A — Readiness Truth Consolidation

Goal:

- establish one canonical readiness model across release-facing docs

Required sync targets:

- `README.md`
- `docs/roadmap/backlog.md`
- `docs/roadmap/milestones.md`
- `docs/roadmap/v1_readiness.md`
- `docs/roadmap/compatibility_statement.md`
- `reports/g1_*.md`
- stale scope docs whose status language no longer matches current `main`

Done only when:

- the four truth categories are used consistently
- no release-facing document silently overstates current readiness
- new widening work can be evaluated against one shared vocabulary

### Phase B — Module Authoring Completion

Goal:

- remove the strongest remaining practical-programming weakness without opening
  a redesign track

#### B1. Harden Current Narrow Contour

Focus:

- negative boundary cases
- cycle rejection
- duplicate symbol behavior
- deterministic ordering
- repeated import behavior
- helper collision scenarios

#### B2. One Narrow Widening Wave

Choose exactly one:

- selected import
- namespace-qualified executable access

#### B3. Optional Second Narrow Widening

Allowed only if:

- the first widening still leaves module authoring as the dominant practical
  blocker

Explicit non-goals:

- alias + wildcard + re-export + package-qualified mix in one wave
- broad package/import redesign
- ecosystem-wide package management expansion

Done only when:

- module-based authoring no longer feels like a narrow helper hack
- remaining limits sound like advanced scope, not everyday pain

### Phase C — Gate 1.1 Re-Synthesis

Goal:

- recompute the release verdict using updated evidence after any practical
  widening

Required reports:

- `reports/g1_real_program_trial.md`
- `reports/g1_frontend_trust.md`
- `reports/g1_surface_expressiveness.md`
- `reports/g1_release_scope_statement.md`

Allowed outcomes:

- `limited release` remains unchanged
- `limited release` broadens
- `public release candidate` becomes justified

Done only when:

- the new verdict is traceable to evidence rather than sentiment

### Phase D — External Usability Completion

Goal:

- let a strong external engineer become productive without author-side guidance

Required deliverables:

- canonical examples pack
- one-page onboarding
- developer quickstart path

Examples should cover:

- rule/state
- CLI utility
- data-heavy program
- module-based program
- one explicit boundary example where useful

Done only when:

- an external engineer can start productively in roughly 15–20 minutes
- core workflows do not require oral explanation

### Phase E — Release-Facing Artifact Completion

Goal:

- make the supported product/artifact model explicit

Required work:

- define supported artifacts
- define supported platform scope
- keep stable assets and current-`main` widened behavior clearly separated
- align release bundle checks, smoke matrix, and public docs

Done only when:

- users can answer four questions unambiguously:
  - what to download
  - what to run
  - what is promised by that artifact
  - what is only current-`main` behavior

### Phase F — Final Readiness Review

Goal:

- make one explicit completion decision rather than drifting into endless
  cleanup

Review questions:

- does the technical core now support full trust on the admitted contour
- does practical-programming authoring feel natural enough
- is the module/import story sufficiently usable
- can an external engineer work productively without author help
- do all public truth layers agree

Required final output:

- `ready for strong limited release`
- or `ready for public release`
- or `one more narrow blocker-removal cycle`

Done only when:

- the final decision exists as an explicit artifact rather than a discussion

## Recommended Default Path

Unless evidence suggests otherwise, the recommended order is:

1. Phase A
2. Phase B1
3. Phase B2
4. Phase C
5. Phase D
6. Phase E
7. Phase F

If Phase B2 proves unnecessary after Phase B1, move directly from B1 to D/E/F
only through an explicit decision.

## Near-Term Execution Reading

For the next 2–4 weeks, the default readiness-completion sequence is:

- Week 1:
  - Phase A
  - Phase B1
- Week 2:
  - Phase B2
  - optional decision on B3
- Week 3:
  - Phase C
  - start Phase D
- Week 4:
  - finish Phase D
  - Phase E
  - prepare Phase F

## Explicit Non-Goals During This Completion Cycle

Do not let this cycle expand into:

- large Workbench growth
- broad agent/runtime/provider expansion
- a new UI platform push
- broad ecosystem storytelling beyond the admitted contour
- unrelated package/import redesign

Those may be valid future tracks, but they are not on the current readiness
critical path.
