# Quad-native proof and trust boundaries

Status: implementation contract for the Semantic-written compiler in `compiler/`, as it exists on
`main`. It records which checks run, where, and what a passed check does and does not establish.
It is not a language specification, not a Bootstrap Contract document and not a stage-promotion
record.

The evidence carrier and the propositions are defined in
[QUAD_NATIVE_EVIDENCE_CONTRACT.md](QUAD_NATIVE_EVIDENCE_CONTRACT.md). Historical sources are cited
as `PR46:<file>` and refer to commit `bd2206fae01d3a3052eee3359c23c28551a93e0f`; they are not part of
`main`. Where this text and the code disagree, the code and its tests are right.

Test anchors: `TQN` is `tests/compiler/test_quad_native.py`, `TPF` is
`tests/compiler/test_pr46_followup.py`, fixtures are in `tests/compiler/programs/`.

## 1. Four words that are not synonyms

| Term | Meaning in this compiler | Provided by |
|---|---|---|
| **Validation** | a record is internally consistent: identity well formed, no duplicate witness, state derived from witnesses | `evidence_valid`, `evidence_try_join` |
| **Provenance** | a checked link from a proposition to the AST node, the byte offset and the source file it came from | `q3_proof_error`, `q4_phase_proof`, `q4_trace_function`, `q4_source_bundle_error` |
| **Authenticity** | proof that a record was produced by the trusted analysis and not written by hand | **not provided** as a property of the record; approximated only by recomputation (`q3_trusted_admit`) |
| **Authorization** | permission to lower and emit | `q3_lowering_error` returning no error, then `q4_trace_function` succeeding for every function |

A carrier that validates is not thereby authentic, and a carrier with correct provenance is not
thereby true. Authorization in the production path does not come from any carrier's contents: it
comes from the fact that the driver built the AST and the `Sem` itself, in the same run, from
bytes it checked.

## 2. Production path

`compile_source_with_origins(src, files, spans)` in `compiler/driver.sm` is the entry point used by
`compiler/main.sm`. `compile_source(src)` wraps it with one synthetic file named `<memory>`.

| Step | Check | Failure output |
|---|---|---|
| 1 | `q4_source_bundle_error(src, files, spans, true)` | `error unsupported @0: source provenance: …` |
| 2 | `lex`, `parse_program` | the lexer or parser diagnostic |
| 3 | `analyze` | the semantic diagnostic, with its own category and offset |
| 4 | `q3_lowering_error(sem)` | `error unsupported @0: lowering precondition: …` |
| 5 | for each function: `lower_function`, then `q4_trace_function` | `error unsupported @0: proof provenance: …` |
| 6 | `encode_program` | `error unsupported @0: …` |

No step has a retry, a second implementation or a substitute value. A failure at any step returns
one `error` line and no artifact.

Since PR #46's last commit (`bd2206f`), the production path is single-pass: it does **not** call
`q4_parsed_identity_error` or `q3_trusted_admit`. Both remain in the code as checks for AST and
`Sem` values supplied from outside; tests call them directly. The reasoning recorded at the time,
and repeated in a comment in `compile_source_with_origins`, is that the driver constructs both
values itself from validated bytes, so replaying them re-checks the driver's own work and no
additional input.

| | |
|---|---|
| Implementation | `compile_source_with_origins`, `compile_source` in `compiler/driver.sm` |
| Tests | TQN `test_q39_compile_source_replay_c0_admission`; the differential corpus (`tests/compiler/run_tests.py`) |
| Historical source | `PR46:docs/architecture/PR46_DRIVER_SINGLE_PASS_20261101.md` |
| Guarantee | no artifact is emitted unless steps 1–6 all succeed for the whole program |
| Not guaranteed | defence against a modified compiler. Lexer, parser, analysis, lowering and driver are all inside the trusted implementation |

An earlier PR #46 document (`PR46:docs/architecture/QUAD_NATIVE_TRUSTED_EVIDENCE_Q3_9.md`) says the
production driver lowers only the replayed canonical `Sem`. That was true before `bd2206f` and is
**historical**; it does not describe current code.

## 3. Q3: the lowering precondition

### 3.1 Producers and consumers

