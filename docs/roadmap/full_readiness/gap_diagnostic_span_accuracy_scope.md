# GAP: Diagnostic Span Accuracy

Status: implementation-scope draft  
Track: Semantic Full Readiness / diagnostics quality  
Source trigger: Probe Pack v0

## Problem

Probe Pack v0 shows that several semantic errors report the location as line `1:1`, pointing at the first function declaration, even when the actual error is deeper in the file.

Examples observed:

- `if q` where `q: quad` reports line `1:1` instead of the `if` condition;
- `i32 + i32` reports line `1:1` instead of the arithmetic expression;
- unary `-1` reports line `1:1` instead of the unary expression.

The diagnostic category can be correct while the span is not.

## Goal

Propagate accurate source spans from parser/frontend nodes through semantic analysis so diagnostics point to the offending expression or statement.

## Required behavior

Diagnostics should anchor to the smallest useful source construct:

```semantic
fn bad(q: quad) -> bool {
    if q {
        return true;
    }
    return false;
}
```

Expected diagnostic anchor:

```text
line 2, condition expression `q`
```

not:

```text
line 1, function declaration
```

## Scope

Improve span accuracy for:

- `if` condition type mismatch;
- binary operator type mismatch;
- unary operator type mismatch;
- return type mismatch if currently function-anchored;
- missing return diagnostics if currently unanchored or function-anchored.

## Required tests

Add diagnostic snapshot tests for:

- `if q` where `q: quad`;
- `i32 + i32` before i32 arithmetic is implemented, or another deliberately unsupported binary op;
- unsupported unary operator;
- bad return expression type;
- missing return in non-unit function.

Each snapshot should assert:

- line number;
- column number;
- caret line;
- diagnostic code;
- useful help text.

## Acceptance criteria

- common semantic diagnostics no longer collapse to `1:1`;
- diagnostic spans point to the offending expression/statement;
- line-ending normalization, once implemented, does not regress span positions;
- diagnostic tests cover both parser and semantic paths where relevant;
- docs for diagnostics mention span expectations.

## Out of scope

- full multi-label diagnostics;
- IDE/LSP integration;
- Unicode column-width rendering;
- formatter;
- changing diagnostic codes unless required for correctness.

## Validation commands

```powershell
cargo test -q
cargo test -q -p sm-front
cargo test -q -p sm-sema
smc check examples/probe-pack/probe_quad_bad_only.sm
smc check examples/probe-pack/probe_i32_add.sm
```
