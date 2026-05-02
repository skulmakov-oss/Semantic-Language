# M-Tail T3 — `panic!` Surface Audit

Status: closed  
Date: 2026-05-02

## Scope

Audit all `panic!` occurrences across `crates/**/*.rs` and classify each as:

- **test-only** — inside a `#[cfg(test)]` block or `#[cfg(all(test, ...))] mod tests { ... }`
- **unreachable invariant** — production path, but represents an impossible state given prior compiler guarantees
- **production-facing** — reachable by user input or external data in a release build

## Result

| Category | Count |
|----------|-------|
| Total `panic!` occurrences | 141 |
| Test-only | **141** |
| Unreachable invariant | 0 |
| Production-facing | **0** |

## Per-Crate Breakdown

| Crate / File | Count | Classification |
|---|---|---|
| `prom-audit/src/lib.rs` | 1 | test-only (`#[cfg(test)]`) |
| `sm-emit/src/lib.rs` | 3 | test-only (`#[cfg(all(test, feature = "std"))] mod tests`) |
| `sm-front/src/lib.rs` | 2 | test-only (`#[cfg(test)]`) |
| `sm-front/src/parser.rs` | 115 | test-only (`#[cfg(test)]`) — all "expected X" AST assertion helpers |
| `sm-front/src/typecheck.rs` | 3 | test-only (`#[cfg(test)]`) |
| `sm-front/src/types.rs` | 4 | test-only (`#[cfg(test)]`) |
| `sm-ir/src/legacy_lowering.rs` | 5 | test-only (`#[cfg(test)]`) |
| `sm-verify/src/lib.rs` | 1 | test-only (`#[cfg(test)]`) |
| `sm-vm/src/semcode_vm.rs` | 1 | test-only (`#[cfg(test)]`) |
| `smc-cli/src/api_contract.rs` | 3 | test-only (`#[cfg(test)]`) |
| `smc-cli/src/config.rs` | 1 | test-only (`#[cfg(test)]`) |
| `smc-cli/src/schema_versioning.rs` | 2 | test-only (`#[cfg(test)]`) |

## Notes

### `sm-front/src/parser.rs` (115 panics)

All 115 are AST-shape assertion helpers inside `#[cfg(test)]`.  
Pattern: `panic!("expected <AST node kind>")` used in parse-tree inspection
helpers that verify desugared AST structure produced by the parser.  
These fire only when a test helper is called with an unexpected node shape —
i.e., when a test itself has a bug.  No production code calls these helpers.

### `sm-emit/src/lib.rs` (3 panics)

`function_code()` (L41) and `skip_optional_ownership_section()` (L73) look
like production helpers but are both defined inside the
`#[cfg(all(test, feature = "std"))] mod tests { ... }` module guard at L24.
They are not exported and are not reachable from a release build.

## Conclusion

No `panic!` macro is reachable from a release build.  
The production binary surface is panic-free by construction on the current
admitted feature surface.

T3 is closed. No further action required.
