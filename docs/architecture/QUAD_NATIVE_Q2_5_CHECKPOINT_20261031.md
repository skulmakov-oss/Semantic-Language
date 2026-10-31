# Q2.5 / Q3.4 / Q4.4 — Operator and Non-map Builtin Evidence Checkpoint

**Date:** 2026-10-31. **Branch:** `feat/quad-native-shf15`. **Base HEAD:** `972066b25d2201344747becff0a0d6abfeb0bf4d`. **State:** experimental local worktree, uncommitted and unpublished.

## Implementation

- `compiler/sema.sm` now materializes `Evidence` for unary operator admissibility, binary operator-family admissibility, and non-map builtin argument admissibility.
- Evidence is proposition-scoped, retains ordered AST positions and both polarities, uses validated `EvidenceResult` JOIN for multi-operand operators and builtins. Invalid programs retain original diagnostic category/message and are not admitted.
- Operator validation is stronger than operand-type equality: `bool + bool` gives F, not T; `i32 + bool` gives S (positive+negative), while permitted types give T.
- Non-map builtin coverage is restricted to names `len`, `push`, `pop`, `to_text`, `text_len`, `text_byte_at`, `text_slice`, `args_read`, `fs_read_text`, `stdout_write`. `map_empty/map_contains/map_get/map_set` are **explicitly excluded**, pending dependent-type design.
- `compiler/lower.sm` now demands the appropriate T-only evidence for selected unary, binary and builtin nodes, with exact owner identity, count and source-position matching. The Q3 builtin obligation is recovered from the classified callee ID, rather than from carrier presence.
- The existing Q4 `PhaseProof` snapshots are populated for new expression families in non-emitted `FnIr.proofs`. Frozen SemCode format and verifier are untouched.

## Targeted tests

- Real compiler source fixture: `tests/compiler/programs/quad_builtin_operator_internal.sm`, including real Sema and lowered IR, exact source positions and invalid/missing evidence rejection.
- `tests/compiler/test_quad_native.py`: direct pinned C0-vs-native comparison for 14 independently compiled valid/invalid programs. Positive cases are verified and executed on pinned C0.
- Earlier Q4 tests have been expanded to recognize additional binary-operator evidence and independently check all contributor positions.
- At initial checkpoint the 2 newly targeted tests and 2 updated previous Q4 tests both passed separately. Full current-run unit/import and compiler differential results are appended after completion.

## Safety and authority limits

A source-level `Evidence` record remains forgeable. Exact AST-position congruence protects against missing and mismatched witnesses but cannot cryptographically authenticate a type claim with a perfectly forged positive carrier. File-aware provenance, authoritative derivation identities, SemCode proof transport, verifier-visible proof checks, frozen Bootstrap authority and C1/C2 fixed-point proof remain future work.

**No commit, push, PR, merge, frozen compiler modification, VM/Verifier/Harness edit or Bootstrap Contract change.**

## Final qualification — same candidate source

- Pinned C0 compiler compilation: **PASS**; output `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-20261031-q25\candidate.smc`, verified **58 functions**, `SEMCOD23`, epoch `0.24`.
- Final quad-native and import unit suite: `python -B -m unittest test_quad_native test_imports -v`, **20/20 PASS** after updating the two existing Q4 fixtures to include newly created operator proofs.
- Full compiler differential: **123/124 PASS**; **9/9 SemCode mutation checks PASS**. The process exited nonzero because of previously known legacy `n24_unsupported_type_annotation`. **This is not a full green bootstrap qualification.**
- Differential evidence: `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-20261031-q25\differential\results.json`.
- Full-tree Harness check: **PASS**, 32 changed paths in permitted scope. `git diff --check`: **PASS**.
- Golden quad differential artifact SHA-256 is unchanged: `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`. Current test VM metrics: **636,980 Steps / 7,609 Calls / 12 Peak Frames / 3 Effect Calls**. This is increased versus Q2.4's 550,767 Steps / 5,428 Calls; this is a per-scenario observation, **not** a controlled compiler performance benchmark.
- Experimental nine-module source bundle identity `8144d6fc5005d5ca286f63e107dcf5705affaf14fd25c4d8a49ffdde4b5614c3` (local content framing; **not** canonical bootstrap source-set identity).
- `compiler/sema.sm` SHA-256: `a0c2e2f98abbe55b7696340785dced75b4e29b75aeaa403a224827ca15020930`.
- `compiler/lower.sm` SHA-256: `43e3885e3b843abe73df19914556d89f63a0823b1f7519d247b65845d7071d71`.
- `tests/compiler/test_quad_native.py` SHA-256: `048c1ce1c96abd7272aca0869bd3e9091b2ee10116339840935ae868917878ec`.
- `tests/compiler/programs/quad_builtin_operator_internal.sm` SHA-256: `1aed2963434db659a2f11d0163850aa4a7c5b3cf0c1c93d5b5ec806d363b38e7`.
- Frozen C0 binary SHA-256 remains `a612fc710c0961103aff6f0d078f442214815d8c6a49db2ea163c6bddf3b758e`.

**Disposition:** Q2.5, Q3.4, Q4.4 accepted as an experimental vertical slice only. General type-system coverage, ownership/effect lowering proofs, file-qualified provenance, map-family dependencies, normative C1/C2 fixed point, and Bootstrap Seal remain **open**. No commit, push, PR, merge or protected infrastructure change.
