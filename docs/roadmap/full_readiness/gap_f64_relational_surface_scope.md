# GAP: f64 Relational Operator Surface

Status: implementation-scope draft  
Track: Semantic Full Readiness / everyday expressiveness  
Source trigger: `weather_station.sm` canonical-program trial

## Problem

The weather-station trial had to define helper functions such as `gt(x, y)` and `lt(x, y)` using `abs()` because direct `f64` relational operators were not usable in that path.

That workaround makes ordinary numeric decision programs harder to read and weakens Semantic's practical programming contour.

## Goal

Define and implement direct relational operators for admitted `f64` expressions in the Rust-like executable surface.

## Candidate operator set

```text
<
<=
>
>=
```

## Required behavior

```semantic
fn high(x: f64) -> bool {
    if x > 50.0 {
        return true;
    }
    return false;
}
```

should lower through the standard pipeline:

```text
source -> sema -> IR -> SemCode -> verify -> VM
```

## Semantic policy

The implementation must explicitly define:

- accepted operand families for each operator;
- same-family restriction or admitted coercions;
- behavior for `NaN`, if `NaN` can be represented or produced;
- deterministic result for all runtime values;
- diagnostic for unsupported mixed numeric families.

## Required tests

Positive tests:

- `f64 < f64`;
- `f64 <= f64`;
- `f64 > f64`;
- `f64 >= f64`;
- relational operators in `if` conditions;
- relational operators inside composed boolean expressions.

Negative tests:

- mixed unsupported numeric family if not admitted;
- relational operation on `quad`;
- relational operation on `text`;
- malformed chained comparison if chaining is not admitted.

## Acceptance criteria

- direct `f64` relational operators are specified;
- `weather_station.sm` no longer needs `gt` / `lt` helper workarounds for ordinary comparisons;
- IR/SemCode/VM support is explicit or the lowering target is clearly documented;
- verifier admits the new operator path only under the right SemCode/capability contract;
- docs and diagnostics are updated.

## Out of scope

- broad numeric promotion system;
- vector comparisons;
- units-of-measure relational semantics beyond explicit admitted scope;
- approximate equality;
- total ordering abstraction for all values.

## Validation commands

```powershell
cargo test -q
cargo test -q -p sm-front
cargo test -q -p sm-ir
cargo test -q -p sm-vm
smc check examples/canonical/weather_station/weather_station.sm
```
