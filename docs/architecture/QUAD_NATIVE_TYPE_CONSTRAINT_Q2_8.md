# Q2.8 — Statement Conditions, For-In Type and Unannotated Binding Inference

**Date:** 2026-10-31. **Status:** local experimental vertical slice, not a normative complete type inference solver.

## New Q2 type propositions

| Source owner (AST kind) | Kind | One observed source | Required type |
|---|---|---|---|
| statement `if` (`k=12`) | `if-condition-type` | `nd.a` expression `Node.pos` | `bool` |
| `while` (`k=13`) | `while-condition-type` | `nd.a` expression `Node.pos` | `bool` |
| `assert` (`k=17`) | `assert-condition-type` | `nd.a` expression `Node.pos` | `bool` |
| `for x in ...` (`k=42`) | `for-sequence-type` | `nd.a` expression `Node.pos` | Sequence family `TyInfo.k=5` |
| value-producing `if ... else ...` (`k=26`) | `if-expression-condition-type` (separate carrier map) | `nd.a` expression `Node.pos` | `bool` |

Each **type admissibility** proposition receives a single `T` or `F` witness from an actual expression result type. In particular, `assert(false)` has type evidence **T**; the assertion's runtime failure is a distinct behavioral fact. `if` expressions already carry independent `if-branch-type` evidence for the two branch results; their condition Evidence is stored separately in `Sem.if_expr_condition_evidence` to prevent cross-proposition JOIN and artificial `S`.

If semantic analysis fails before a meaningful type is produced (unit, unresolved, malformed), no false positive is fabricated; existing fail-closed Sema diagnostics remain the authority.

## Inferred unannotated bindings

`Sem.inferred_binding_types[let_statement_id]` stores the actual resolved initializer type for each successful unannotated `let x = rhs`, including scalar literals, variable references, calls and compound expressions. This is **type derivation metadata, not a positive compatibility proof**; with no explicit required type, the compiler has nothing to equate against.

No false `T` carrier is created to certify simple inference. The existing `TupleInferenceLink` mechanism remains separate for tuple-component inference.

## Q3.7–Q3.8 admission

- `q3_required_kind` now requires the four statement-level condition and iterable type proofs. `q3_source_nodes` maps each to the actual condition/iterable AST expression, and the existing general gate requires exact owner identity, T-only state, one source-congruent witness.
- `q3_binding_inference_error` is mandatory for unannotated `k=10`. It requires a recorded inferred type, valid non-unit type ID, destination slot and equality with a separately recovered RHS inferred type. Literal AST kinds have fixed type IDs; other expression results use `Sem.exprty`.
- `q3_control_type_error` independently recomputes the observed expression family from the current typed AST: boolean family 2 for conditions, Sequence family 5 for for-in. A well-shaped T witness and correct `Node.pos` alone are therefore **insufficient** if the actual typed expression does not satisfy the condition.
- Value-producing `k=26` additionally requires a separate, valid T-only `if-expression-condition-type` proof, with its own exact identity and source offset; deletion of this one carrier must fail even when `if-branch-type` remains valid.

This checks the source-level typed AST and its in-memory Sema metadata; it does not authenticate arbitrary independently forged `Sem.exprty` or carrier values.

## Q4.7–Q4.8 (non-emitted IR origin)

- `FnIr.binding_inferences: Sequence(BindingInferenceLink)` stores `owner`, initializer source node, resolved type, source-local initializer byte offset and binding location. **Not** Evidence and never JOINed with propositions.
- `FnIr.proofs` retains the ordinary T-only statement condition/for-in proofs alongside pre-existing operator and type proofs.
- `FnIr.control_type_checks: Sequence(ControlTypeCheck)` records condition/iteration owner AST kind, source node, observed resolved type, observed family, required family, source position and the validated Evidence carrier. Captured for statement control owners and value-producing if expressions. This makes revalidation inspectable after Sema→Lower inside the experimental compiler, but is deliberately **not emitted** into SemCode.
- Q4 snapshots follow deterministic lowering traversal and do not change the frozen VM or verifier ABI.

## Regression coverage

`tests/compiler/programs/quad_control_inference_internal.sm` exercises real parser/Sema/Q3/Q4 for T/F control cases, valid `assert(false)` type, inferred scalar/container bindings, Q3 failure on missing or altered inference metadata, source mismatch, erased condition carrier, changed independently derived type, and separate if-expression condition/branch propositions. It inspects `FnIr.binding_inferences` and `FnIr.control_type_checks` and verifies replay.

`test_quad_native.py` adds 12 native-vs-frozen-C0 examples, including valid/invalid conditions, loops, assertions, and inferred `bool`, `text`, `Sequence` and function-return types. Accepted cases are verified and executed on the pinned frozen C0.

## Remaining boundaries

This completes **the listed Q2.8 surface**, not all of Q2: general unification, generic/inferred type variables, index operand/collection proof, ownership/effect and full symbol-level proof semantics are not covered. Q3/Q4 currently provide typed-AST consistency, not unforgeable proof production, file-qualified source spans, transformation genealogy or verifier-visible evidence. Separate governance is required before modifying SemCode or frozen verifier contracts. C1/C2 canonical fixed point is not established here.
