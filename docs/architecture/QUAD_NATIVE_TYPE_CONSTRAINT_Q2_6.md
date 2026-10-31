# Q2.6 — Map<K,V> Independent Dependent-Type Evidence

**Status:** local experimental Q2 extension. Independent proof families, not a complete dependent-type system. Date: 2026-10-31.

## Assertion identities

Every proposition carries `(proposition_kind, call_ast_node_id, scope=0)`. Different kinds **must not** be JOINed together.

| Map API | Container/context | Key | Value/default |
|---|---|---|---|
| `map_empty()` | `map-context-type`: expected contextual type has `TyInfo.k=9`; provenance = call AST position | not applicable | not applicable |
| `map_contains(map, key)` | `map-container-type`: first argument is a Map; provenance = map argument AST position | `map-key-type`: actual key type matches declared `Map<K,V>.K`; provenance = key expression position | not applicable |
| `map_get(map, key, default)` | `map-container-type` | `map-key-type` | `map-value-type`: default type matches `Map<K,V>.V`; provenance = default expression position |
| `map_set(map, key, value)` | `map-container-type` | `map-key-type` | `map-value-type`: assigned value matches `Map<K,V>.V` |

The carrier maps are `Sem.map_container_evidence`, `Sem.map_key_evidence`, and `Sem.map_value_evidence`. These are distinct from `Sem.type_evidence`: no unrelated container/key/value facts can accidentally merge into `S`. Single admissible observation produces `T`; an incompatible one produces `F`; `map_empty` with no contextual expected type retains `N` in `map-context-type` and the existing diagnostic. No fabricated witness of zero-argument actual value is created. A rejected `map_set(..., wrong_value)` preserves valid container and key proof alongside a separate negative value proof.

## Safeguard against missing dispatch metadata

Q3 independently recognizes map builtins from the reserved AST names `map_empty`, `map_contains`, `map_get`, `map_set`, then **requires** the corresponding `Sem.callees` mapping to equal the expected builtin ID. Missing/wrong callee identity is fail-closed, even if all evidence was otherwise valid. User-defined functions with those names are already disallowed by Sema.

## Q3 admission

`q3_map_proof_error` uses `q3_map_slot_error` to validate:
- exact slot presence, `evidence_valid`, kind/subject/scope;
- `state == T`, no negative family, exactly one supporting witness;
- exact AST source-position correspondence to the relevant map argument, key, default/value or call node for `map_empty`;
- correct number of required independent obligations for the builtin, from its AST name.

`q3_lowering_error` invokes this checker for every AST node, independent from ordinary scalar `q3_required_kind`. Any missing/invalid container, key or value proof rejects lowering. The existing type diagnostics are preserved for rejected source programs.

## Q4 provenance

`q4_map_phase_proofs` creates one `PhaseProof` per admitted map obligation in deterministic container → key → value order, each retaining its exact original `Evidence`, `source_nodes`, `source_offsets`. `lower_expr` transports these snapshots into non-emitted `FnIr.proofs`. No SemCode format or verifier contract change.

The `map_empty` context witness uses the **call location** as an observation marker: it does **not** separately prove the provenance of the expected type expression (e.g. containing variable annotation). That complete derivation requires source-qualified parentage in a future Q4.

## Tests

- Real Sema→Lower test: `tests/compiler/programs/quad_map_dependency_internal.sm` checks four map APIs; positive, missing-context N, incompatible-container F, incompatible-key F, incompatible-value F; exact witnesses and provenance; IR transport; deterministic replay.
- The test removes only key, only value or dispatch metadata; corrupts a value witness position; substitutes valid key Evidence into the value slot; checks an attempted cross-proposition JOIN is rejected.
- `tests/compiler/test_quad_native.py` checks all twelve existing map corpus files `m01..m12` against frozen C0, verifying and executing accepted native and C0 artifacts.
- Full qualification details recorded separately in `QUAD_NATIVE_Q2_6_CHECKPOINT_20261031.md`.

## Limits

The raw `Evidence` record and metadata maps remain constructible/mutable Semantic values. This boundary checks deterministic structural consistency against the same in-memory AST, **not** a cryptographic or unforgeable type proof, nor a trusted proof derivation from file-qualified AST spans. No nested/generic dependent constraint solver is claimed. Unchecked source origins, optimized IR lineage, verifier-visible serialization, full Q2/Q3/Q4, normative C1/C2 fixed point and Bootstrap Seal remain future work.
