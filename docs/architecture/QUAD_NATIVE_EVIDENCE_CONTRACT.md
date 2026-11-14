# Quad-native evidence contract

Status: implementation contract for the Semantic-written compiler in `compiler/`, as it exists on
`main`. It records what the code does and what its tests check. It is not a language
specification, not a Bootstrap Contract document and not a stage-promotion record.

## 1. Scope and authority

- **Covered:** the compiler-internal `Evidence` carrier, its state law, the checked join and the
  type propositions that semantic analysis records.
- **Not covered:** the meaning of `quad` in the Semantic language. That is owned by the upstream
  `Semantic` repository (`docs/spec/quad_logic_frame_v1.md`, `docs/spec/source_semantics.md` at the
  pinned C0 revision). This document only states which part of that surface this compiler admits
  and how it uses four-state values internally.
- **Source of truth:** where this text and the code disagree, the code and its tests are right and
  this text is wrong.
- **Proof boundaries** (who checks evidence, and what a passed check means) are in
  [QUAD_NATIVE_PROOF_BOUNDARIES.md](QUAD_NATIVE_PROOF_BOUNDARIES.md).

Historical sources are the experimental design documents of PR #46 at commit
`bd2206fae01d3a3052eee3359c23c28551a93e0f`. They are cited below as `PR46:<file>`; they remain in
Git history and are not part of `main`. A statement from them appears here only if current code
supports it.

Test anchors: `TQN` is `tests/compiler/test_quad_native.py`, `TPF` is
`tests/compiler/test_pr46_followup.py`, fixtures are in `tests/compiler/programs/`.

## 2. Four states

| State | Positive witnesses | Negative witnesses | Source literal | `LOAD_Q` immediate |
|---|---|---|---|---|
| N | none | none | `N` | 0 |
| F | none | some | `F` | 1 |
| T | some | none | `T` | 2 |
| S | some | some | `S` | 3 |

The two planes are independent: one records support for a proposition, the other records support
against the same proposition. S is conflict, not an error code, and N is absence of evidence, not
falsehood.

| | |
|---|---|
| Implementation | `evidence_state` in `compiler/sema.sm`; literal lowering in `lower_expr` (`compiler/lower.sm`, node kind 9) |
| Tests | TQN `test_full_quad_truth_4x4_and_match_vs_c0`; `quad_flow_internal.sm` |
| Historical source | `PR46:docs/QUAD_NATIVE_ARCHITECTURE_AUDIT.md` §2; `PR46:docs/QUAD_NATIVE_BOOTSTRAP_EXPERIMENT.md`, "Four-state model" |
| Guarantee | the stored state of a valid carrier is exactly the state derived from its two witness sequences |
| Not guaranteed | that the encoding matches any consumer other than pinned C0; this compiler emits what C0's verifier and VM accept |

## 3. The `Evidence` carrier

```semantic
record Evidence {
    proposition_kind: text,
    subject: i32,
    scope: i32,
    state: quad,
    positive: Sequence(u32),
    negative: Sequence(u32),
}
```

### 3.1 Proposition identity

The identity of a proposition is the tuple `(proposition_kind, subject, scope)`.

- `subject` is the AST node id that owns the proposition.
- `scope` is 0 for every proposition the compiler creates today.
- A witness is a `u32`. The compiler uses the source byte offset (`Node.pos`) of the contributing
  AST node in the expanded compilation unit.
- Witnesses are owned by the proposition they are recorded under. Positive and negative witnesses
  must speak about the same proposition; unrelated facts are never encoded as opposite polarities.

| | |
|---|---|
| Implementation | `record Evidence`, `evidence_empty` in `compiler/sema.sm` |
| Tests | `quad_flow_internal.sm`; `quad_type_constraint_internal.sm` |
| Historical source | `PR46:docs/architecture/QUAD_NATIVE_EVIDENCE_MODEL.md`; `PR46:docs/QUAD_NATIVE_ARCHITECTURE_AUDIT.md` §1 and §5 |
| Guarantee | two carriers describe the same proposition only if all three identity fields are equal |
| Not guaranteed | global uniqueness of a witness: an offset identifies a position in one expanded unit, not a file, and not a derivation |

### 3.2 Construction

