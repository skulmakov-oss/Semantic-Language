# Q2.8 / Q3.8 / Q4.8 — Control Preconditions, Binding Inference, Typed AST Audit

**Date:** 2026-10-31. **Branch:** `feat/quad-native-shf15`. **Base HEAD:** `972066b25d2201344747becff0a0d6abfeb0bf4d`. **Disposition:** local unpublished and uncommitted experimental vertical slice.

## Implementation summary

1. **Q2.8 source semantics:** One proposition per AST owner, for statement `if`, `while`, `assert`, `for-in`, and separately for `if` expression condition. `T` means the observed expression type is admissible, not that a condition evaluates true at runtime. `assert(false)` is well-typed and will still fail at runtime. `for-in` requires an actual Sequence family type.
2. **Q2.8 inference:** `Sem.inferred_binding_types` records the resolved RHS type for each successful unannotated `let`. No fabricated positive compatibility proof exists without an expected type. Tuple component origins remain separately represented from Q2.7.
3. **Q3.7 source proof:** The four statement kinds require their own T-only, exact AST-owner, exact one-witness carrier. A value-producing `if` expression has an independent `Sem.if_expr_condition_evidence` carrier, separate from branch-result `if-branch-type` evidence. Deleting only one condition carrier is rejected.
4. **Q3.8 independent check:** `q3_control_type_error` recomputes the observed source-expression type family from the current typed AST and requires bool or Sequence family. An otherwise valid, position-congruent positive Evidence cannot bypass a mismatched expression type. `q3_binding_inference_error` independently checks recorded unannotated binding type against the typed RHS and presence of a destination slot. Both must pass before lowering.
5. **Q4.7/Q4.8:** `FnIr.binding_inferences` contains non-proof derivation links. `FnIr.control_type_checks` stores the actual typed AST source, source offset, observed type and family, required family and checked Evidence for each control owner. `FnIr.proofs` carries T-only ordinary statement condition witnesses. All are non-emitted, and the frozen SemCode encoding and verifier have not changed.

## Direct qualification evidence

- Frozen C0 compiled the experimental candidate `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-q28\hardened.smc`. Its self-contained internal fixture `quad_control_inference_internal.sm` built and was accepted by frozen verifier and VM. Separate experimental research execution cap: 2,200,000 Steps / 50,000 Calls / 256 Frames.
- `tests/compiler/test_quad_native.py` adds two tests: a real Sema→Lower fixture and **12** native-vs-frozen-C0 admission/execution examples for good/bad conditions, loops and `let` type inference.
- Negative fixture checks missing/incompatible inferred types, missing T carrier, wrong origin, independent if-expression condition-vs-branch proof identities, cross-proposition JOIN rejection and independently recomputed expression family.
- Two existing Q4 fixtures were updated to account for the additional `assert-condition-type` proof while preserving all individual source-position checks.
- The final full unit/import and differential results are recorded in the appended qualification block after completion; the earlier differential was run before Q3.8 and **is not** the final evidence.

## File identity (SHA-256)

- `compiler/sema.sm`: `140a076f5ce47a16b9499a9707128e1c41e3401d73c95cd6c2149d84db8a254a`.
- `compiler/lower.sm`: `5766ef1a25f5512e88a3e309f8f7ad7e307e85764e10919a55422206f6c7a3d5`.
- `tests/compiler/test_quad_native.py`: `5d92fb129626f3f44563c136d774d8ee54eb588930b5488e79c6bd91e5bbcfe4`.
- `tests/compiler/programs/quad_control_inference_internal.sm`: `b4fe06b70b866fb8423eec4c0eb9cb670564f91b8929d3b0a6a1e889d6a82686`.
- `tests/compiler/programs/quad_binding_return_internal.sm`: `f64bbf0b23800b34dc341511278cbe93e951540ab2c739e42d7effe0f0d8647a`.
- `tests/compiler/programs/quad_lowering_provenance_internal.sm`: `4473aaceff4bbc2b4893becef909ea3af351fb78e5096d4bf9dff85b5b666c80`.
- Experimental nine-module content identity: `f5522908dff8fa6d307ebc3995dc1b9f9d884ba18c43dce3cdcf44b31f05381a` (**not** canonical Bootstrap source set identity).

## Explicit open boundaries

- Raw `Evidence` and typed AST metadata in `Sem` remain mutable and forgeable. Rechecking an independently stored expression type is stronger than checking an offset but does not prove type truth against an adversary who can forge both the type mapping and carrier.
- This is **not full Q2/Q3/Q4 closure**: index/collection obligations, generics, whole-symbol derivation, ownership/effect, multi-file source provenance, authorized proof construction/ADT, transformation ancestry and verifier-visible serialization are not established.
- The known legacy `n24_unsupported_type_annotation` expectation differs from experimental quad-native admission; it requires separate owner-governed resolution, never silent fixture editing.
- Normative C1/C2 fixed-point qualification, quota validation and Bootstrap Seal have not been performed. No git commit, push, PR, merge or changes to frozen C0, VM, verifier, Harness scripts or Bootstrap Contract were made.

## Final qualification (same hardened source candidate)

| Gate | Result |
|---|---|
| Pinned C0 compiler build | PASS |
| Frozen C0 Verifier, hardened candidate | **PASS: 72 functions, SEMCOD23, epoch 0.24** |
| New targeted Q2.8 / Q3.8 / Q4.8 cases | PASS: 2 new tests, 12 standalone C0/native comparisons and real Sema→Lower negative checks |
| Whole quad-native + import Python suite | **26/26 PASS** |
| Whole compiler differential | **123/124 PASS**; only previously known legacy `n24_unsupported_type_annotation` mismatch |
| SemCode mutation tests | **9/9 PASS** |
| `git diff --check`, Harness | PASS before final append; recheck below |
| Canonical C1/C2 fixed point / Bootstrap Seal | **NOT RUN** |

- Differential artifact: `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-q28\final_differential\results.json`; the differential process's own result is nonzero because the existing `n24` contract mismatch remains unresolved. This is **not** a complete green qualification.
- The frozen C0 golden quad differential provenance SHA-256 remains `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`. Current golden test metrics: **812,163 Steps / 11,920 Calls / 12 peak Frames / 3 Effect Calls**. Q2.7 recorded 679,352 Steps / 8,841 Calls; this increased cost requires a controlled future performance investigation, not an assumption of compiler regression.
- Final code identity and experimental source bundle remain as recorded above, from the same code candidate. No tracked files in frozen C0, runtime, verifier, harness or Bootstrap Contract changed.
- **Owner-governed follow-up:** resolve the n24 legacy-vs-quad admission expectation; enumerate remaining type-system obligations beyond this specific surface; design opaque or provenance-authorized carrier construction, file-qualified origin IDs and verifier-visible proof ABI separately under permitted governance; only then seek normative C1/C2 fixed-point qualification.

**Disposition: Q2.8/Q3.8/Q4.8 accepted as an experimental vertical checkpoint only. No commit, push, PR or merge.**

### Post-documentation scope recheck

- `git diff --check`: PASS.
- `scripts/harness-check.ps1`: PASS, **41 changed paths within authorized scope**.
- Branch remains `feat/quad-native-shf15`; frozen base HEAD remains `972066b25d2201344747becff0a0d6abfeb0bf4d`.
