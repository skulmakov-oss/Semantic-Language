# GAP: Match Terminal Control-Flow Recognition

Status: implementation-scope draft  
Track: Semantic Full Readiness / source semantics and lowering  
Source trigger: Probe Pack v0

## Problem

Probe Pack v0 shows that a function containing a `match` whose every arm returns can pass `smc check` but fail at compile time with:

```text
function 'classify' may exit without returning F64
```

A manual fallthrough `return <value>;` after the `match` works around the issue.

This indicates that the compiler/check path does not currently recognize fully-returning `match` expressions/statements as terminal control flow.

## Goal

Teach the source/lowering/control-flow analysis that a `match` is terminal when all admitted arms are terminal.

## Minimal reproducer

```semantic
fn classify(q: quad) -> f64 {
    match q {
        T => { return 1.0; }
        F => { return 0.0; }
        S => { return 2.0; }
        _ => { return 0.0; }
    }
}
```

Expected result after fix:

```text
check: pass
compile: pass
verify: pass
run-smc: pass
```

## Scope

Define and implement terminality rules for `match`:

- `match` with all arms returning is terminal;
- `match` with all arms trapping is terminal if trap semantics are admitted;
- `match` with at least one fallthrough arm is not terminal;
- `_` wildcard/default arm participates in totality;
- non-exhaustive match behavior must remain consistent with current source semantics.

## Required tests

Positive tests:

- `match quad` with all arms `return`;
- `match quad` with wildcard arm returning;
- nested terminal `match` if admitted;
- terminal `match` in non-`unit` function.

Negative tests:

- one arm without `return`;
- missing wildcard/default where exhaustiveness is not proven;
- unsupported match target type remains rejected;
- terminality must not hide type mismatches inside arms.

## Acceptance criteria

- fully-returning `match` no longer requires artificial fallthrough return;
- compile-time return analysis agrees with check/source semantics;
- diagnostics point to the actual non-terminal arm when terminality fails;
- generated IR/SemCode remains deterministic;
- behavior is documented in source semantics.

## Out of scope

- general exhaustiveness engine beyond currently admitted match domains;
- match expressions yielding values;
- pattern guards;
- ADT payload destructuring improvements;
- record pattern matching.

## Validation commands

```powershell
cargo test -q
cargo test -q -p sm-front
cargo test -q -p sm-ir
cargo test -q -p sm-emit
smc check examples/probe-pack/probe_match_nested.sm
smc compile examples/probe-pack/probe_match_terminal.sm -o target/probe_match_terminal.smc
smc verify target/probe_match_terminal.smc
smc run-smc target/probe_match_terminal.smc
```
