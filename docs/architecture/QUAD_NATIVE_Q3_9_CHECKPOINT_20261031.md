# Q3.9 — Canonical Replay Evidence Admission Checkpoint

**Date:** 2026-10-31. **Branch:** `feat/quad-native-shf15`; **HEAD:** `972066b25d2201344747becff0a0d6abfeb0bf4d`. **Worktree:** local, uncommitted and unpublished.

## Authorized development surface

`compiler/lower.sm` and `compiler/driver.sm` only, plus targeted compiler fixtures and architecture documentation. Frozen C0, verifier, VM, harness scripts and Bootstrap Contract unchanged. No SemCode proof extension.

## Implementation

- `q3_trusted_admit(candidate: Sem)` repeats the trusted `analyze(candidate.prog)`, compares canonical Sema's complete evidence families and AST-indexed type/callee/slot/inference metadata with the candidate, runs `q3_lowering_error` on the canonical replay, then returns `TrustedSemAdmission`.
- `q3_same_evidence` checks full ordered positive and negative witness sequences, complete proposition identity, state and structural validity; `q3_evidence_map_equal` checks presence as well as contents.
- `q3_ty_table_equal` checks actual type table elements; function metadata and scalar context are also compared. AST-indexed maps cover all Q2.1–Q2.8 evidence and inference surfaces.
- **Critically, `compile_source` lowers only `admission.canonical` and encodes using the replayed arithmetic mode.** No caller-supplied carrier is forwarded into production IR.
- An original Sema error remains a rejection and cannot be bypassed by clearing `sem.err` plus manufacturing structurally valid `T`.

## Focused acceptance

`quad_trusted_admission_internal.sm` runs actual analyzer + lowering. Tamper cases replace a witness position, falsify inferred type metadata, change actual expression typing, substitute an unrelated valid map-key witness for map-value, forge tuple source position, and construct a valid-looking positive binary type proof from a rejected `true + false` expression with the error manually cleared. All replay admissions are rejected.

`test_quad_native.py` adds eight native-vs-frozen-C0 positive and negative programs verifying that `compile_source` still emits accepted code and rejects ill-typed code. The targeted two tests passed, and pinned frozen C0 compiled/verified the new internal fixture.

## Recorded identities (SHA-256)

- `compiler/sema.sm`: `140a076f5ce47a16b9499a9707128e1c41e3401d73c95cd6c2149d84db8a254a` — unchanged from Q2.8.
- `compiler/lower.sm`: `273363e9f2e45a182d18c52ce4d4d38fdc966f97d88322e35b594e82094efe89`.
- `compiler/driver.sm`: `febdf349bcbf48e53c02285c87a86bbcf95f52a33101e1d9f40521f5be2d04ba`.
- `tests/compiler/test_quad_native.py`: `d6ba4c44838ea4bc7a05b9b79489f804a50e7056a3b1f21b78557ca9a9c0de5d`.
- `tests/compiler/programs/quad_trusted_admission_internal.sm`: `fcabd8a298cbb502852f5554401adceda10577c2f3f6619f874845574f90605a`.
- Experimental nine-module source content identity: `dc37e63ef09e66f9792ffdc2df344417eb626a56984647d9cf04812cb52f72a5` (**not** canonical fixed-point source set identity).
- Golden quad differential baseline provenance: `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`, if unchanged after final tests.

## Important limitations

**Q3.9 is trusted replay admission, not a genuinely unforgeable Evidence constructor.** Semantic currently exposes public record construction/updates, including `TrustedSemAdmission`, and `lower_function` may be directly invoked by experiments without entering the production driver. Therefore the trust guarantee applies to `compile_source(text)` only, given the trusted implementation of `analyze` and an authentic parsed AST.

Replay trusts the AST carried in `candidate.prog`; it does not bind that AST to independently authenticated source bytes, file IDs, module graph or cryptographic provenance. In-memory AST or trusted compiler source tampering is outside this boundary. Comparisons run on valid AST node IDs; synthetic out-of-domain extra metadata is discarded when canonical state is forwarded. Language-level private ADT constructors, authenticated provenance/identity and verifier-visible proof transport remain future Q3/Q4 milestones.

This safety barrier costs an additional full analyzer replay and metadata comparison per candidate. Measured example: compiler differential `a12_mini_lexer_workload` rose from Q2.8's ~420,566 to ~607,384 VM Steps (approximately +44.4%), with other workload effects to be assessed. This is not a controlled standalone microbenchmark.

## Full qualification

Final unit/import suite, full differential, mutation checks, Harness recheck and post-documentation status are recorded below once finished. **C1/C2 fixed-point proof and Bootstrap Seal: NOT RUN.**

**No commit, push, PR, merge or protected contract modification.**

## Final local qualification — 2026-10-31

| Gate | Result |
|---|---|
| Frozen C0 built Q3.9 candidate | **PASS** |
| Frozen Verifier of compiler candidate | **PASS**, 79 functions, `SEMCOD23` epoch `0.24` |
| Standalone Q3.9 test and 8 C0/native admission examples | **2/2 PASS** |
| Full quad-native/import unit suite | **28/28 PASS** |
| Full differential | **123/124 PASS**; one known `n24_unsupported_type_annotation` contract mismatch |
| SemCode mutation checks | **9/9 PASS** |
| `git diff --check` and Harness | PASS before final report appendix, rerun below |
| Canonical C1/C2 fixed point, Bootstrap Seal | **NOT RUN** |

Verified differential report: `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-q39\differential\results.json`. This runner returns a nonzero differential RC due to `n24`: **do not claim a full green compiler qualification**.

Golden quad-native artifact SHA-256 unchanged: `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`. Golden test metrics after Q3.9: **1,220,136 Steps; 23,554 Calls; 13 peak Frames; 3 Effect Calls**. Q2.8 baseline: **812,163 Steps; 11,920 Calls; 12 Frames; 3 Effects**. Thus this workload incurs **+50.2% VM Steps** and **+97.6% Calls**, without any optimization or standard benchmark claim. Large lexical workload `a12_mini_lexer_workload`: 607,384 Steps vs 420,566 (+44.4%). Explicit future work: engineering optimization of replay once security semantics are frozen and benchmarked.

**Disposition:** Q3.9 canonical-replay admission accepted as a limited experimental production-driver trust boundary. It is **not** an ADT/unforgeability proof and no verifier-facing proof transport is provided. Frozen C0 and protected governance unchanged. No commit, push, PR or merge.
