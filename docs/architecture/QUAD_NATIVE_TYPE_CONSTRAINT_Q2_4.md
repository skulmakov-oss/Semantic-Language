# Q2.4 — Tuple, Sequence and Match Type Evidence

**Checkpoint:** experimental, local, 2026-10-31. **Not** all Q2 type constraints implemented.

## Three additional proposition families

| AST owner | Proposition identity | Positive / negative witness | Expected type source |
|---|---|---|---|
| `k=22` tuple with known expected tuple type | `("tuple-target-items", node_id, 0)` | Each tuple item expression position supports / refutes the corresponding required tuple component type | Inherited expected `TyInfo.k=6` passed into `check_expr` |
| `k=23` nonempty sequence literal | `("sequence-elements", node_id, 0)` | Each element expression position supports / refutes the common element type | `Sequence(T)` context, or type inferred from the first element |
| `k=25` match expression with checked arms | `("match-arm-types", node_id, 0)` | Each arm **result expression** position supports / refutes the common match result type | Expected context, otherwise result type of first arm |

All three use `accumulate_type_constraint` with checked `EvidenceResult` JOIN and deterministic witness ordering. A mixed support/refutation set is `S`, a single unsupported comparison is `F`, all supporting witnesses are `T`. Original compiler diagnostic text and category remain authoritative.

## Important distinction for tuples

An unconstrained tuple `(i32, bool)` is valid: the components **must not** be compared against each other. An expected tuple type is registered in the private experimental `Sem.tuple_expected: Map(i32, i32)`, and only a *contextual* tuple gets `tuple-target-items` evidence. The Q3 preflight derives this requirement from the registered target metadata. This registry is a data flow marker, **not** an ADT-level authority or an independently verified expected-type derivation.

For a tuple with an incompatible expected arity, the ordinary outer type-checker remains responsible for the error; a successful program must satisfy any registered Q3 obligation before lowering. An unannotated/inferred tuple does not receive fabricated evidence.

## Sequence and match details

A nonempty sequence inferred without an annotation has its first item as the target-type anchor. That first positive witness is an inference anchor, not evidence of an externally declared obligation. Every later item is checked against the same type. An empty annotated `[]` has no observed element comparisons and therefore creates no synthetic per-element proof; its type inference requires context as before.

A match proposition aggregates value-producing arm results; the first arm establishes the inferred target when no expected context is present. This does not replace pattern validity, wildcard/exhaustiveness or `Option` arm rules. A statement-form `match` that does not perform a value-type unification is not assigned fictitious value-result evidence.

## Q3 + Q4

`compiler/lower.sm` extends `q3_required_kind` and `q3_source_nodes` for the three families. Tuple and sequence contributors are their literal child expressions. Match contributors are each arm's result expression, **not** the pattern or whole arm. Required `Evidence` must be valid, exact-identity, T-only, full-count and position-matched before code generation. The actual `lower_expr` path records them in `FnIr.proofs` with original AST source IDs and offsets. **No SemCode ABI/VM/verifier changes**.

## Verification

- Frozen C0 builds `tests/compiler/programs/quad_container_match_internal.sm`; Verifier and bounded C0 execution PASS.
- The fixture inspects direct Sema `T/F/S`, contributor positions, absence of phantom inferred-tuple and empty-sequence evidence, Q3 missing/faked-proof rejection and Q4 `FnIr.proofs`.
- `test_quad_native.py` checks native/C0 admission parity and C0 verifier/execution for **9** good/bad standalone programs covering all three families.
- Full local/differential/harness qualification is tracked separately in `QUAD_NATIVE_Q2_4_CHECKPOINT_20261031.md`.

## Remaining limitations

`Sem.tuple_expected` is mutable structural metadata. Q3 compares witness offsets to the same in-memory AST and cannot prove the truth of independently forged witness claims. Neither source-file identity nor cross-optimization proof lineage is present. Q2 still does not cover builtin signatures, arithmetic/logical operators, all tuple destructuring constraints, symbol uniqueness, inferred-type equivalences and ownership/effect constraints.

**Do not claim full Q2, Q3, Q4, Bootstrap Seal, or C1/C2 fixed-point based on this vertical slice.**

## Match obligation registration safety refinement

The real `check_expr` path for value-producing `match` now writes `Sem.match_result_checked[owner_node_id] = true` **before** evaluating the match arms. It intentionally registers the obligation even when a later arm is rejected, so negative `S/F` evidence remains classifiable. This registration is separate from `Sem.type_evidence`; removing only a match `Evidence` carrier does **not** remove the lowering requirement.

Q3 now requires `match-arm-types` for every `k=25` node registered in `match_result_checked`. A statement-form match not subjected to a value-result unification has no registration, preserving ordinary statement match semantics. The real Sema→Lower negative fixture removes only the match evidence while retaining other valid carriers and confirms fail-closed rejection. As with `Sem.tuple_expected`, removing *both* the independently mutable registry and Evidence is not prevented by this experimental data model: a trusted construction boundary is still needed.