| Role | Code |
|---|---|
| Producer of evidence | `analyze` and `check_expr` in `compiler/sema.sm` |
| Checker | `q3_lowering_error` in `compiler/lower.sm`, which runs five checks on every AST node |
| Consumer | `lower_expr`, `lower_stmt` through `q4_phase_proof` and the other `q4_*` constructors |

### 3.2 What `q3_proof_error` requires

For each node that has a required proposition kind (`q3_required_kind`):

1. a carrier exists for that node in `Sem.type_evidence`;
2. it is valid and its identity is exactly `(kind, node id, 0)`;
3. its state is T and it has no negative witness;
4. the number of positive witnesses equals the number of contributing AST nodes
   (`q3_source_nodes`);
5. each positive witness equals the source offset of the corresponding contributor, in order.

N, F and S never authorize lowering.

The other four checks:

| Check | Requirement |
|---|---|
| `q3_map_proof_error` | a call named `map_empty`, `map_contains`, `map_get` or `map_set` must have the matching builtin id in `Sem.callees` and a T carrier with exactly one witness in each applicable map (container or context, key, value). The obligation comes from the name in the AST, so deleting metadata cannot remove it |
| `q3_tuple_proof_error` | tuple destructuring needs a T `tuple-destructure-shape` carrier and consistent inferred component metadata |
| `q3_binding_inference_error` | an unannotated `let` needs an inferred type that equals the type of its initializer, and a slot |
| `q3_control_type_error` | the condition of `if`, `while`, `assert` and of a value-producing `if` must have type `bool`, and the operand of `for … in` must be a `Sequence`, as recomputed from the typed AST; a value-producing `if` also needs a T `if-expression-condition-type` carrier |

### 3.3 Inside the boundary

The Q3 boundary is the combination of the five checks that `q3_lowering_error` runs. No single
helper enumerates it:

| Propositions | Required by |
|---|---|
| the eighteen kinds of the evidence contract, section 5.1 | `q3_required_kind`, checked by `q3_proof_error` |
| `map-container-type`, `map-context-type`, `map-key-type`, `map-value-type` | `q3_map_proof_error` |
| `tuple-destructure-shape` | `q3_tuple_proof_error` |
| `if-expression-condition-type` | `q3_control_type_error` |

`q3_required_kind` alone returns only the first row.

### 3.4 Outside the boundary

No proposition is required, and none is implied, for:

- literals, names and field access;
- calls of user functions with no arguments;
- record literals with no fields; empty sequence literals;
- tuple literals without a contextual tuple type;
- `match` without a registered result obligation;
- unannotated `let` (checked as a derivation fact, not as evidence);
- `return;` without a value;
- the Option constructor `Option::Some(…)` / `Option::None`.

"Outside" means only that the Q3 check asks nothing about these nodes. They are still type-checked
by semantic analysis. For example a wrong `Option::Some` payload is reported at the enclosing
binding, argument or return, each of which is inside the boundary.

| | |
|---|---|
| Implementation | `q3_required_kind`, `q3_source_nodes`, `q3_proof_error`, `q3_map_slot_error`, `q3_expected_map_callee`, `q3_map_proof_error`, `q3_tuple_proof_error`, `q3_binding_inference_error`, `q3_control_type_error`, `q3_lowering_error` in `compiler/lower.sm` |
| Tests | TQN `test_q3_q4_lowering_admission_and_provenance`, `test_q26_map_dependent_evidence_and_lowering`, `test_q27_tuple_shape_assignment_and_inference_origins`, `test_q28_control_conditions_inference_and_provenance`; fixtures `quad_lowering_provenance_internal.sm`, `quad_map_dependency_internal.sm`, `quad_tuple_inference_internal.sm`, `quad_control_inference_internal.sm` (missing proof, wrong subject, conflict, source mismatch, forged origin, wrong-kind substitution) |
| Historical source | `PR46:docs/architecture/QUAD_NATIVE_LOWERING_PRECONDITIONS_Q3.md` (Q3 to Q3.8) |
| Guarantee | lowering starts only if every node inside the boundary has a T carrier whose identity and witness positions agree with the current AST |
| Not guaranteed | that the type system is covered completely; that a carrier is authentic. A matching offset shows agreement with the in-memory AST, not the truth of a forged carrier. The Q3 check says nothing about ownership, effects or runtime behaviour |

