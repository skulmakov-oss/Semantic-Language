# Q3 — Checked Lowering Preconditions (Vertical Slice)

**Status: implemented experimentally for three Q2 proof families, not a complete lowering proof system.**

## Intent and authority

Lowering must not infer authorization from a syntactically well-formed AST alone. The Sema result must be error-free, and every selected AST expression must carry a checked, admitted type proposition.

`compile_source` in `compiler/driver.sm` invokes `q3_lowering_error(sem)` after `analyze` and before any `lower_function`. A failure returns `error unsupported @0: lowering precondition: ...` and emits **no artifact**. The source's own type error remains the authoritative diagnostic when Sema has already rejected the program.

## Checked domains

| AST kind | Required identity | Expected witnesses |
|---|---|---|
| 26 — value-producing `if` | `("if-branch-type", node_id, 0)` | `then` block and `else` block source positions, in order |
| 7 — named user-function call with one or more arguments | `("call-arguments", node_id, 0)` | argument AST positions, in order |
| 29 / 21 — record literal / copy-with with fields | `("record-fields", node_id, 0)` | field AST positions, in order |

Calls to builtins, zero-argument calls, record constructors with zero fields and unrelated expressions are **outside this proof boundary**, not silently deemed proved.

## Policy

For each checked expression, `q3_proof_error` requires:
1. Evidence exists at the exact owner node ID in `Sem.type_evidence`.
2. `evidence_valid` holds, with exact `kind/subject/scope` identity.
3. The state is **T**, no negative witnesses remain.
4. Positive witness cardinality matches the AST contributors exactly.
5. Every positive witness ID equals the corresponding source `Node.pos`.

Missing, contradictory, inconsistent, duplicated or source-mismatched evidence is rejected before emission. `N`, `F`, and `S` cannot be interpreted as successful lowering authorization.

The proof only certifies the three declared type-compatibility obligations. It does not prove all semantic invariants, runtime safety, ownership, effect restrictions or verifier correctness.

## Tests

`tests/compiler/programs/quad_lowering_provenance_internal.sm` tests actual Sema→Lower on a combined program and deliberately tampers with the evidence map (missing proof, source mismatch, conflict, different subject), verifying every case is rejected. The fixture inspects IR provenance carried in `FnIr.proofs`. `tests/compiler/test_quad_native.py` drives frozen C0 compilation, verification and execution of this fixture.

## Constraints

No change to the frozen compiler, VM, Verifier, canonical source-set contract or SemCode format. Q3 is a first fail-closed boundary; fuller coverage must be specified by AST operator family before declaring Q3 globally complete.

## Q3.2 — Statement obligations

`q3_required_kind` and `q3_source_nodes` now include:

| Owner kind | Required proof | Contributors |
|---|---|---|
| `k=10` with explicit annotation | `annotated-binding` | initializer `nd.a` |
| `k=11` | `assignment-type` | assignment RHS `nd.a` |
| `k=14` with a returned value | `return-value-type` | return value expression `nd.a` |

There is intentionally no proof requirement for type-inferred declarations or unit `return;` statements with no value. Every admitted proof must remain valid and in state **T** with one witness matching the exact AST source node position. Any `F` witnessed comparison yields the original Sema error before lowering; a deliberately corrupted valid `Evidence` source-offset or missing proof is rejected by `q3_lowering_error` before artifact generation.

**Boundary of this check:** a matching source position authenticates agreement with the current in-memory AST, **not** the semantic truth of arbitrary forged positive evidence. A future authoritative proof derivation/ADT boundary is required for that stronger guarantee.

## Q3.3 — Container and match result obligations

Q3 now examines the following additional expression owners:
- `k=22` **only when** `Sem.tuple_expected` declares a contextual tuple target: `tuple-target-items`;
- `k=23` **when nonempty**: `sequence-elements`;
- `k=25` with an existing, Sema-produced result-arm evidence carrier: `match-arm-types`.

`q3_source_nodes` maps tuple/sequence source IDs to their element expressions. For match, it maps each arm node to `arm.a`, its actual result expression. Q3 rejects missing/invalid evidence at required boundaries, non-T states, count mismatches, incorrect AST offsets or mismatched proposition identity. The exact expression owner must be protected against forged proof origins by a future authoritative carrier, because contextual tuple and match registration currently depends on mutable Sema metadata.

