# Q2.7 — Tuple Destructuring, Tuple Assignment, Inference Origins

**Status:** experimental vertical slice, 2026-10-31. Not full type-system Q2 closure. No frozen C0, VM/Verifier, Harness or Bootstrap Contract edits.

## AST obligations and identities

| Source form | Independent proposition or origin | Exact witness and meaning |
|---|---|---|
| `let (a,b) = rhs` (`k=40`) | `tuple-destructure-shape` in `Sem.tuple_shape_evidence` | One `rhs Node.pos`: actual inferred type is tuple with exactly two members |
| `let (a,b): (A,B) = rhs` (`k=40`, annotated) | `tuple-annotation-type` in `Sem.type_evidence` | One `rhs Node.pos`: actual tuple type agrees with required type annotation |
| `(x,y) = rhs` (`k=41`) | `tuple-destructure-shape` | One `rhs Node.pos`: actual tuple has exact target count |
| `(x,y) = rhs` (`k=41`) | `tuple-assignment-items` in `Sem.type_evidence` | One observation per target `Name Node.pos`: actual corresponding component type agrees with existing destination variable type |
| A new tuple binder `a` or `b` | `Sem.tuple_inferred_components[binding_id] = resolved component type` | **Derivation metadata, not Evidence**. Source tuple type forwards one component's type to the newly created variable |

The shape proposition is independent from assignment element compatibility. An arity mismatch yields its own `F` and keeps the original Sema diagnostic. An assignment with one matching and one incompatible component produces `S` in *one coherent proposition* `tuple-assignment-items`, preserving both witnesses; it must **not** JOIN these facts into shape or annotation identities.

A non-annotated `let (a,b)` does not impose a fabricated equality constraint: no artificial `T` carrier is created for the inferred binder type. Instead the resolved type ID is forwarded from the input tuple's `TyInfo.kids[i]`, stored separately.

## Q3.6 admission

`q3_required_kind` now requires `tuple-annotation-type` for annotated `k=40`, and `tuple-assignment-items` for nonempty `k=41`, checking exact owner/kind, T-only state, contributor count and position as usual.

`q3_tuple_proof_error` **always** requires `tuple-destructure-shape` for `k=40/41`, independent from the existence of any scalar carrier. Its state must be T with exactly one source position from `rhs`. For `k=40`, it additionally checks that the source expression's inferred type exists, is a tuple of the expected arity, each target `Name` has a recorded inferred component type matching the `TyInfo.kids` position, and each target has a destination slot.

The preflight rejects a missing/forged shape carrier or missing/wrong inferred origin before emitting an artifact. It does **not** prove arbitrary source code trustworthy, nor validate all control flow, ownership and effect constraints.

## Q4.6: strictly separate proofs and inference links

- A checked `tuple-destructure-shape` is added as `PhaseProof` to the function's non-emitted `FnIr.proofs`.
- Existing `tuple-annotation-type` / `tuple-assignment-items` are captured via the existing statement proof path.
- `FnIr.tuple_inferences: Sequence(TupleInferenceLink)` now records non-emitted **derivation links**, each containing `owner`, `source`, `binding`, `component`, `resolved_type`, `source_offset`, `binding_offset`. These are not converted to `Evidence` or merged as positive observations.
- The order follows the original binding order. Re-analyzing identical input must reproduce proof and link order and source positions.

## Verification fixture

`tests/compiler/programs/quad_tuple_inference_internal.sm` executes the real parser→Sema→Lower path, checks both independent proof families and source offsets, verifies two non-proof links for inferred variables, rejects removed or forged shape evidence and removed or mistyped inference origins, checks mixed-support `S` on assignment targets, and verifies repeatability.

`tests/compiler/test_quad_native.py` adds C0/native parity on ten legacy positive and negative tuple cases; the known `r34_tuple_expression_call` fixture is **expected to fail at runtime** due to an explicit `assert(false)`, so the test verifies C0/native parity of that expected failure rather than treating it as successful execution.

## Limits and open contracts

No generalized type inference solver or path-unification proof is implemented. Bare `let x = expr`, `if/while/assert` condition admissibility, `for-in` sequence admissibility, tuple pattern alias/ownership side conditions and other type relations remain separate Q2 obligations. Raw `Evidence` and in-memory `Sem` are constructible; exact source-offset congruence is not a cryptographic proof of truth. File-qualified origins, derivation parent chains, serialized proof transport and verifier-facing checking remain beyond Q4.6.

This experimental IR metadata does **not** change frozen SemCode bytes. No C1/C2 canonical fixed-point proof or Bootstrap Seal is claimed.
