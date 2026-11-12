# Q2 — Quad-Native If-Expression Type Constraint (Vertical Slice)

**Status:** experimental candidate; source-level semantics derived from frozen C0; Q2 broader scope remains open.

## Precise proposition

For one value-producing `if` expression, **each branch result type belongs to the type family inferred from the first (`then`) branch**.

The first branch establishes the comparison type, as in the pre-existing `check_expr` algorithm; it contributes a positive witness for that established type. The other branch independently supports (`T`) or refutes (`F`) the *same type-compatibility property*.

| Then compatible | Else compatible | Joined quad | Decision |
|---|---|---|---|
| none observed | none observed | N | Not admitted as a proof |
| positive | positive | T | Type consistency admitted |
| positive | negative | S | Type error, both witnesses retained |
| no positive | negative | F | Type error / unresolved under this slice |

The last two rows describe states of the reusable evidence algebra; in this narrowly implemented `if` algorithm the first branch fixes the target type, so complete successful analysis normally produces either T or S, not F alone.

## Source identity and preservation

- Proposition tuple: `("if-branch-type", expression_node_id, 0)` where scope 0 denotes the current compilation unit/node arena, not an external namespace.
- Witness IDs: `Node.pos` of the `then` and `else` expression blocks (byte offsets). Source identities/spans beyond this compilation unit are not yet represented.
- `type_branch_witness` produces a positive or negative witness based on the concrete type comparison.
- `evidence_try_join` validates identity, state integrity, duplicate policy and deterministic ordering.
- `check_expr` stores accepted merged evidence in `Sem.type_evidence: Map(i32, Evidence)` before applying the executable decision.
- **Only state T** satisfies the branch compatibility constraint. For state S, the existing `type` error category and the existing experimental compiler diagnostic text are preserved.
- `bool` is still used for independent structural facts; `quad` is used for the actual combined semantic type evidence.

## Executable evidence and qualification

- `tests/compiler/programs/quad_type_constraint_internal.sm` imports the real Sema, parses and analyzes both matching and mismatching value-producing `if` expressions, and inspects the witness positions, identities, deterministic ordering and the joined T/S outcomes directly from `Sem.type_evidence`.
- `tests/compiler/test_quad_native.py` independently builds, verifies and executes the fixture on pinned C0, then compiles valid and invalid `if` programs with the native Semantic-written compiler. Execution/diagnostic *category* is compared against pinned C0; byte-for-byte error wording is **not** required.
- Existing quad truth-table, negative, import, verifier and Harness checks must remain passing.

## Non-goals / open boundaries

This slice is not a general type-constraint solver. It does not prove an explicit ambient expected type against both branches, does not follow all propagation paths through the AST, and does not authenticate witness locations. It cannot establish C1/C2 fixed-point qualification. SemCode format and frozen C0 were not modified. The legacy `n24` subset mismatch remains a separate governance issue.

**Do not treat successful native code generation or higher diagnostic quotas as Bootstrap Seal.**

## Verified execution — 2026-10-30

Candidate source identity: experimental nine-module bundle SHA-256
`0b976482428997c5fb2e27122fc88d0e0d2764414ae0833217ca6ecf7d482cfa`
(method: domain-separated path/length/content framing in `source_snapshot_identity.py`, **not** canonical Bootstrap source-set identity).

- `compiler/sema.sm` SHA-256: `c7b4fbf09a47a453a2b7b134a5dcf2d0672a6ab46c6481977990deb231a8eec4`.
- Pinned C0 compilation: PASS; C0 `smc verify`: PASS, 49 functions, `SEMCOD23`, epoch 0.24.
- `python -B -m unittest test_quad_native test_imports -v`: **11/11 PASS**, including real `Sem.type_evidence` inspection, C0/native execution and rejection parity, source positions, T/S and fail-closed cross-proposition checks.
- Full compiler differential suite: **123/124 passed**, **9/9 mutation checks passed**, process exit nonzero. Sole mismatch: legacy `n24_unsupported_type_annotation` intentionally disallows `quad` in an outdated subset where pinned C0 admits it. This is not a blanket PASS for the qualification suite.
- Harness scope and `git diff --check`: PASS.
- Full differential evidence: `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-20261030\q2-full-differential\results.json`.

