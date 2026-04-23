# Semantic — Readiness Completion Plan

Status: active completion plan
Primary owners: language maturity, release discipline, public-facing readiness

## Purpose

This document defines the completion plan for bringing Semantic from its
current state of:

- architecturally mature
- engineering-convincing
- `limited release` qualified

to a state of full readiness.

This is not a Wiki plan.
This is not a feature wishlist.
This is the focused completion program required to reach a point where:

- the technical core is trusted
- practical-programming contour is sufficiently broad
- the release posture is simple and honest
- an external strong engineer can start productively without direct author
  guidance
- the public-facing story matches the real repository state

## Current State

Current `origin/main` is already strong:

- the project is positioned as a deterministic, contract-driven
  compiler/runtime system
- architecture and crate ownership are mature
- execution integrity is strong
- release discipline is strong
- the first `Gate 1` qualification cycle is complete
- the current formal verdict is `limited release`, not `public release`
- no active blocker-removal stream is currently open on `main`, and any new
  widening now requires an explicit scope decision

### What Is Already Strong

- architecture and ownership boundaries
- verifier-first execution discipline
- runtime integrity and determinism baseline
- benchmark baseline
- qualification discipline
- release governance

### What Is Not Yet Complete

- one simple public status model
- sufficiently broad practical module authoring contour
- fully consolidated release truth across `README`, readiness, backlog,
  milestone, and qualification layers
- external usability without author guidance
- a fully finished release-facing artifact story

## Definition Of Full Readiness

Semantic should be considered fully ready only when all of the following are
true.

### R1 — Technical Core Readiness

- verifier / VM / runtime contour is trusted
- determinism and integrity are proven
- benchmark baseline exists and is reproducible

### R2 — Practical Programming Readiness

- real programs are no longer blocked by narrow everyday authoring pain points
- module-based authoring no longer feels like a special-case workaround
- the language contour is broad enough for a strong limited-release or
  public-release claim

### R3 — Honest Release Posture

- the current claim is simple and unambiguous
- `published stable`, `qualified limited release`, `landed on main, not yet
  promised`, and `out of scope` are clearly separated

### R4 — External Usability Readiness

- a strong external engineer can install, build, check, run, verify, and
  understand the system without direct author guidance

### R5 — Public Truth Consolidation

- `README`, backlog, milestones, readiness docs, compatibility docs, and
  qualification reports describe the same truth model

## Global Execution Rule — Readiness Freeze On `main`

During readiness completion, `main` must remain under a narrow freeze policy.

Allowed changes:

- regression fixes
- documentation and status consolidation work
- the explicitly chosen module-authoring widening wave
- qualification and benchmark reruns required by this plan

Disallowed changes:

- unrelated feature widening
- parallel language-surface expansion outside the chosen wave
- opportunistic refactors that change the basis of qualification mid-cycle

Every PR during readiness completion must explicitly state whether it is:

- freeze-safe documentation/status work
- regression fix
- chosen Phase B wave work
- qualification or benchmark re-gating work

This freeze exists to prevent Phase C and Phase F from being written against a
moving target.

## Authority Order

During readiness completion, the release-facing authority order is:

1. `docs/roadmap/public_status_model.md`
   Vocabulary authority for:
   - `published stable`
   - `qualified limited release`
   - `landed on main, not yet promised`
   - `out of scope`
2. `docs/roadmap/v1_readiness.md`
   Current release-facing posture authority
3. `reports/g1_release_scope_statement.md`
   Current practical-programming qualification verdict authority

No other document may silently override these layers.

## Completion Phases

### Phase A1 — Readiness Truth Skeleton

Target duration: 2–3 days.

#### Goal

Create the truth-consolidation structure early, without prematurely freezing
factual release claims that may still change after module widening and
re-synthesis.

#### Required Vocabulary

Use one canonical status vocabulary everywhere:

- `published stable`
- `qualified limited release`
- `landed on main, not yet promised`
- `out of scope`

#### Work Items

- create the canonical status document skeleton
- define where each status class is allowed to appear
- align document structure and terminology across:
  - `README.md`
  - `docs/roadmap/backlog.md`
  - `docs/roadmap/milestones.md`
  - `docs/roadmap/v1_readiness.md`
  - `docs/roadmap/compatibility_statement.md`
  - `docs/roadmap/stable_release_policy.md`
  - `reports/g1_*`
- review stale scope docs and mark them for later factual-claim sync
- define the release-artifact classification skeleton early

#### Deliverables

- `docs/roadmap/public_status_model.md`
- release-artifact classification skeleton

#### Definition Of Done

- the status vocabulary is fixed
- the repository has one agreed truth-model structure
- factual claims that may change after re-synthesis are intentionally left for
  Phase A2