| Function | Result |
|---|---|
| `evidence_empty(kind, subject, scope)` | carrier with no witnesses, state N |
| `evidence_positive(e, witness)` | appends to the positive sequence and re-derives the state |
| `evidence_negative(e, witness)` | appends to the negative sequence and re-derives the state |

The two appending constructors do not check for duplicates. Their result is not trusted until it
passes `evidence_valid` or a checked join.

### 3.3 Validity

`evidence_valid(e)` holds when all of the following hold:

1. `proposition_kind` is not empty, and `subject` and `scope` are not negative.
2. Neither witness sequence contains the same value twice.
3. `state` equals `evidence_state(positive, negative)`.

The same value may appear once in each of the two sequences. The validator allows that; the
compiler uses it to build an S carrier in `always_returns`.

| | |
|---|---|
| Implementation | `evidence_valid`, `evidence_has_duplicate` in `compiler/sema.sm` |
| Tests | `quad_flow_internal.sm` (forged S without witnesses and a duplicated witness are both invalid) |
| Historical source | `PR46:docs/architecture/QUAD_NATIVE_EVIDENCE_MODEL.md`, "State law" |
| Guarantee | a valid carrier is internally consistent: identity well formed, no duplicate within a family, state derived |
| Not guaranteed | that any witness is true, that an offset points at the right node, or that the carrier was produced by semantic analysis. `Evidence` is an ordinary public record and can be constructed by hand |

### 3.4 Duplicate detection during accumulation

`accumulate_type_constraint` adds one witness to the proposition of an owner node. It keeps a
per-owner index (`C.type_witness_seen`) so that a repeated witness of the same polarity is found
without rescanning the sequence. A duplicate is a compile error:
`invalid accumulated type evidence: duplicate evidence witness`.

| | |
|---|---|
| Implementation | `accumulate_type_constraint` in `compiler/sema.sm` |
| Tests | TPF `test_duplicate_witness_rejected_in_incremental_accumulator`, `test_large_sequence_does_not_exhaust_steps`; `quad_evidence_incremental_internal.sm` |
| Historical source | `PR46:docs/architecture/CODEX_PR46_P2_FOLLOWUP_20261101.md` (commit `9a9e818`) |
| Guarantee | the index agrees with the carrier it extends; a pre-existing carrier is validated once before the index is built |
| Not guaranteed | protection against a carrier that was altered outside this accumulator between two calls |

## 4. Checked join

`evidence_try_join(a, b)` returns an `EvidenceResult { ok, value, error }`.

| Order | Condition | Result |
|---|---|---|
| 1 | either carrier is not valid | rejected, `invalid evidence carrier` |
| 2 | identities differ | rejected, `incompatible proposition identity` |
| 3 | the concatenated positive or negative sequence has a duplicate | rejected, `duplicate evidence witness` |
| 4 | otherwise | accepted: `a`'s witnesses followed by `b`'s, state re-derived |

`evidence_can_join(a, b)` is the boolean form of conditions 1 and 2.

Properties that follow from the code and are checked by tests:

- **Conflict is kept.** Joining a T carrier with an F carrier of the same proposition gives S with
  both witnesses. S is never reduced to T or F.
- **Order is kept.** Witness order is `a` then `b`; the result is deterministic.
- **Not idempotent.** Joining a carrier that has a witness with itself is rejected as a duplicate.
- **Rejected value is inert.** On rejection `value` is a placeholder; callers must test `ok`.

Callers in the compiler: `if-branch-type` and `binary-operator-types` in `check_expr`, and
`builtin-argument-types` in `check_builtin`. The other propositions are built with
`accumulate_type_constraint` (section 3.4) or as single-witness carriers.

| | |
|---|---|
| Implementation | `evidence_try_join`, `evidence_can_join`, `evidence_reject` in `compiler/sema.sm` |
| Tests | `quad_flow_internal.sm`, `quad_type_constraint_internal.sm`, `quad_control_inference_internal.sm`, `quad_map_dependency_internal.sm` (cross-proposition join rejected) |
| Historical source | `PR46:docs/architecture/QUAD_NATIVE_EVIDENCE_MODEL.md`, "Admitted operations"; `PR46:docs/architecture/QUAD_NATIVE_PROPAGATION_MAP.md`, rules 1–4 |
| Guarantee | a successful join never merges two different propositions and never produces an invalid carrier |
| Not guaranteed | commutativity of witness order; any relation to the source-level operator `\|\|` on `quad` values, which is a different operation (section 6) |