No policy changes are made for empty sequences or inferred tuples without an independently imposed compatibility comparison.

## Q3.4 — Operator and non-map builtin type obligations

- For unary operators (`k=5`), Q3 requires one `unary-operator-type` witness, exactly at `nd.a`.
- For binary operators (`k=6`), Q3 requires two `binary-operator-types` witnesses, at `nd.a` and `nd.b` in source order. The proposition checks type **admissibility for the chosen operator**, not merely equality.
- For non-map builtins (`k=7`, builtin callee IDs for 1..7 and 12..14), Q3 requires `builtin-argument-types` with one witness per argument. The builtin identity is taken from `Sem.callees`, not from the existence of Evidence.
- Fail-closed identity, witness count, state T and exact source-position policies are unchanged. F/S evidence from a rejected Sema program never authorizes artifact emission.
- Excluded: map-family builtins, whose dependent key/value obligations remain a separate Q2 design checkpoint.

## Q3.5 — Independent Map<K,V> admission

Q3 now invokes `q3_map_proof_error` on every source AST node as part of `q3_lowering_error`. The reserved map builtin names in the source AST independently establish a lowering obligation; Sema's `callees` metadata **must** agree with the expected builtin ID and cannot be erased to bypass the requirement. Map `container/context`, `key`, and `value/default` evidence are stored in **separate carrier maps** and validated separately. All required slots must be valid, T-only, and source-position matched; absent, N/F/S or misidentified slots are rejected. No cross-proposition JOIN or fabricated zero-argument value witness is allowed.

`quad_map_dependency_internal.sm` proves failure on missing key, value, callee metadata, forged origin, and wrong-kind substitution while preserving unrelated valid proofs.

## Q3.6 — Tuple shape, annotation, target assignment and inferred origin

The Sema→Lower gate now checks tuple destructuring statements (`k=40` and `k=41`) independently from ordinary assignment/binding carrier rules. A valid `tuple-destructure-shape` `T` witness from the source RHS is mandatory for both. For `k=40` with an explicit tuple annotation, `tuple-annotation-type` is additionally mandatory. For a nonempty `k=41`, `tuple-assignment-items` covers each destination `Name` in stable source order. `k=40` has **no** fabricated positive per-binding type-comparison proposition; instead it must possess component inference origin metadata consistent with the RHS `TyInfo.kids` and known destination slots. Missing, incompatible or position-mismatched evidence fails preflight. This remains consistency checking inside mutable Sema data, not an unforgeable proof authority.

## Q3.7/Q3.8 — Control type admission and stronger typed-AST consistency

The required `type_evidence` owners now include `if` statement `k=12`, `while` `k=13`, `assert` `k=17` and `for-in` `k=42`. Their exact one-witness propositions `if-condition-type`, `while-condition-type`, `assert-condition-type` and `for-sequence-type` are fail-closed before lowering. An unannotated `let` instead must have an independent, type-correct `Sem.inferred_binding_types` derivation record: this **is not a fabricated positive proof**.

`q3_control_type_error` independently reconstructs the source expression family from the current typed AST, requiring `bool` or `Sequence` as appropriate. For `k=26` if-expressions, Sema also records a separate `if-expression-condition-type` proof alongside existing branch compatibility, and missing/forged conditions fail even when the branch result proof remains T. Correct offsets alone cannot authorize an inconsistent observed type family. This remains internal evidence consistency, not unforgeable Sema authority.

## Q3.9 — Canonical reanalysis before trusting any public `Sem` carrier

The production driver now calls `q3_trusted_admit(sem)` and lowers **only** `admission.canonical`. This function runs a fresh trusted `analyze(sem.prog)`, compares AST-indexed Evidence families and typed metadata against the candidate, rejects any mismatch or new Sema error, then calls normal `q3_lowering_error` on the canonical replay state. `Evidence` and `Sem` are still constructible: Q3.9 provides a local **trusted computation and canonical transfer** boundary for `compile_source`, not an unforgeable ADT. The analysis is repeated once at driver admission, not on every IR expression. For details and limitations see `QUAD_NATIVE_TRUSTED_EVIDENCE_Q3_9.md`.
