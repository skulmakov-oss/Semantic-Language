# Q2.5 — Non-map Builtin and Operator Type Evidence

**Status:** experimental vertical slice, not full Q2. Frozen C0 / ABI / verifier / VM / Harness / Bootstrap Contract are outside the modification boundary.

## Proposition identities

All owners use `(proposition_kind, AST_owner_node_id, 0)`. Each contributing witness is the corresponding operand/argument `Node.pos` within this one compilation unit.

| Node | Proposition kind | Observation |
|---|---|---|
| Unary operator `k=5` | `unary-operator-type` | One operand type is admissible for the specific unary operator |
| Binary operator `k=6` | `binary-operator-types` | Each operand satisfies the operator-family type constraint; the second also matches the first type |
| Named non-map builtin `k=7` | `builtin-argument-types` | Each argument type is admissible for the selected builtin and, where needed, matches a dependent input element type |

For `-` only `i32` is accepted; for `!` only `bool` or `quad`. Binary arithmetic allows matching `i32` or `u32`, and text concatenation is allowed for its existing `+` opcode only. Equality uses matching scalar families `i32/bool/u32/text/quad`. Ordering requires matching `i32` or `u32`. Logical operators require matching `bool` or `quad` for the permitted knowledge operations.

**Do not equate type equality with operator admissibility.** `bool + bool` yields two negative observations (`F`), `i32 + bool` yields `S` with one supporting and one negative observation, while `i32 + i32` yields `T`. A single unsupported unary operand yields `F`. No contradiction is fabricated from unrelated propositions.

Covered non-map builtin IDs: `len, push, pop, to_text, text_len, text_byte_at, text_slice, args_read, fs_read_text, stdout_write`. The tests sample several of them, and the classifier covers all ten. Each has an arity check and existing original type diagnostic; evidence only attaches after successful arity checking and successful non-unit argument-type resolution. Dependent `push` compatibility is checked against the first argument's Sequence element type, whereas `len/pop` require a Sequence operand.

## Q3 / Q4

The existing Q3 admission rule requires a structurally valid, exact-owner, exact-kind, `T`-only carrier and exact AST source offsets before lowering. Unary operations require one AST operand, binary operations require both AST operands, and a covered builtin requires each AST argument. For builtins the required kind is recovered from the compiler's `Sem.callees` builtin identity, **not** from the presence of `Evidence` itself. Removing the evidence carrier without removing the independent builtin callee metadata fails Q3.

The existing `lower_expr` path adds every admitted operator/builtin proof to `FnIr.proofs` (non-emitted Q4 PhaseProof with AST node IDs and source-local offsets). The SemCode bytes and frozen verifier contract are unchanged.

## Tests

`tests/compiler/programs/quad_builtin_operator_internal.sm` imports the real Sema and Lower, observes `T/F/S`, polarity families, AST positions, Q3 tamper rejection and Q4 IR provenance. `tests/compiler/test_quad_native.py` compares native candidate with pinned frozen C0 across 14 independently compiled valid and invalid operator/builtin programs; valid artifacts are verified and executed on frozen C0.

## Limits

The compiler uses an in-memory, constructible `Evidence` record. The carrier and Q3 position checks do **not** authenticate claims against a forged but otherwise congruent positive carrier. Builtin identity metadata can also be fabricated if an attacker constructs Sema internals. The model does not yet cover `map_empty/map_contains/map_get/map_set`: these builtins have dependent map type, key and value constraints, including a context-sensitive zero-argument constructor. They require a separately specified evidence shape and Q3 obligation policy.

No file-qualified source spans, optimization derivation graph, verifier-visible proof transport, canonical C1/C2 fixed point, or Bootstrap Seal is established by Q2.5.
