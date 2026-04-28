# GAP: i32 Arithmetic Surface

Status: implementation-scope draft  
Track: Semantic Full Readiness / everyday expressiveness  
Source trigger: Probe Pack v0

## Problem

Probe Pack v0 shows that `i32` currently behaves as a partially executable type:

- `let x: i32 = 5; if x == 5` passes;
- `let a: i32 = -1` fails;
- `let c: i32 = a + b` fails;
- the error for `a + b` says `f64 arithmetic requires f64 operands, got I32 and I32`.

This means `i32` exists for literals/equality, but ordinary arithmetic is not admitted in the practical programming contour.

## Goal

Implement and document the minimal `i32` arithmetic surface required for ordinary stateful programs.

## Required operator set v0

```text
unary -
+
-
*
```

Optional follow-up, not required for v0:

```text
/
%
checked/wrapping variants
```

## Required behavior

```semantic
fn add_i32(a: i32, b: i32) -> i32 {
    return a + b;
}

fn neg_i32(a: i32) -> i32 {
    return -a;
}
```

should pass through:

```text
source -> sema -> IR -> SemCode -> verify -> VM
```

## Semantic policy to decide

Before implementation, define:

- overflow behavior;
- division/modulo status if deferred;
- unary minus behavior for `i32::MIN` if relevant;
- whether `u32` arithmetic is part of this slice or a separate slice;
- diagnostics for mixed numeric families.

## Required tests

Positive tests:

- `i32 + i32`;
- `i32 - i32`;
- `i32 * i32`;
- unary `-i32`;
- arithmetic in `return`;
- arithmetic in `let` initializer;
- arithmetic inside `if` via equality check.

Negative tests:

- unsupported mixed `i32 + f64` if no coercion is admitted;
- `quad + quad` remains rejected;
- `text + i32` remains rejected unless text concatenation is separately admitted;
- overflow behavior follows the documented policy.

## Acceptance criteria

- `i32` arithmetic has explicit source/type/IR/SemCode/VM behavior;
- error message no longer routes `i32 + i32` through a misleading f64-only arithmetic path;
- Probe Pack `probe_i32_arithmetic.sm` can be updated to pass once relational operators are also admitted;
- negative tests prove unsupported domains remain rejected;
- docs/spec updates describe admitted operators and overflow policy.

## Out of scope

- broad numeric promotion;
- arbitrary precision integers;
- vector arithmetic;
- units-of-measure arithmetic;
- text concatenation;
- Map/Sequence utility layer.

## Validation commands

```powershell
cargo test -q
cargo test -q -p sm-front
cargo test -q -p sm-sema
cargo test -q -p sm-ir
cargo test -q -p sm-vm
smc check examples/probe-pack/probe_i32_arithmetic.sm
```
