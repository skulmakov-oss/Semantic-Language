# SemCode Version Discipline

Status: proposed checkpoint

## Goal

Freeze the SemCode version-discipline rules for the current `main` baseline.

This checkpoint exists to stop SemCode drift between:

- IR/emit ownership
- verifier admission
- VM execution support
- release-facing compatibility statements

The point of this track is version discipline, not new binary behavior.

## Canonical Reading

SemCode is the versioned binary contract between the lowered execution pipeline
and verified VM execution:

`frontend -> semantics -> lowering -> IR passes -> emit -> verify -> VM`

In that reading:

- `sm-ir` owns SemCode header, opcode, section, and capability semantics
- `sm-emit` remains a producer-facing facade over that owned contract
- `sm-verify` admits or rejects produced artifacts structurally before execution
- `sm-vm` executes only admitted SemCode and must not redefine the format

## Current Landed State

The current `main` already includes:

- a supported header family from `SEMCODE0` through `SEMCOD13`
- canonical header specs and capability masks in `sm-ir`
- verifier admission aligned to the same supported header family
- VM rejection of unsupported headers without silent reinterpretation
- additive capability widening across the admitted header line
- explicit release-facing distinction between the published stable line and the
  wider admitted line on current `main`

That is enough to freeze the narrow version-discipline contract.

## Included In This Freeze

- SemCode as a versioned binary contract owned by `sm-ir`
- the current supported family from `SEMCODE0` through `SEMCOD13`
- additive capability widening as the only admitted widening model in the
  current baseline
- explicit version review for binary layout or meaning changes
- release-facing honesty about published stable versus forward-only admitted
  `main` behavior
- explicit required follow-up items for any future SemCode family bump

## Explicit Non-Goals

This checkpoint does not include:

- a new SemCode header family
- opcode widening
- section-layout widening
- verifier semantic widening
- VM execution widening
- stable-tag promotion of post-stable admitted families
- textual IR or package-level versioning work

## Freeze Rules

- existing admitted header families remain fixed once they ship on `main`
- capability bits widen additively and must not be repurposed in place
- header selection remains derived from actual emitted usage, not from profile
  permission alone
- verifier and VM support must stay aligned to the same supported header family
- release docs must distinguish the published stable line from the wider
  forward-only admitted line on `main`
- SemCode format changes require explicit version review, not silent mutation

## Required Follow-Up For A Future SemCode Bump

Any future header or capability widening must update all of:

- `docs/spec/semcode.md`
- `docs/roadmap/compatibility_statement.md`
- `docs/roadmap/v1_readiness.md`
- verifier compatibility tests
- VM compatibility tests
- golden or compatibility fixtures if public behavior changed

## Acceptance Criteria

This checkpoint is complete only when:

- `docs/spec/semcode.md` reflects the same staged-contract reading
- architecture docs treat SemCode widening as explicit and additive
- roadmap docs point to this file as the active SemCode discipline checkpoint
- release-facing compatibility docs use the real admitted header names
- no document implies silent retroactive widening of the published stable line
