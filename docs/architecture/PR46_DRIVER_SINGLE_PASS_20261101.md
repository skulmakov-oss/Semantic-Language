# PR #46 — single-pass driver performance checkpoint (2026-11-01)

## Scope and trust boundary

Owner-requested review of redundant work in `compiler/driver.sm`, in `compile_source_with_origins`. The previous ordinary source path performed:

1. `lex(src)` then `parse_program(tokens)`.
2. `q4_parsed_identity_error(src, original)`, replaying full lex/parse and comparing all AST nodes.
3. `analyze(original)`.
4. `q3_trusted_admit(sem)`, replaying `analyze(sem.prog)`, comparing semantic metadata and executing the lowering guard.

The source loader first performs `q4_source_bundle_error(src, files, spans, true)`, checking source bytes and re-reading declared files; the resulting AST and Sema objects are freshly constructed from validated bytes. They are not received from untrusted code. A replay of these exact locally constructed values therefore adds execution cost without validating another trust boundary.

## Implementation

- Remove the *repeated invocation* of `q4_parsed_identity_error` only on the ordinary compiler path. Retain the function itself for explicit forged-AST/provenance tests.
- Replace the *repeated invocation* of `q3_trusted_admit(sem)` on that same path with `q3_lowering_error(sem)`, retaining complete canonical proof validation before lowering. Keep `q3_trusted_admit` unchanged for externally constructed/tampered Sema tests.
- Preserve existing `q4_source_bundle_error(..., true)` file-byte re-read and per-function `q4_trace_function` checks; do not weaken provenance, IR evidence or output admission.
- No change to `sema.sm`, `lower.sm`, `semcode.sm`, C0, VM, verifier, Bootstrap Contract, Harness or any governance surface.

## Measured effects versus the same prior corpus

All four outputs are byte-for-byte identical (SHA-256 checked against the prior full 125-case test artifacts).

| Case | Baseline Steps | Optimized Steps | Steps saved | Baseline Calls | Optimized Calls | Calls saved |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| p01_empty_main | 66,364 | 61,929 | 6.68% | 96 | 54 | 43.75% |
| p03_recursive_factorial | 214,826 | 155,193 | 27.76% | 2,407 | 1,422 | 40.92% |
| a12_mini_lexer_workload | 800,409 | 512,839 | 35.93% | 10,188 | 5,725 | 43.81% |
| p24_quad_type_annotation | 78,695 | 69,178 | 12.09% | 206 | 111 | 46.12% |

Separate regression fixture: 1000-witness test 3,866,718 -> 2,310,101 Steps; 40,762 -> 22,444 Calls. Quad-native end-to-end probe 1,513,471 -> 971,487 Steps; 22,276 -> 13,108 Calls; golden SHA-256 unchanged.

## Verified qualification and known limitations

- Pinned frozen C0 compiles `compiler/main.sm` to SemCode; verifier accepted **92 functions** (`SEMCOD23`, epoch 0.24).
- Quad-native suite **27/27 PASS**, PR46 follow-up **3/3 PASS**, preserving Q3.9 tampered-Sema and Q4.9 provenance tests.
- Full pinned-C0 identity-checked differential **125/125 PASS** and **9/9 mutation checks PASS**, exit **0**, on the final single-pass driver candidate. Evidence: `%TEMP%\pr46-driver-n24-125cases\results.json`.
- Corpus regeneration reproducibility **1/1 PASS**; imports **5/5 PASS**; reference outcome tests **6/6 PASS**; `git diff --check` and local Harness **PASS**. Bootstrap Contract v2 validator **PASS** before the driver-only change (which touches no bootstrap paths).

**Normative self-compile remains BLOCKED.** Running `smc run main.sm --envelope trusted-compiler --max-steps 20000000 --max-calls 50000` on the complete current compiler source stopped at 20,000,001 Steps, with exact counters: **20,000,000 Steps**, **1,992 Calls**, 30 peak frames, 19 effect calls, `quota_exceeded: Steps`. Neither quotas nor contract were raised. An isolated, temporary stage-instrumented source copy (outside the repo) reached EXPANDED, DRIVER_ENTER, SOURCE_VERIFIED, LEXED, then exceeded the 20M-Step ceiling *before PARSED*. The canonical failure is therefore inside `parse_program` of the full compiler source, before `analyze`, any `always_returns`-driven lowering, or IR encoding. Instrumentation is diagnostic only and is not part of the product patch.

Three `test_ownership_semcode.py::test_invalid_ownership_anchors_fail_before_artifact` subtests (missing/wrong_store/wrong_record) exceeded the 50,000-Call limit when the broad unittest suite was run *before this driver edit*. These are in an independent SemCode fixture path and must not be silently classed PASS.

The existing `evidence_has_duplicate` is already O(n) via `Map(u32,bool)`; residual duplicate validation and repeated `always_returns` tree evaluation are separate optimization targets, not proven to be the normative self-compile blocker at the measured 20M boundary.

Historical `docs/architecture/QUAD_NATIVE_*` checkpoint reports are kept intact: deleting history is not part of a performance fix.

## Test identity and change control

Baseline candidate HEAD (prior to these worktree edits): `9a9e818c433ce46b29201afb668ed56235fb2d5f`. The patch is one authorized `compiler/driver.sm` change; the n24 test contract correction is a separate logical task with its own documentation. Both should retain distinct commit identities if delivered together. No bootstrap promotion, self-hosted seal, or independent compiler comparison is claimed.
