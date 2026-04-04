# Text Type Full Scope

Status: active M8.1 post-stable subtrack
Related roadmap package:
`docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`

## Goal

Introduce a first-class text type and the minimum text semantics needed for
real-world language use without silently widening the published `v1.1.1` line.

This is a future language-maturity subtrack. It is not a claim that text has
already landed on the published stable line.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- the source-visible type family does not currently expose a first-class text
  or string type
- current literals cover `quad`, `bool`, integer families, `f64`, and `fx`, but
  not a text literal family
- current runtime, IR, verifier, and VM contracts do not yet admit a canonical
  text value carrier

That baseline remains the source of truth until this subtrack explicitly lands
its widened contract on `main`.

## Included In This Track

- explicit ownership of a first-class text type
- explicit text literal spelling
- equality semantics for text values
- parser -> sema -> IR -> verifier -> VM path for admitted text values
- a narrow concatenation policy only if explicitly admitted in the same track
- docs/spec/tests/compatibility wording for the widened contract

## Explicit Non-Goals

- full formatting engine
- interpolation syntax
- regex library story
- i18n/localization framework
- rich text / styled text APIs
- host/runtime widening beyond the text carrier itself
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance

- scope checkpoint
- roadmap/milestone/plan linkage

### Wave 1 — Owner Layer

- text type ownership
- literal family ownership
- diagnostics inventory

### Wave 2 — Source Admission

- parser support
- source typing
- equality semantics

### Wave 3 — Runtime Path

- IR/lowering path
- verifier admission
- VM execution
- concatenation only if explicitly admitted

### Wave 4 — Freeze

- docs/spec/tests/goldens/compatibility freeze

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint
2. PR 2: text owner-layer types and literal ownership
3. PR 3: parser/sema/type admission for text and equality
4. PR 4: IR/verifier/VM path
5. PR 5: freeze and close-out

## Current Wave Reading

Current branch scope for Wave 2:

- parser admission for double-quoted text literals in the Rust-like executable
  source surface
- parser admission for `text` in declared type positions
- source typing for executable text literals
- same-family equality admission for `text == text` and `text != text`
- explicit source-level gap wording for text arithmetic beyond equality

Still intentionally not included in Wave 2:

- IR, verifier, or VM text carrier/runtime work
- text concatenation
- formatting, interpolation, or host-facing text ABI widening

## Acceptance Reading

This subtrack is done only when:

- text is an explicit source-visible type family with a deterministic runtime
  carrier
- literal, equality, and any admitted concatenation semantics are explicit and
  inspectable
- parser, sema, IR, verifier, and VM agree on the same admitted text surface
- published `v1.1.1` and widened `main` are explicitly distinguished

## Non-Commitments After Close-Out

Even after this first wave lands, the repository still does not claim:

- formatting/interpolation as part of the first-wave text contract
- regex support
- rich text APIs
- localization framework ownership