### Phase B0 — Canonical Examples Draft And Widening Decision Input

Target duration: 2–3 days.

#### Goal

Produce an early rough canonical examples draft so the widening choice is based
on real authoring friction rather than guesswork.

#### Work Items

Prepare a draft examples pack covering at minimum:

- rule/state-oriented program
- CLI utility
- data-heavy small program
- module-based program
- one current narrow-contour boundary example

For each draft example, record:

- required import/module forms
- current workarounds
- friction points
- which missing surface causes the most distortion

#### Decision Rule For The Widening Wave

Choose exactly one widening wave based on the draft examples.

- choose `selected import` if draft examples are forced into awkward local
  rebinding or boilerplate because direct symbol import is missing
- choose `namespace-qualified executable access` if draft examples remain
  structurally clean but become noisy or unnatural due to repeated
  namespace-only access
- do not choose by intuition alone

#### Deliverables

- draft canonical examples pack
- `docs/roadmap/module_authoring_wave_decision.md`

#### Definition Of Done

- the widening wave is selected using explicit evidence from example friction
- there is a written justification for why the chosen wave reduces the
  highest-value practical pain

### Phase B — Module Authoring Completion

Target duration: 1–2 weeks.

#### Goal

Remove module authoring as the main remaining practical weakness.

#### Scope Rule

Do not redesign the whole import/package system.
Only perform 1–2 honest, narrow widening waves.

#### B1 — Harden The Current Narrow Contour

Required work:

- negative fixtures for all still-out-of-scope import forms
- cycle rejection checks
- duplicate symbol handling checks
- deterministic ordering checks
- repeated import handling checks
- helper collision scenarios

#### B2 — One Narrow Widening Wave

Choose exactly one, based on Phase B0 evidence:

- `selected import`
- `namespace-qualified executable access`

The choice must be justified by the draft canonical examples and their
workaround burden.

#### B3 — Optional Second Narrow Widening Wave

Allowed only if one widening wave does not sufficiently reduce practical
authoring pain.

#### Explicitly Not In Scope

Do not attempt, in this phase:

- alias + wildcard + re-export + package-qualified redesign as one wave
- large import/package architecture rewrite

#### Deliverables

- widened fixtures and tests for the chosen wave
- updated source/module docs
- short module-wave completion note

#### Definition Of Done

- module-based executable authoring no longer looks like a narrow special-case
  path
- remaining limitations clearly read as advanced scope, not basic daily pain

### Phase C — Gate 1.1 Re-Synthesis And Benchmark Re-Gate

Target duration: 3–5 days.

#### Goal

Re-run the affected qualification conclusions after module-authoring widening,
and explicitly re-gate benchmark evidence if the widening touched compiler or
runtime behavior.

#### Update The Following Reports

- `reports/g1_real_program_trial.md`
- `reports/g1_frontend_trust.md`
- `reports/g1_surface_expressiveness.md`
- `reports/g1_release_scope_statement.md`

Update these as well if the widening changes execution-path behavior or
measurable pipeline/runtime cost:

- `reports/g1_execution_integrity.md`
- `reports/g1_benchmark_baseline.md`

#### Benchmark Re-Gate Requirement

If Phase B changes frontend, semantics, lowering, emit, verifier, or VM
behavior in a way that can affect pipeline cost or runtime behavior, rerun the
relevant benchmark baseline and record whether:

- the prior baseline still stands unchanged
- the baseline changed but remains acceptable
- a regression or step-change requires explicit documentation

#### Possible Outcomes

- `limited release` remains unchanged
- `limited release` contour broadens
- `public release candidate` becomes justified
- another blocker-removal cycle is still required

#### Definition Of Done

- the new verdict is evidence-based
- benchmark posture is explicitly reconfirmed or updated
- the release claim broadens only if qualification evidence actually supports
  it

### Phase A2 — Factual Truth Consolidation

Target duration: 2–3 days.

#### Goal

Apply the now-finalized factual readiness and release claims after Phase C, so
the truth model is consolidated once rather than rewritten twice.

#### Work Items

- update factual release posture in:
  - `README.md`
  - `docs/roadmap/backlog.md`
  - `docs/roadmap/milestones.md`
  - `docs/roadmap/v1_readiness.md`
  - `docs/roadmap/compatibility_statement.md`
  - `docs/roadmap/stable_release_policy.md`
  - `reports/g1_*`
- update stale scope docs so they reflect the post-Phase C state
- finalize the release-artifact status mapping

#### Definition Of Done

- release-facing documents now contain synchronized factual claims
- no second truth-consolidation pass is expected for the same cycle

### Phase D — External Usability Completion

Target duration: 1–2 weeks.

#### Goal

Make the project independently usable by a strong external engineer.

#### Operational Note

