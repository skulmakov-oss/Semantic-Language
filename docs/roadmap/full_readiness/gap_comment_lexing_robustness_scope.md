# GAP: Comment and Source Text Lexing Robustness

Status: implementation-scope draft  
Track: Semantic Full Readiness / frontend robustness  
Source trigger: `weather_station.sm` canonical-program trial

## Problem

The weather-station trial exposed that valid-looking source files with leading comments or decorative comment text can fail before ordinary parsing.

A production-ready source frontend must handle ordinary source comments predictably.

## Goal

Make comment handling explicit and robust for the Rust-like executable Semantic surface.

## Scope

Define and implement comment handling policy for:

- leading `//` line comments before the first `fn`;
- blank lines around comments;
- comment-only lines between declarations;
- ASCII comment text;
- non-ASCII comment text policy: accept UTF-8 comments or reject with a dedicated diagnostic, but never fail as a generic parser confusion.

## Required behavior

The following should be valid if comments are supported in the selected parser profile:

```semantic
// file-level note

fn main() {
    // inside function note
    return;
}
```

If a parser profile intentionally does not support comments, it must emit a clear diagnostic instead of reporting an unrelated syntax expectation at line 1.

## Required tests

Positive tests if comments are admitted:

- leading comment before first function;
- comment between functions;
- comment inside function body;
- blank line plus comment;
- ordinary ASCII comments in canonical examples.

Policy tests:

- decorative UTF-8 comment line if accepted;
- deterministic diagnostic if rejected.

## Acceptance criteria

- `smc check` behavior for comments is documented;
- leading comments no longer produce misleading generic parse errors;
- comments do not affect line/column diagnostics for following code;
- examples may include ordinary comments without breaking the Rust-like frontend;
- non-ASCII comment policy is explicit.

## Out of scope

- doc comments as metadata;
- block comments;
- formatter changes;
- Unicode identifiers;
- Logos indentation comment policy unless separately scoped.

## Validation commands

```powershell
cargo test -q -p sm-front
smc check tests/fixtures/comments/leading_line_comment.sm
smc check tests/fixtures/comments/inline_line_comment.sm
```
