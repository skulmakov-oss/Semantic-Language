# Q4 — Sema→Lower Evidence Provenance (Vertical Slice)

**Status: one real cross-phase transport, not verifier-visible proof provenance.**

## Data path

```text
AST branch/argument/field node ID and Node.pos
       |
       v
Sema: Evidence(proposition_kind, subject, scope,
               state, positive[] positions, negative[] positions)
       |
       | q3_lowering_error: identity, validity, T-only,
       |                     contributor count and exact Node.pos checks
       v
Lower: PhaseProof {
         evidence: Evidence,
         source_nodes: Sequence(i32),
         source_offsets: Sequence(u32)
       }
       |
       v
FnIr.proofs: Sequence(PhaseProof)  [internal metadata, non-emitted]
       |
       +--> SemCode serializer intentionally does NOT encode this metadata
```

For every checked `lower_expr` owner, `q4_phase_proof` snapshots validated `Evidence` plus corresponding source node IDs/offsets into the active function's IR provenance list. No negative `S/F` evidence crosses the emission gate, because Q3 rejects it.

The order of the proofs follows lowering traversal; each proof's contributing witness order follows Sema source traversal. A repeated analysis of the same source must produce identical proof identities, source-node IDs and positions.

## Trust limits

- `source_offsets` are local to the expanded compilation input. They are **not** a fully qualified `(file, span, expansion provenance)` identity.
- This proves structural consistency between `Sem.type_evidence` and its current AST snapshot. It does not cryptographically authenticate source origins.
- The proof snapshot is carried in `FnIr` in memory, not serialized into SemCode. Thus the frozen C0 verifier cannot check the proof and no verifier-facing contract is claimed.
- Only selected type obligations have provenance. Return-path provenance still attaches a root `Node.pos` after computing a bare quad; independent return branch derivations have not been preserved.
- This is an experimental internal data structure, **not** an allowed normative change to the Bootstrap Contract.

## Qualification

`quad_lowering_provenance_internal.sm` verifies three proof families in real lowered IR, AST positions, fail-closed tampering, and repeatable proof order. Negative evidence is retained at rejected Sema decisions but is never treated as lowering authorization.

## Next Q4 tasks

Define file-aware source identities, transformation origin/parentage, an explicit versioned proof transport contract, stable serialization, and verifier-facing checks under a separately authorized bootstrap-contract checkpoint.

## Q4.2 — Statement provenance

`lower_stmt` now snapshots Q3-admitted statements through `q4_phase_proof` to `FnIr.proofs`, in addition to the existing `lower_expr` snapshot path. A proof for an annotated binding, assignment or returned value contains the **one** value-expression AST node ID and its source-local byte position, plus the full validated carrier. Order follows deterministic lowering traversal; the non-emitted proof metadata does not modify SemCode bytes or the frozen verifier contract.

`quad_binding_return_internal.sm` explicitly checks `FnIr.proofs` for an annotated declaration and assignment in `main`, plus an explicit return in a separate function, and rejects tampered source metadata. Source-file IDs, actual source spans, macro/import expansion genealogy, transformation ancestry and verifier-visible provenance remain outside this checkpoint.

## Q4.3 — Container and match result provenance

The existing `lower_expr` proof-capture path now also stores `tuple-target-items`, `sequence-elements`, and `match-arm-types` in `FnIr.proofs`. Each `PhaseProof` retains the checked Evidence state/witness families, AST contributor IDs and their source-local offsets. The match proof traces the arm's result expression rather than the arm's pattern position. The source traversal order is retained; the internal data are deliberately not emitted into frozen SemCode.

`quad_container_match_internal.sm` validates these three proof kinds in real lowered IR, checks a structurally valid but source-mismatched counterfeit witness and verifies missing-evidence rejection. This is **Sema→Lower internal provenance**; verifier-visible file-qualified proof chains remain a separately governed future task.

## Q4.4 — Operator and builtin provenance

`lower_expr` now carries selected unary, binary and non-map builtin evidence through the same validated `q4_phase_proof` into `FnIr.proofs`. It preserves `Node.pos` per contributing source operand/argument, owner identity, witness polarity and deterministic traversal order. This is non-emitted internal IR provenance; the SemCode verifier receives no additional proof metadata. Full source file/derivation lineage and authority-bearing serialization are still future work.

## Q4.5 — Map<K,V> proof transport

The Sema→Lower handoff now includes `q4_map_phase_proofs`: independently validated context/container, key and value/default Evidence snapshots are stored in deterministic order in `FnIr.proofs`. Each `PhaseProof` carries exactly one source AST ID and its source-local offset; for context-only `map_empty`, that origin is the call node rather than an invented argument. No proof bytes are emitted and the pinned C0 verifier does not consume this metadata. There is still no trustworthy origin chain from the map's `K,V` type declarations to the compatibility checks: this remains Q4 future scope.

## Q4.6 — Separate inferred-type derivation from checked type witnesses

For tuple destructuring, `FnIr.proofs` carries validated shape evidence and, where present, tuple annotation or assignment compatibility. A distinct `FnIr.tuple_inferences` array carries `TupleInferenceLink(owner, source, binding, component, resolved_type, source_offset, binding_offset)` for new inferred tuple bindings only. These links are explicitly **not** positive Evidence and cannot JOIN into `T` or `S` type constraints. This metadata stays in-memory and is not emitted into SemCode. It does not prove file-qualified origin, signed evidence authority, or transformation ancestry.

## Q4.7/Q4.8 — Inference links and checked control type origin

`FnIr.binding_inferences` stores `BindingInferenceLink` with owner/source type derivation and source-local byte locations for unannotated `let`, separated from positive/negative Evidence. `FnIr.control_type_checks` stores `ControlTypeCheck`: owner/source AST identity and kind, independently observed type and family, required family, exact source offset and carrier. `FnIr.proofs` still carries T-only condition/iterable proofs for statements; if-expression conditions have a distinct carrier to avoid JOIN with branch type agreement. All remain non-emitted, not a verifier-readable SemCode proof chain. Source-file IDs and unforgeable derivation lineage remain future work.

## Q4.9 precondition — canonical source for the existing internal proof snapshots

The driver now forwards only a Sema-replayed canonical state after Q3.9 admission into `lower_function`. Existing `FnIr.proofs`, `binding_inferences`, `tuple_inferences` and `control_type_checks` therefore originate from the admitted replay, not from caller-mutated `Sem` fields. They still contain source-local positions rather than file-qualified identity and remain non-emitted. This is a provenance boundary refinement, **not** completion of Q4.9 authenticated source lineage.

## Q4.9 — File-qualified experimental source and proof lineage

The loader now produces exact per-file snapshots and flat-to-file byte spans. The driver rechecks file snapshots, re-parses the flattened text, invokes Q3.9 canonical Sema replay, and rejects mismatched file/AST/Evidence/IR provenance via `q4_trace_function`. Internal `ProvenanceStep` records identify the concrete file path and byte offset behind every recorded proof/inference/control source. No provenance is emitted into SemCode; no file hash, signature, optimization ancestry, or verifier-readable proof is claimed. See `QUAD_NATIVE_FILE_PROVENANCE_Q4_9.md`.
