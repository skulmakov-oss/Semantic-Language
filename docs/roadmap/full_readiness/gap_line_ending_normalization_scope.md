# GAP: Line Ending Normalization

Status: implementation-scope draft  
Track: Semantic Full Readiness / frontend robustness  
Source trigger: `weather_station.sm` canonical-program trial

## Problem

The current Rust-like frontend path appears fragile when a valid `.sm` file uses Unix LF line endings instead of Windows CRLF line endings.

A Semantic source file must not depend on host/editor line-ending style.

## Goal

Normalize source line endings before parsing so that LF, CRLF, and mixed line endings are accepted consistently where the token stream is otherwise identical.

## Scope

Implement source normalization in the frontend input boundary:

- accept LF files;
- accept CRLF files;
- reject or normalize mixed line endings by explicit policy;
- preserve stable line/column diagnostics;
- preserve byte/span mapping policy or document any normalized-span behavior.

## Required behavior

```text
fn main() {
    return;
}
```

must parse/check identically with:

- LF;
- CRLF;
- trailing newline;
- no trailing newline.

## Required tests

Add positive fixtures for:

- minimal function with LF;
- minimal function with CRLF;
- function sequence with blank lines;
- weather-station-style function file with LF.

Add diagnostics regression:

- parse error line/column remains stable after normalization.

## Acceptance criteria

- `smc check` accepts LF and CRLF versions of the same program;
- diagnostics still point to correct line/column;
- no parser profile silently diverges because of line endings;
- no docs claim CRLF-only behavior;
- a short note is added to source syntax or frontend docs.

## Out of scope

- formatter implementation;
- Unicode normalization;
- comment parsing;
- semantic feature expansion;
- platform-specific file IO changes outside the frontend input boundary.

## Validation commands

```powershell
cargo test -q
cargo test -q -p sm-front
smc check tests/fixtures/line_endings/minimal_lf.sm
smc check tests/fixtures/line_endings/minimal_crlf.sm
```
