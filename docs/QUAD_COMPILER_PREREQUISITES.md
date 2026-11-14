# Native Quad compiler prerequisites

Status: reconciled on 2026-11-14 with the compiler on `main` after PR #63. The original note was
written with PR #62, when the compiler rejected `quad` as an unsupported annotation. Its
rationale is kept below; its "not available" statement no longer describes the compiler.

## Why the VM kernel uses a record

The clean-slate VM kernel in `vm/main.sm` represents a four-state value as a nominal `Quad` record
with two Boolean planes. It was written that way because, at the time, the compiler's primitive
`quad` type was not available, and encoding the state as an integer would have hidden the state
semantics. The kernel still uses the record today.

## Status of the six prerequisites

| # | Prerequisite | Status | Evidence |
|---|---|---|---|
| 1 | Canonical `N`, `F`, `T`, `S` literal tokens and AST representation | implemented and tested | `compiler/parser.sm` (literal node kind 9); `tests/compiler/test_quad_native.py` `test_full_quad_truth_4x4_and_match_vs_c0` |
| 2 | A primitive Quad type identity in `compiler/sema.sm` | implemented and tested | type id 5; corpus case `p24_quad_type_annotation` |
| 3 | Quad expression typing and plane operations, with no coercion to `bool` | implemented and tested for `==`, `!=`, `&&`, `\|\|`, `!` and `match` | supported subset in [COMPILER.md](COMPILER.md); `test_negative_boundaries` (bare `quad` condition, mixed `bool`/`quad`) |
| 4 | A Quad value family and typed control lowering in `compiler/lower.sm` | implemented and tested | `lower_expr`; `test_terminal_unit_match_has_valid_jump_target` |
| 5 | Extend `compiler/semcode.sm` only after the upstream SemCode opcode and revision contract is approved | satisfied through the existing upstream contract | the compiler emits only opcodes that pinned C0 already defines: `LOAD_Q`, `Q_AND`, `Q_OR`, `Q_NOT` and the existing equality comparison. No opcode and no SemCode revision was added |
| 6 | Compiler fixtures for all four states and negative cases for implicit Boolean coercion | implemented and tested | `test_full_quad_truth_4x4_and_match_vs_c0` (16 joins, 16 meets, 4 negations, match), `test_negative_boundaries` |

What the compiler admits is stated in [COMPILER.md](COMPILER.md), "Static rules". How the
compiler uses four-state values internally is stated in
[architecture/QUAD_NATIVE_EVIDENCE_CONTRACT.md](architecture/QUAD_NATIVE_EVIDENCE_CONTRACT.md).

## Still open

- **The VM kernel does not consume native `quad` values.** `vm/main.sm` keeps its `Quad` record.
  Nothing has replaced it with the primitive type, and no test shows the kernel running a program
  compiled from native `quad` source.
- **No SemCode loader in the VM kernel.** [`vm/README.md`](../vm/README.md) states that the kernel
  does not claim SemCode compatibility and starts above the legacy decoder. That is unchanged:
  the kernel executes its own instruction record, not artifacts produced by `compiler/`.
- **Operations outside the admitted subset.** Truth-table operators distinct from the knowledge
  meet and join, and any conversion between `quad` and `bool`, are not implemented in the
  compiler.

## Independent limitations of the Semantic VM

The `vm/` implementation is an experimental, non-canonical track (Issue #29). The existence of
native Quad compilation in `compiler/` does not give it canonical VM or verifier authority:

- artifacts produced by the compiler are verified and executed by pinned Frozen C0, not by `vm/`;
- the differential corpus and the self-compilation checkpoint in the README were qualified against
  Frozen C0 only;
- `vm/` is not a prerequisite for, and not evidence of, the contractual bootstrap proof.

Any claim that `vm/` executes compiler output needs its own evidence and its own approval of the
opcode and revision contract.