## 4. Q3.9: canonical replay

`q3_trusted_admit(candidate)` treats a `Sem` as a candidate, not as a token:

1. it runs `analyze(candidate.prog)` again and rejects if the replay reports an error;
2. `q3_replay_difference` compares candidate and replay: the type table, arithmetic mode, function
   return and parameter types, local names, the six evidence maps (presence and full ordered
   contents) and eight metadata maps (`exprty`, `slots`, `callees`, `fieldix`, `tuple_expected`,
   `inferred_binding_types`, `tuple_inferred_components`, `match_result_checked`), for every AST
   node id;
3. it runs `q3_lowering_error` on the replay;
4. it returns `TrustedSemAdmission { ok, canonical, error }`, where `canonical` is the replay.

| | |
|---|---|
| Implementation | `q3_trusted_admit`, `q3_replay_difference`, `q3_same_evidence`, `q3_evidence_map_equal`, `q3_int_map_equal`, `q3_bool_map_equal`, `q3_ty_table_equal` in `compiler/lower.sm` |
| Tests | TQN `test_q39_canonical_sema_replay_fail_closed`; fixture `quad_trusted_admission_internal.sm` (forged T witness, altered inferred type, key evidence placed in the value map, forged tuple shape, cleared error with forged operator evidence) |
| Historical source | `PR46:docs/architecture/QUAD_NATIVE_TRUSTED_EVIDENCE_Q3_9.md` |
| Guarantee | a caller that uses `canonical` lowers a `Sem` computed by the trusted analysis from the supplied AST, whatever the candidate contained |
| Not guaranteed | authenticity of the AST: the replay starts from `candidate.prog`. Unforgeability: `TrustedSemAdmission` is a public record. Exhaustive equality of every `Sem` field: maps are compared over AST node ids only. Use on the production path: since `bd2206f` the driver does not call this function |

## 5. Q4: cross-phase provenance

Lowering copies the checked proposition of each node it visits into the function's IR:

| IR field of `FnIr` | Built by | Content |
|---|---|---|
| `proofs` | `q4_phase_proof`, `q4_map_phase_proofs`, `q4_tuple_shape_proof` | the carrier, contributing AST node ids, their offsets |
| `tuple_inferences` | `q4_tuple_inference_links` | inferred tuple component links |
| `binding_inferences` | `q4_binding_inference` | inferred binding type and source |
| `control_type_checks` | `q4_control_type_check` | observed and required type family of a condition |

`q4_phase_proof` asserts `q3_proof_error(s, id) == ""` before it copies a carrier.

The copy is not complete transport of everything Q3 checked. `q3_lowering_error` examines every
AST node, but lowering stops at a `return` and after a statement that always returns, so
statements after that point are never visited. For example, in
`fn main() { return; let x: i32 = 1; }` semantic analysis records an `annotated-binding` carrier
for the `let` and Q3 checks it, but no proof for it appears in `FnIr.proofs`.

`q4_trace_function(sem, fn_ir, spans)` then checks every entry: the carrier must be valid, equal to
the carrier semantic analysis holds for the same owner (`q4_matches_canonical_proof`,
`q3_same_evidence`), and each offset must resolve to a file through `q4_origin_at`. Each link
yields a `ProvenanceStep` with one of three stages: `evidence-to-ir`, `type-inference-to-ir`,
`control-type-to-ir`.

| | |
|---|---|
| Implementation | `q4_*` functions in `compiler/lower.sm`; `q4_trace_function`, `q4_matches_canonical_proof`, `q4_append_origin`, `q4_origin_at` in `compiler/driver.sm` |
| Tests | TQN `test_q3_q4_lowering_admission_and_provenance`, `test_q2_q3_q4_binding_assignment_return_evidence`, `test_q2_q3_q4_tuple_sequence_match_provenance`, `test_q49_source_file_proof_chain_tamper` |
| Historical source | `PR46:docs/architecture/QUAD_NATIVE_CROSS_PHASE_PROVENANCE_Q4.md` |
| Guarantee | every proof that is in the IR of an emitted function equals the carrier held by semantic analysis and resolves to a byte in a named source file |
| Not guaranteed | completeness: a proposition on a node that lowering does not visit is checked by Q3 but is not carried into the IR. Visibility outside the compiler: `FnIr.proofs` and the three link lists are **not serialized into SemCode**. C0's verifier cannot see or check them, and no verifier-facing proof format is claimed |

