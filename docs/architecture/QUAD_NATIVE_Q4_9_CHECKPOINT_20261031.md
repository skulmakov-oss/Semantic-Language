# Q4.9 — File-Qualified Source→AST→Evidence→IR Provenance Checkpoint

**Date:** 2026-10-31. **Branch:** `feat/quad-native-shf15`. **Base HEAD:** `972066b25d2201344747becff0a0d6abfeb0bf4d`. **State:** local experimental worktree, not committed, pushed or published.

## Changes

- `compiler/main.sm`: `Load` tracks complete source snapshots and a deterministic flattened-byte-to-file byte mapping, including recursive imports and loader-generated delimiters. Existing import expansion and SemCode behavior are preserved.
- `compiler/driver.sm`: `SourceFile`, `SourceSpan`, `SourceOrigin`, `ProvenanceStep` and `ProvenanceResult`, full-byte source-bundle validation with physical file re-read, complete AST reparse/equality, proof lineage checks, and a new `compile_source_with_origins` entry used by the loader. `compile_source` continues to support the in-memory identity.
- Before emitting SemCode, the driver checks Q3.9 canonical Sema admission and validates the chain `file snapshot -> expanded text -> Node.pos -> Sema Evidence/derivation -> FnIr`. Rejected mappings cannot mint a valid internal trace.
- No protected verifier, VM, Harness script or Bootstrap Contract edit. No new normative SemCode header or serialized proof section.

## Test corpus

- New `quad_source_provenance_internal.sm` executes real C0-compiled Rust-like Semantic and verifies positive file-qualified IR links, original byte offsets, deterministic replay, and fail-closed behavior for swapped/duplicate files, missing/overlapping origins, wrong offsets, AST tampering and altered IR source proof.
- New Python tests cover the real internal fixture and direct/nested/comment-boundary physical imports; accepted artifacts are verified and run by pinned frozen C0.
- Existing direct-import regression tests additionally exercise CRLF/LF/EOF, duplicate/cycle paths, bad imports and large files.
- Full unit/import and compiler differential + mutation qualification recorded below upon completion.

## Exact experimental identities (SHA-256)

- `compiler/main.sm`: `181c53aeef3483a1ade29ad8686e2ee64f7edd3e55f20cfb00768c7da305bee9`
- `compiler/driver.sm`: `329158de0ab5d9f24994b71d279ee906e658626947fe8a439ba10f708a5ed299`
- `tests/compiler/test_quad_native.py`: `dd5583ad45fd50e952f6980aa251b5e540b5380257f0380af371060f9a6fa048`
- `tests/compiler/programs/quad_source_provenance_internal.sm`: `e610d67f84073ea869b9d88b968a449e3ad4d3a9274514c172aede5aced6ced8`
- Experimental source bundle: `1345841c0dc5e90d6ba2549a2c41b235e5eb76963b6f9f982762c12fcfe21bce` (**not** the canonical source-set identity).
- Golden quad differential artifact expected to remain `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`.

## Explicit limits

Source identity is complete `(loader path, exact source bytes)`, not a cryptographic digest, filesystem-signature authority or path canonicalization contract. File snapshots are physically re-read but there is a TOCTOU window after admission. Public Semantic record construction still permits forging `Evidence` and `ProvenanceStep`; the defended boundary is the validated `compile_source_with_origins` path. Internal `FnIr` traces do not authenticate optimizer transforms, ABI encoding or execution; they are not emitted into SemCode or checked by frozen Verifier. The proposed genuinely closed, verifier-enforced ADT is **design only** in `EVIDENCE_ADT_DESIGN_Q3_10.md`.

C1/C2 fixed-point proof and Bootstrap Seal are **NOT RUN**. The known `n24` legacy/quad mismatch remains separate owner governance, never silently edited.

**No commit, push, PR, merge or change to frozen C0/VM/Verifier/Harness/Bootstrap Contract.**

## Final qualification — verified on the completed Q4.9 candidate

| Gate | Result |
|---|---|
| Frozen C0 compilation | **PASS** |
| Frozen C0 Verifier | **PASS** — 88 functions, SEMCOD23 / epoch 0.24 |
| New Q4.9 real-internal and imported-file tests | **2/2 PASS** |
| Complete quad-native/import unit tests | **30/30 PASS** |
| Full differential suite | **123/124 PASS**, only pre-existing `n24_unsupported_type_annotation` |
| Mutated SemCode cases | **9/9 PASS** |
| Canonical C1/C2 fixed point | **NOT RUN** |
| Bootstrap Seal | **NOT CLAIMED** |

Full differential artifact: `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-q49\differential\results.json`. The differential runner returned **RC=1** because the known `n24` legacy expectation conflicts with quad-native acceptance. **This must not be described as complete green qualification.**

Golden quad differential artifact SHA-256 remained unchanged at `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`. Q4.9 golden VM metrics: **1,578,534 Steps, 27,346 Calls, 13 peak Frames, 4 Effect Calls**. Q3.9 reference: **1,220,136 Steps, 23,554 Calls, 13 peak Frames, 3 Effect Calls**. Additional overhead on this workload: approximately **+29.4% Steps, +16.1% Calls** plus one file re-read effect. These are controlled named-workload observations, **not** a general compiler throughput benchmark.

The physical source-byte re-read is a best-effort file-snapshot consistency check within the constrained loader, not atomic authenticated filesystem provenance. Internal proof traces remain unsealed. Q3.10 remains a **design document only**, requiring explicit future owner authorization for opaque compiler/verifier construction and proof-transport enforcement.

**Disposition:** Q4.9 is qualified as a bounded, local, non-emitted source-to-IR provenance vertical slice. No commit, push, PR, merge or protected contract modification.

### Final post-documentation enforcement

- `git diff --check`: PASS.
- `scripts/harness-check.ps1`: PASS, **48 changed paths entirely within authorized scope**.
- Worktree branch: `feat/quad-native-shf15`; HEAD unchanged: `972066b25d2201344747becff0a0d6abfeb0bf4d`.
- Work remains uncommitted and unpublished.
