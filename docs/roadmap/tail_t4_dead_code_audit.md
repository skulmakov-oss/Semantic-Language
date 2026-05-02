# M-Tail T4 — `allow(dead_code)` Audit

Status: closed  
Date: 2026-05-02

## Scope

Audit all `#[allow(dead_code)]` attributes across `crates/**/*.rs` and classify
each as:

- **legitimate** — correctly suppressing a warning on code that is genuinely
  used but triggers a false positive (e.g. test-only types, re-export facades)
- **masking unused production code** — suppressing a real warning on dead
  production code that should be removed

## Result

| Category | Count |
|----------|-------|
| Total `#[allow(dead_code)]` attributes | 3 |
| Legitimate suppressions | **3** |
| Masking unused production code | **0** |

## Per-Occurrence Analysis

### `crates/sm-ir/src/lib.rs:20`

```rust
#[cfg(feature = "std")]
#[allow(dead_code)]
mod local_format;
```

**Classification: legitimate.**

`local_format` is a private implementation module. All of its items are
re-exported through the adjacent `pub mod semcode_format { pub use
crate::local_format::... }`. The compiler warns because `local_format` itself
is private and its individual items appear "unused" from the module perspective.
The suppression is correct; removing it would produce spurious warnings without
removing any code.

### `crates/semantic-core-exec/src/lib.rs:1582` and `1589`

```rust
#[cfg(any(feature = "alloc", feature = "std"))]
#[allow(dead_code)]
pub(crate) struct CoreProgramBuilder { ... }

#[cfg(any(feature = "alloc", feature = "std"))]
#[allow(dead_code)]
impl CoreProgramBuilder { ... }
```

**Classification: legitimate.**

`CoreProgramBuilder` is a test-support builder. Its methods are called
exclusively from `#[test]` functions (L2097, L2107, L2126, L2144, L2165,
L2194, L2227, L2247, L2261, L2286, L2328, L2383). The compiler warns because
`pub(crate)` items with no non-test callers appear dead. The suppression is
correct; the type is genuinely used in tests and removing the attribute would
produce spurious warnings.

## Conclusion

No `#[allow(dead_code)]` attribute is masking genuinely unused production code.
All three suppressions are correct false-positive silencers on:

- a private module backing a public re-export facade (`local_format`)
- a test-only builder type used exclusively in `#[test]` functions (`CoreProgramBuilder`)

T4 is closed. No further action required.