## 6. Q4.9: file and source-span identity

`main.sm` expands imports into one text and returns two sidecar lists:

- `SourceFile { path, content }`: the full text of every participating file;
- `SourceSpan { path, local_start, flat_start, size }`: one contiguous piece of a file as it appears
  in the expanded text. A span with an empty path is a separator generated by the loader.

`q4_source_bundle_error` requires:

1. file paths are non-empty and unique;
2. each physical file, when read again, still has the recorded content;
3. spans are non-empty, contiguous, in bounds and cover the whole expanded text;
4. each file span equals the same bytes of its file;
5. a generated span is exactly one LF or one CRLF.

`q4_origin_at` maps an expanded offset to a file and a local offset, and refuses an offset that
falls on a generated separator or outside every span.

| | |
|---|---|
| Implementation | `q4_source_bundle_error`, `q4_origin_at` in `compiler/driver.sm`; import expansion in `compiler/main.sm` |
| Tests | TQN `test_q49_source_file_proof_chain_tamper`, `test_q49_nested_import_source_identity_parity`; TPF `test_interspersed_imports_preserve_source_lineage`; `tests/compiler/test_imports.py`; fixture `quad_source_provenance_internal.sm` (replaced file content, duplicate file identity, gapped span, wrong local offset, missing span, separator, tampered IR witness, tampered AST position) |
| Historical source | `PR46:docs/architecture/QUAD_NATIVE_FILE_PROVENANCE_Q4_9.md`; `PR46:docs/architecture/CODEX_PR46_P2_FOLLOWUP_20261101.md` |
| Guarantee | the expanded text is exactly the concatenation of the recorded file pieces, and every witness offset carried in the IR of an emitted function maps to one file and one local offset |
| Not guaranteed | cryptographic identity: the identity is a path plus exact text, not a hash or signature. Freedom from races: the re-read can be overtaken by a later change to the file. Authentication of the path or of the file system |

`q4_parsed_identity_error(src, prog)` re-lexes and re-parses a source snapshot and compares every
AST element with a supplied `Prog`. It is available as a check for externally supplied ASTs and is
exercised by tests; the production path does not call it.

## 7. Why a passed check proves only its own scope

| A passed check | Establishes | Does not establish |
|---|---|---|
| `evidence_valid` | internal consistency of one record | that the record came from analysis, or that its claim is true |
| `evidence_try_join` | two consistent records of one proposition were merged without loss | that either input was authentic |
| `q3_lowering_error` | every in-boundary node has a T carrier matching the current AST | anything about out-of-boundary nodes, ownership, effects, runtime safety |
| `q3_trusted_admit` | the returned `Sem` was computed by the trusted analysis from the given AST | that the AST matches any source file |
| `q4_trace_function` | the proofs present in the IR equal analysis carriers and resolve to file bytes | that those bytes are the intended program, or that SemCode carries the proof |
| `q4_source_bundle_error` | the expanded text is consistent with the recorded files at check time | that the files are authentic or unchanged afterwards |
| Frozen C0 `verify` on the artifact | the artifact satisfies C0's SemCode admission rules | anything about the evidence, which is not in the artifact |

Together these checks make the compiler refuse to emit when its own internal records disagree.
They are consistency checks inside one trusted program. They are not a proof system that a third
party can re-check from the artifact.

## 8. Open and unimplemented

- **Opaque evidence type.** Proposed in `PR46:docs/architecture/EVIDENCE_ADT_DESIGN_Q3_10.md`;
  not implemented. Its open decisions (representation, proof transport, source manifest, threat
  model) are unresolved.
- **Proof transport in SemCode.** Not implemented; would need a separately governed format
  revision.
- **Option constructor.** Outside the Q3 boundary (section 3.4). Whether it should carry its own
  proposition is undecided.
- **Coverage.** The boundary is defined by enumeration. Extending it is a design change, not a
  correction.

This document does not change the Bootstrap Contract, any quota, the supported subset in
[`docs/COMPILER.md`](../COMPILER.md) or the authority of Frozen C0.
