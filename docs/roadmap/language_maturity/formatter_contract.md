# Formatter Contract

Status: proposed v0

## Purpose

This document defines the first intended public formatter contract for
Semantic.

The goal is to make source formatting a real tooling surface rather than a
future idea implied by examples and manual editing habits.

## Canonical Entry Point

The first canonical public formatter command should be:

- `smc fmt`

Reason:

- it keeps the formatter inside the already-public CLI shell
- it avoids fragmenting the first tooling wave into too many binaries
- it matches the current release story, where `smc` is already the main
  user-facing entrypoint

Optional future note:

- a standalone `smfmt` wrapper may exist later, but it should be treated as a
  convenience shell around the same formatter contract rather than a separate
  formatting language

## First-Wave Command Shape

The first public command shape should be:

```text
smc fmt <path>
smc fmt --check <path>
```

Phase-1 expectations:

- formatting a single file
- formatting a directory tree of `.sm` sources
- check mode for CI and repository hygiene

## Scope

The first formatter wave should format source that already belongs to the
Semantic executable language surface.

That includes:

- ordinary Rust-like Semantic source
- import/export layout
- `if`, `match`, and function bodies
- Logos blocks, with explicit indentation rules

It should not wait for future package or workspace tooling to become stable.

## Output Guarantees

The first formatter contract should promise:

- deterministic output for the same source input
- idempotence once a file is already formatted
- no silent semantic changes
- one canonical layout style for stable language constructs

This is the minimum bar for using the formatter in CI, examples, and editor
integration.

## Formatting Domains

### Ordinary Source Layout

The formatter should normalize:

- indentation
- brace placement
- spaces around operators
- blank-line boundaries between declarations
- trailing whitespace

### Imports And Exports

The formatter should normalize:

- one canonical spacing/alignment style
- grouped import/export declarations without inventing semantic reordering

### Logos

The formatter should treat Logos as a first-class formatting domain.

Phase-1 rule:

- Logos blocks should receive intentional indentation and layout rules rather
  than being left as raw preserved text

This matters because Logos is part of the advertised source surface, not a
private syntax corner.

## Check Mode

`smc fmt --check` should report whether input already matches canonical
formatting.

Phase-1 expectation:

- exit success when no reformat would occur
- exit failure when formatting changes would be produced

The exact numeric exit-code taxonomy may evolve with the broader CLI contract,
but the success/failure meaning should stay clear.

## Relationship To LSP

The formatter is the source-layout authority.

The intended rule is:

- editor tooling may call into the formatter
- the LSP layer should not define a competing formatting dialect

This keeps editor integrations aligned with the same repository and CLI
behavior.

## Non-Goals

The first formatter contract does not yet try to define:

- comment-preservation guarantees beyond ordinary stable formatting behavior
- user-configurable style profiles
- import sorting by semantic dependency analysis
- formatting for future features not yet frozen in the source-language bundle

## Stability Note

The formatter should begin as:

- `draft target`

It should only move toward `stable` once:

- source syntax and semantics are frozen enough
- CI and example workflows actually rely on it
- output behavior has survived at least one release wave without churn

## Cross-References

This formatter contract works together with:

- `docs/roadmap/language_maturity/tooling_maturity.md`
- `docs/roadmap/language_maturity/tooling_layers.md`
- `docs/roadmap/language_maturity/tooling_workflows.md`
- `docs/spec/syntax.md`
- `docs/spec/logos.md`
