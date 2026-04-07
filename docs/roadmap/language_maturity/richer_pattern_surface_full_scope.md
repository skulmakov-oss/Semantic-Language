# Richer Pattern Surface Full Scope

Status: proposed M9.4 post-stable subtrack
Related roadmap package:
`docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`

## Goal

Widen the first-wave match/pattern surface to cover binding patterns in let,
guard wildcards, nested tuple destructuring, and or-patterns — without opening
a full pattern compiler or adding implicit coercions.

This is a forward-only language-maturity subtrack for current `main`. It is not
a claim that these pattern forms already exist on the published stable line.

## Why This Track Exists

Semantic now has a working `match` surface for ADT enum variants and records,
and basic tuple destructuring in `let`. The current first-wave pattern baseline
is intentionally narrow. The next practical barrier to ergonomic code is the
absence of:

- or-patterns for collapsing multiple arms that share a body
- nested tuple destructuring beyond one level
- wildcard `_` in `let` bindings
- range patterns in match arms
- `if let` guard desugaring for binding guards

Without these forms, common pattern-matching idioms require verbose workarounds
that do not match the stated language-maturity goals of the M9 package.

This track opens the minimum richer pattern surface without mixing in a full
pattern compiler rewrite, string patterns, slice patterns, or exhaustiveness
checker overhaul.

## Decision Check

- [ ] This is a new explicit post-stable track with its own scope decision
- [ ] This does not silently widen published `v1.1.1`
- [ ] This is one stream, not a mixture of multiple tracks
- [ ] This can be closed with a clear done-boundary

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- `match` works for ADT enum variants and records with explicit field binding
- `let (a, b) = tuple` works for one level of tuple destructuring
- no or-patterns exist in the published stable baseline
- no nested destructuring beyond one level is admitted
- no wildcard `_` binding in `let` is admitted in the published stable baseline
- no range patterns exist in match arms in the published stable baseline
- `if let` guard desugaring is not part of the published stable contract

That baseline remains the source of truth until this subtrack explicitly lands
its widened contract on `main`.

## Included In This Track

- or-patterns: `Variant::A | Variant::B =>` in match arms
- nested tuple destructuring: `let (a, (b, c)) = ...` beyond one level
- `_` wildcard in let bindings
- range patterns: `1..=5 =>` in match arms
- binding guards: `if let Some(x) = ...` desugaring
- docs/spec/tests/compatibility wording for the widened contract

## Explicit Non-Goals

- active patterns or view patterns
- string pattern matching beyond equality
- slice patterns
- macro-generated pattern expansion
- exhaustiveness checker rewrite
- implicit coercions across pattern boundaries
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance

- scope checkpoint
- roadmap/milestone/plan linkage

### Wave 1 — Owner Layer

- AST pattern node extensions for new pattern forms
- pattern surface ownership inventory
- explicit typecheck/exhaustiveness gap markers before executable admission

### Wave 2 — Parser Admission

- parser admission for or-patterns, nested tuple destructuring, wildcards,
  range patterns, and binding guards
- explicit diagnostics for unsupported pattern forms

### Wave 3 — Typecheck + Exhaustiveness Updates

- typecheck pass updates for new pattern node types
- exhaustiveness checker updates for admitted pattern forms
- verifier compatibility for widened pattern surface

### Wave 4 — Freeze

- docs/spec/tests/compatibility freeze

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint
2. PR 2: owner-layer AST pattern node extensions
3. PR 3: parser admission for new pattern forms
4. PR 4: typecheck and exhaustiveness updates
5. PR 5: freeze and close-out

## Initial First-Wave Reading

The first-wave richer pattern contract is intentionally narrow:

- only the five listed forms are admitted (or-patterns, nested tuples,
  wildcards, range patterns, binding guards)
- no full pattern compiler rewrite
- no implicit coercions introduced at pattern boundaries
- exhaustiveness checker receives targeted updates only, not a rewrite
- all new forms are additive over the existing match/let baseline

That keeps the track additive over the current first-wave pattern surface
without opening a full abstraction system in one step.

## Acceptance Reading

This track is done only when:

- all five admitted pattern forms are explicit and inspectable
- parser, typecheck, and exhaustiveness updates agree on one deterministic
  first-wave model
- docs/spec/tests describe the same admitted baseline
- published `v1.1.1` and widened `main` are explicitly distinguished

## Non-Commitments After Close-Out

Even after this first wave lands, the repository still does not claim:

- active patterns or view patterns
- string pattern matching beyond equality
- slice patterns
- macro-generated pattern expansion
- a rewritten exhaustiveness checker
- that these pattern forms were already part of the published `v1.1.1` line

## Merge Gate

Before closing this track:

- [ ] code/tests are green
- [ ] spec/docs are synced
- [ ] public API or golden snapshots are updated if needed
- [ ] compatibility/release-facing wording is honest
