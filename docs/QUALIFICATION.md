# Qualification

Status: canonical qualification discipline (SHF reset)

Supersedes the earlier `docs/BOOTSTRAP.md` method and incorporates the evidence discipline
developed in PR #3. Applies to every unit that moves through the migration states in
[ARCHITECTURE.md](./ARCHITECTURE.md) and to the C1/C2 proof in
[BOOTSTRAP_CONTRACT.md](./BOOTSTRAP_CONTRACT.md).

## 1. Comparison hierarchy

1. byte-for-byte identity
2. exact structured identity
3. normalized structured identity
4. contractual diagnostic/result equivalence
5. deterministic externally observable behavior

Use the strongest comparison the contract supports. **Never weaken a comparison after observing
a failure merely to make qualification pass.**

Typical forms: tokens + spans (lexer); normalized AST (parser); type/binding facts and
diagnostics (sema); canonical normalized IR (lowering); bytes (SemCode); accept/reject +
contractual diagnostics (admission); canonical C1/C2 artifacts (fixed point).

## 2. Unit lifecycle

```text
1. select contract
2. pin reference (repository + exact SHA)
3. capture reference behavior mechanically
4. define canonical comparison form
5. implement Semantic mirror
6. run differential corpus
7. mutation / adversarial proof
8. freeze proven contract (narrowly)
9. record migration state
```

## 3. Required record per unit

| Field | Content |
|---|---|
| Reference repository | `skulmakov-oss/Semantic` |
| Reference SHA | exact commit |
| Source-set identity | where `S` is involved |
| Input corpus | identity and size |
| Comparison form | tier from §1 and exact schema |
| Positive cases | valid behavior |
| Negative cases | rejection/error behavior where contractual |
| Boundary cases | domain edges |
| Mutation proof | at least one deliberate defect that the gate detects |
| Unexplained deltas | **required: 0** |
| Limitations | what the evidence does not prove |

## 4. Evidence discipline

Salvaged from PR #3; each item is a requirement for any gate built in this repository.

- **Exact reference pinning.** The gate verifies the reference checkout HEAD equals the pinned
  SHA and the tree is clean before running, and fails otherwise. The reference (C0) is evaluated as
  a reproducible historical executable reference, not an infallible normative specification. An explicit
  drift override may exist only for non-qualification diagnostics: output of an override run is never qualification
  evidence, is never attributed to the pinned SHA, and cannot advance migration state.
- **Mechanical extraction.** Reference vectors are produced by running the reference, never
  hand-typed or transcribed.
- **Explicit comparison schema.** Every compared field is named; a missing or wrongly shaped
  field on either side is a failure.
- **Positive, negative and boundary vectors.**
- **Exhaustive finite domains.** Where the state space is small, enumerate all of it and check
  coverage (every combination exactly once), not just counts.
- **Mutation / adversarial proof.** Show that a meaningful defect makes the gate fail, including
  a mutation in the real runtime path, not only in the corpus.
- **Independent normative oracle where appropriate.** Compare corpus and live extraction against
  a third, independently derived oracle, so synchronized two-sided corruption cannot pass.
- **Fail-closed structural validation.** Frozen constants are compared by exact value, not only
  by shape.
- **Runtime-path verification.** Exercise the path the runtime actually uses (source → lowering
  → SemCode → verifier → VM where the contract is a source-level behavior), not a convenient
  internal helper.
- **Host ABI boundary verification** in both directions where a boundary encoding is claimed.
- **Ordering independence.** Results must not depend on incidental iteration order of the
  reference's data structures.
- **Target-directory isolation.** Builds of the reference must not mutate the reference checkout's
  tracked state or share build output with it.
- **Exit-code checking.** Process/cargo exit codes are trusted independently of parsed summaries.
- **Explicit limitation disclosure.** Every gate states what it does not cover.
- **Zero unexplained deltas.**

## 5. Distinct operation families stay distinct

A gate must not merge operation families that the reference defines separately. Example from
PR #3: the legacy lattice Quad operations (`QNot`/`QAnd`/`QOr`/`QImpl`) and the QTruth
operations (`QTruth*`) are different families; some operations coincide today and others do
not. Qualification of one family says nothing about the other.

## 6. Divergence handling

```text
DIFFERENCE
   ├─ bootstrap defect          -> fix bootstrap
   ├─ reference defect          -> fix upstream first, then re-pin
   ├─ intentional language change -> explicit upstream contract decision
   └─ unknown                   -> STOP that unit and investigate
```

## 7. Upstream change handling

When the pinned reference changes a contract under qualification: identify affected units;
classify the change (semantic / representational / qualification-only); invalidate only the
affected evidence; update the mirror; re-run; record the new SHA. Unrelated qualified units
remain valid unless dependency evidence says otherwise.

## 8. Anti-patterns

- porting files mechanically without identifying their contract;
- declaring equivalence from happy-path tests;
- weakening a gate to get green CI;
- removing the Rust oracle before the replacement is qualified;
- letting bootstrap convenience APIs become language contracts.