`main` validates each carrier once in this function. PR #46 validated twice; the accepted set,
the rejection reasons and the merged value are the same.

## 5. Q2 type propositions

Semantic analysis records one proposition per owner node. A positive witness means "this
contributor satisfies the constraint", a negative witness means "this contributor violates the same
constraint". A violated constraint also produces the ordinary `type` diagnostic; the evidence does
not replace it.

Carriers are stored in six maps of `Sem`, keyed by AST node id.

### 5.1 `Sem.type_evidence`

| Proposition kind | Owner node | Witnesses, in order |
|---|---|---|
| `if-branch-type` | value-producing `if` (26) | then block, else block |
| `unary-operator-type` | unary operator (5) | operand |
| `binary-operator-types` | binary operator (6) | left operand, right operand |
| `call-arguments` | call of a user function with arguments (7) | each argument |
| `builtin-argument-types` | call of a non-map builtin (7) | each argument |
| `record-fields` | record literal (29) or copy-with (21) with fields | each field |
| `tuple-target-items` | tuple literal (22) with a contextual tuple type | each item |
| `sequence-elements` | non-empty sequence literal (23) | each element |
| `match-arm-types` | `match` (25) whose result type is checked | result expression of each arm |
| `annotated-binding` | `let` with an annotation (10) | initializer |
| `assignment-type` | assignment (11) | right-hand side |
| `return-value-type` | `return` with a value (14) | returned expression |
| `tuple-annotation-type` | tuple destructuring `let` with an annotation (40) | right-hand side |
| `tuple-assignment-items` | tuple assignment (41) | each target |
| `if-condition-type` | `if` statement (12) | condition |
| `while-condition-type` | `while` (13) | condition |
| `assert-condition-type` | `assert` (17) | condition |
| `for-sequence-type` | `for … in` (42) | iterated expression |

### 5.2 Separate maps

| Map | Proposition kind | Owner | Witness |
|---|---|---|---|
| `map_container_evidence` | `map-container-type`; `map-context-type` for `map_empty` | map builtin call | container argument; the call itself for `map_empty` |
| `map_key_evidence` | `map-key-type` | map builtin call | key argument |
| `map_value_evidence` | `map-value-type` | `map_get`, `map_set` | value or default argument |
| `tuple_shape_evidence` | `tuple-destructure-shape` | tuple destructuring (40, 41) | right-hand side |
| `if_expr_condition_evidence` | `if-expression-condition-type` | value-producing `if` (26) | condition |

Container, key and value of one map call are three propositions. They are never joined.

### 5.3 Derivation facts that are not evidence

| Field of `Sem` | Meaning |
|---|---|
| `inferred_binding_types` | type inferred for an unannotated `let` |
| `tuple_inferred_components` | component types inferred for a tuple destructuring |
| `tuple_expected` | contextual tuple type of a tuple literal |
| `match_result_checked` | whether a `match` registered a result-type obligation |

These record how a type was derived. They are deliberately not positive evidence: an inferred type
is not a compatibility comparison, so no T witness is fabricated for it.

### 5.4 Constraints specific to operators

`binary-operator-types` records whether each operand type is admissible for the chosen operator,
not merely whether the two types are equal (`binary_type_admitted`):

| Operators | Admissible operand types |
|---|---|
| arithmetic | `i32`, `u32`; `text` for `+` |
| `==`, `!=` | any type accepted by `supports_eq`: `i32`, `bool`, `u32`, `text`, `quad`, and acyclic records, tuples, `Sequence` and `Option` of comparable types |
| ordering | `i32`, `u32` |
| `&&`, `\|\|` | `bool`, `quad` |

Both operands must have the same type.