No normative Bootstrap Contract edit, C1/C2 fixed-point, source-set registration, verifier/VM modification, commit, push or PR was performed.

**Disposition:** Q2 first vertical type-constraint slice implemented and tested; broader type constraint solver, full source provenance, canonical admission and fixed-point remain open.

## Q2 extension checkpoint — 2026-10-31

Experimental coverage now includes three owner-specific type propositions:

| AST owner | Kind | Positive witness | Negative witness | Result |
|---|---|---|---|---|
| Value-producing `if` | `if-branch-type` | Branch result type equals inferred `then` type | Branch result differs from inferred `then` type | `T` admits, `S` rejects |
| Named user-function call | `call-arguments` | Actual argument type equals the corresponding parameter type | Corresponding parameter type mismatch | `T` admits, `F/S` rejects |
| Record literal or record copy-with | `record-fields` | Actual field type equals the declared field type | Field value type mismatch | `T` admits, `F/S` rejects |

Call and record proofs use each contributing argument/field AST position, in traversal order. No unrelated propositions are merged; `(kind, owner_node_id, 0)` is the identity. `accumulate_type_constraint` checks prior evidence and performs a fail-closed `evidence_try_join` before persisting the result in `Sem.type_evidence`. The first existing compiler diagnostic is retained when Sema rejects a program. `Sem.type_evidence` may therefore hold partial/negative evidence for a rejected program, but the driver never lowers it.

Actual real-Sema fixture: `tests/compiler/programs/quad_type_coverage_internal.sm`. It checks valid calls and record fields (`T`), mismatches (`S`), exact witness positions, and repeatability.

**Not fully covered in Q2:** builtins, ambient expected-type obligations, assignments, returns, tuple/sequence element constraints, match result unification, implicit inference, symbol constraints and generic relations. Zero-argument calls do not create a witness carrier because no individual type comparison occurs. This is an **expanded vertical checkpoint**, not complete coverage of the compiler's type system.

Follow-up boundaries: `QUAD_NATIVE_LOWERING_PRECONDITIONS_Q3.md` and `QUAD_NATIVE_CROSS_PHASE_PROVENANCE_Q4.md`.

## Q2.3 — Annotated bindings, assignments and explicit returns (2026-10-31)

The new experimental `compiler/sema.sm` path adds three **distinct one-witness propositions**:

| Syntax | Proposition identity | Required type | Actual type | Witness origin |
|---|---|---|---|---|
| `let x: Ty = value` / annotated mutable binding | `("annotated-binding", declaration_node_id, 0)` | Resolved `Ty` annotation | Initializer result | Initializer AST `Node.pos` |
| `x = value` | `("assignment-type", assignment_node_id, 0)` | Existing binding's declared/inferred type | Assigned expression | RHS AST `Node.pos` |
| `return value;` in a non-unit function | `("return-value-type", return_node_id, 0)` | Function declared result type | Returned expression | Expression AST `Node.pos` |

The result is **T** for one compatible witness or **F** for one incompatible witness. **S is not invented** where there is only one independently observed comparison. `accumulate_type_constraint` validates identity, witness uniqueness and exact witness polarity before adding its result to `Sem.type_evidence`.

Unannotated `let` declarations infer a type: they do **not** receive a fabricated type-equality witness. Plain `return;` in a unit function similarly has no value-type comparison. Existing diagnostics for erroneous statements are retained.

The downstream `q3_required_kind` now explicitly requires these proof kinds for the respective AST statements. `q3_source_nodes` binds their witness to exactly one `nd.a` expression source node, and `lower_stmt` copies validated statements to `FnIr.proofs` without modifying emitted SemCode.

**Proof of implementation:** `tests/compiler/programs/quad_binding_return_internal.sm` runs the real parser/Sema/Lower pipeline, checks T/F and `Node.pos` origins, verifies the IR proof snapshot, then attempts an invalid source offset and a missing evidence map. `test_quad_native.py` includes native-vs-frozen-C0 admission and executable behavior for valid and rejected programs.

**Still outside Q2:** tuple destructuring/element compatibility, sequences, match result typing, builtins, operator families, zero-argument call signatures, ownership/effect constraints, general inferred type relationships, and provenance source file identity. This is a scoped Q2 extension, **not** a claim of complete type-system coverage.
