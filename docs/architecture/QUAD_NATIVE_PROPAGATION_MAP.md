# Quad-Native Propagation Map — Q1–Q4

Status: experimental / checkpoint-scoped. `IMPLEMENTED` means the actual experimental candidate contains a tested path, not that a canonical bootstrap gate has passed.

| Target | Proposition | Witnesses | Consumer | Status |
|---|---|---|---|---|
| Return-path obligation | Execution either returns or can fall through | Root `Node.pos` derived from bare quad state (not independent origins) | `always_returns` | Q1 CHECKED, provenance incomplete |
| Value-producing `if` branch types | Each branch type equals the inferred type of the first branch | `then` and `else` AST block `Node.pos`, positive/negative independently | `check_expr`, persistent `Sem.type_evidence` | **Q2 VERTICAL SLICE IMPLEMENTED** |
| General type constraints | An expression satisfies an externally required type | Required/actual constraint sources (to be formalized) | `check_expr` and caller | PROPOSED Q2+ |
| Symbol resolution | Name resolves uniquely to candidate binding | Binding candidates and ambiguity evidence | Lookup and diagnostics | PROPOSED |
| Dependencies | Dependency obligations can be discharged | Resolved/missing/cyclic edges | Import graph | PROPOSED |
| Lowering preconditions | IR operation has required facts | Valid and invalid IR preconditions | `lower_expr` / `lower_stmt` | PROPOSED Q3 |
| Proof metadata | Decision has reproducible source evidence | Source spans and proof-chain identities | Verifier-facing structures | PROPOSED Q4 |

## Transition and integrity rules

1. A carrier's identity is `(proposition_kind, subject, scope)`; never join unrelated identities.
2. Positive and negative witnesses must describe the **same** proposition; contradiction is not a generic compiler error.
3. `evidence_try_join` must be checked. `N` and `S` are not implicitly accepted by downstream executable decisions.
4. The order of witnesses is deterministic and must remain observable.
5. Full derivation provenance is **not** yet established merely because a source offset appears in a carrier.
6. Keep genuinely binary structural facts as `bool`, while expanding genuine four-state semantic computations.

## Next checkpoints

- Q2 extension: widen type constraints only after determining a proposition-specific inference/compatibility policy with real independent witnesses.
- Q3: propagate validated evidence into lowering preconditions without modifying frozen SemCode.
- Q4: replace opaque `u32` offsets with stable source-file/node/span + transformation provenance and verify semantic origin, not merely record shape.

Q1-SAFETY is an experimental semantic-boundary checkpoint. Canonical subset admission, normative budget, source-set identity and C1/C2 fixed point remain separate gates.

## 2026-10-31 implementation update

The table above reflects the initial Q2 vertical slice. The experimental worktree now additionally contains these checked paths:

- **Q2-EXT:** user-call argument compatibility and record literal/copy-with field compatibility via `accumulate_type_constraint` in `sema.sm`. Both accumulate distinct positional witnesses under the current AST owner's proposition identity, preserving valid `T` and rejected `F/S` states.
- **Q3-SLICE:** `q3_lowering_error` in `lower.sm` is called by `compile_source` before any lowering. `N/F/S`, missing carriers, wrong identity, duplicate/mismatched witnesses and inconsistent source offsets are rejected.
- **Q4-SLICE:** `q4_phase_proof` copies admitted `Evidence`, AST source node IDs and byte offsets into non-emitted `FnIr.proofs`; the serialized SemCode and frozen verifier remain unchanged.

**These do not yet constitute complete type-system coverage, universal lowering admission, or file-aware, verifier-visible cross-phase provenance.** For exact supported nodes and negative tests see `QUAD_NATIVE_TYPE_CONSTRAINT_Q2.md`, `QUAD_NATIVE_LOWERING_PRECONDITIONS_Q3.md`, and `QUAD_NATIVE_CROSS_PHASE_PROVENANCE_Q4.md`.