| | |
|---|---|
| Implementation | `check_expr`, `check_record_expr`, `check_builtin`, `accumulate_type_constraint`, `type_branch_witness`, `map_type_witness`, `binary_type_admitted`, `unary_type_admitted`, `supports_eq` in `compiler/sema.sm` |
| Tests | TQN `test_type_constraint_evidence_from_real_sema`, `test_q2_call_and_record_type_evidence`, `test_q2_q3_q4_binding_assignment_return_evidence`, `test_q2_q3_q4_tuple_sequence_match_provenance`, `test_q25_operator_and_builtin_real_sema_lowering`, `test_q26_map_dependent_evidence_and_lowering`, `test_q27_tuple_shape_assignment_and_inference_origins`, `test_q28_control_conditions_inference_and_provenance`; `tests/compiler/test_structural_equality.py` |
| Historical source | `PR46:docs/architecture/QUAD_NATIVE_TYPE_CONSTRAINT_Q2.md`, `_Q2_4.md`, `_Q2_5.md`, `_Q2_6.md`, `_Q2_7.md`, `_Q2_8.md`; `PR46:docs/architecture/QUAD_NATIVE_TRANSFORMATION_MAP.md` |
| Guarantee | for every listed owner that semantic analysis accepts, a valid carrier in state T exists with one positive witness per contributor at that contributor's source offset |
| Not guaranteed | complete coverage of the type system. Expressions not listed have no proposition (see the proof-boundaries document, "Outside the boundary"). Structural equality was admitted on `main` after PR #46; PR #46 itself admitted `==` only on scalar types and `quad` |

## 6. Compiler-internal use of `quad` values

Apart from evidence states, the compiler computes with `quad` in one place.

### 6.1 Return-path evidence

`return_path_evidence(nodes, id)` returns a `quad` for a statement or block: T if every path
returns, F if every path falls through, S if both happen on different paths, N if no path outcome
is witnessed.

| Construct | Rule |
|---|---|
| `return` | T |
| alternative branches | joined with the source operator `\|\|` (knowledge join) |
| `if` without `else` | the branch joined with F |
| sequence `a; b` | T stays T; F continues with `b`; S becomes `T \|\| evidence(b)`; N stays N |
| `match` | arms joined in source order; the first `_` arm ends the analysis; without `_` a fall-through is added |
| loops | contribute a fall-through possibility |

`always_returns` accepts only state T. S and N are not success.

| | |
|---|---|
| Implementation | `return_path_evidence`, `always_returns` in `compiler/sema.sm` |
| Tests | TQN `test_semantic_analyzer_return_path_witnesses`; `quad_flow_internal.sm`; corpus cases `p16_if_else_both_return`, `n25_if_else_both_return_not_enough` |
| Historical source | `PR46:docs/QUAD_NATIVE_ARCHITECTURE_AUDIT.md` §2 |
| Guarantee | a function with a non-`unit` return type is accepted only when the analysis reaches T |
| Not guaranteed | precision. The analysis is conservative: it does not reason about conditions, loop termination or reachability. The carrier built in `always_returns` has one root witness; it does not trace which branch produced which outcome |

### 6.2 What stays `bool`

Structural facts with two outcomes stay `bool`: whether a name is already declared, whether a map
has a key. Duplicate function and record names are found with `map_contains`. An earlier
experiment encoded "declared once" and "declared twice" as T and F of one proposition; that was
removed as unsound and is not part of this contract.

## 7. Determinism

- Analysis visits nodes in a fixed order, so witness order inside a carrier is a function of the
  source text.
- The same source produces the same carriers on every run. `q3_trusted_admit` relies on this: it
  re-runs `analyze` and compares carriers field by field.
- Evidence is not written to SemCode, so it does not affect artifact bytes.

Self-compilation reproduces byte-identical compiler artifacts (see the README checkpoint). That
shows the compiler is deterministic on its own source. It is not evidence about the content of this
contract.

## 8. Limitations and unsupported claims

1. `Evidence`, `EvidenceResult` and `Sem` are public records. Nothing prevents a Semantic program
   from constructing a well-formed carrier in state T. The compiler does not treat a carrier as
   authority; see the proof-boundaries document.
2. A witness is a byte offset. It carries no file identity and no derivation.
3. Evidence is internal to one compiler run. It is not serialized, not verified by C0's verifier
   and not visible to the VM.
4. The propositions cover type compatibility at the listed owners. They say nothing about
   ownership, effects, runtime safety or verifier correctness.
5. An opaque, unforgeable evidence type was proposed in
   `PR46:docs/architecture/EVIDENCE_ADT_DESIGN_Q3_10.md`. It is **not implemented**.
6. The truth-table operator family named in the PR #46 documents (`qtruth_*`) is not implemented in
   this compiler. Source `&&` and `||` on `quad` are the knowledge meet and join.
7. This document does not change the Bootstrap Contract, the supported subset in
   [`docs/COMPILER.md`](../COMPILER.md) or any quota.