The original ambition of "productive in 15–20 minutes" must be treated as a
tested operational claim, not a self-assessment.

Define the target user explicitly before evaluation.
Recommended baseline:

- a senior engineer with 5+ years of systems-language experience
- comfortable with Rust-like toolchains and compiler-style workflows
- no prior project-specific domain knowledge

#### D1 — Canonical Examples Pack

Prepare 4–6 canonical examples covering:

- rule/state-oriented program
- CLI utility
- data-heavy small program
- module-based program
- one boundary example showing a narrow current contour honestly

For each example, include:

- purpose
- run/check/verify commands
- expected output
- what language surface it demonstrates

#### D2 — One-Page Onboarding

Include:

- install
- `smc check`
- `smc compile`
- `smc run`
- `smc verify`
- how to read diagnostics
- how `stable`, `limited`, and `main-only` surfaces differ

#### D3 — Developer Quickstart

Explain:

- repository structure
- where spec lives
- where readiness posture lives
- where examples live
- what is promised now

#### D4 — Cold-Start Rehearsal

Run the onboarding on a clean environment or clean VM with a timer.

Record:

- time to first successful `check`
- time to first successful `run`
- time to understand the basic status model
- where onboarding still assumes hidden author knowledge

#### Deliverables

- `docs/getting_started.md`
- `docs/examples_index.md`
- canonical examples pack
- cold-start rehearsal note

#### Definition Of Done

- a strong external engineer profile is defined explicitly
- the onboarding path is tested in a clean environment
- no hidden author-only knowledge is required for the basic productive loop

### Phase E — Release-Facing Artifact Completion

Target duration: 3–5 days.

#### Goal

Define and finish the release-facing artifact story.

#### Work Items

- explicitly define the supported artifact set
- define platform scope honestly
- separate stable assets from current-`main` widened behavior
- connect public docs with:
  - `scripts/verify_release_bundle.ps1`
  - `scripts/verify_release_assets.ps1`
  - `docs/roadmap/release_asset_smoke_matrix.md`
  - `docs/roadmap/release_bundle_checklist.md`

#### Deliverables

- `docs/release_artifact_model.md`
- updated release-facing summary docs if needed

#### Definition Of Done

- a user can answer:
  - what do I download
  - what can I run
  - what is promised by this artifact
  - what is not yet promised

### Phase F — Final Readiness Review

Target duration: 2–3 days.

#### Goal

Make the final completion judgment.

#### Final Review Checklist

##### F1 — Technical Core

Is the execution stack fully trusted?

##### F2 — Practical Programming

Can real programs be written without persistent contour friction?

##### F3 — Module Story

Is module/import authoring sufficiently natural?

##### F4 — External Usability

Can a strong external engineer work productively without the author?

##### F5 — Public Truth

Do all release-facing truth layers agree?

#### Possible Final Decisions

- `ready for strong limited release`
- `ready for public release`
- `one final narrow blocker-removal cycle is still required`

#### Recursion Rule If Another Cycle Is Required

If one final narrow blocker-removal cycle is required, do not automatically
restart the entire A–F program.

Instead:

1. keep the existing truth model from Phase A1 intact
2. run a narrowly scoped continuation cycle:
   - targeted blocker-removal work
   - targeted qualification and benchmark re-gate
   - factual truth update
3. restart the full A–F program only if the new blocker changes the status
   vocabulary, release-artifact model, or external-usability basis

#### Deliverables

- `docs/roadmap/final_readiness_verdict.md`

#### Definition Of Done

- the final readiness verdict is made from evidence, not intuition
- the final verdict exists as a canonical repository document

## Recommended Execution Order

### Core Path

Phase A1 -> Phase B0 -> Phase B -> Phase C -> Phase A2 -> Phase D -> Phase E
-> Phase F

## 4-Week Execution Suggestion

### Week 1

- Phase A1
- Phase B0
- begin Phase B1

### Week 2

- complete Phase B1
- Phase B2
- start B3 only if justified

### Week 3

- Phase C
- Phase A2
- begin Phase D

### Week 4

- complete Phase D
- Phase E
- prepare Phase F

## Out Of Scope During Readiness Completion

Until phases A–F are complete, avoid broad expansion into:

- large Workbench feature pushes
- Agent Helm expansion
- local model provider expansion
- Git boundary growth
- native UI stack work
- broad ecosystem storytelling beyond what readiness requires

These may be important later, but they are not on the current critical path to
full readiness.

## Final Note

The project does not currently need another major conceptual leap.
It needs a final readiness-completion cycle.

The remaining work is not primarily about inventing more architecture.
It is about:

- consolidating truth
- widening the remaining practical contour where it still hurts
- re-synthesizing the release claim honestly
- finishing external usability
- and reaching a state where the final readiness verdict no longer feels
  uncertain
