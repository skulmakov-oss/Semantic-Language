# Quad-Native Evidence Model — Q1-SAFETY

Status: **experimental implementation, checked JOIN; not a sealed Bootstrap contract**.

## Evidence carrier

```semantic
record Evidence {
    proposition_kind: text,
    subject: i32,
    scope: i32,
    state: quad,
    positive: Sequence(u32),
    negative: Sequence(u32),
}
record EvidenceResult {
    ok: bool,
    value: Evidence,
    error: text,
}
```

Proposition identity is the exact tuple `(proposition_kind, subject, scope)`, within the current compilation context. Empty kind and negative subject/scope are invalid. Source provenance is currently a pair of ordered sets of `u32` **witness identifiers**, typically `Node.pos`; source spans, file identities and complete derivation traces are **not** yet carried.

## State law

| Positive | Negative | state |
|---|---|---|
| Empty | Empty | N |
| Empty | Nonempty | F |
| Nonempty | Empty | T |
| Nonempty | Nonempty | S |

`evidence_state(positive, negative)` derives the state from the two witness families. `evidence_valid(e)` checks identity, per-family duplicate witness IDs and equality between stored `state` and the derived state. It does not prove that a witness represents a true assertion or that a source offset is globally unique.

## Admitted operations

- `evidence_empty(kind, subject, scope)`: valid empty carrier, state N.
- `evidence_positive(e, witness)` and `evidence_negative(e, witness)`: append a witness and derive the new state. These constructors can produce duplicate IDs if misused; the result is **not** automatically trusted.
- `evidence_can_join(a, b)`: boolean check of valid carriers and exact proposition identity.
- `evidence_try_join(a, b)`: **fail-closed** checked merge, returning `EvidenceResult`. Invalid evidence, mismatched identities, or duplicates created by the merge return `ok=false` plus a diagnostic string. Callers must examine `ok` before inspecting the returned value. A rejected `EvidenceResult.value` is an inert placeholder, **not** the semantic result of the rejected operation.
- The checked merge preserves each positive and negative witness in deterministic concatenation order, recalculating `state` from the combined sequences; `S` is never collapsed.

These operations are not freely interchangeable with truth-table operations: source-level `||` is the **knowledge join** for `quad`, whereas `qtruth_or` belongs to a distinct family.

## Security and representation limits

The source-level `Evidence` record remains constructible by callers. It is not unforgeable. `evidence_valid` and checked operations are the trust boundary. This model does not authenticate witness origins and cannot prevent someone from fabricating a correctly shaped `Evidence` carrier with invented offsets; provenance trust needs a separate source-indexed proof carrier.

Equal numeric witness IDs across **opposite** witness families are permitted by the current validator, but duplicate IDs **within one family** are rejected. A future evidence-authority policy must decide whether one source is allowed to support both polarities.

## Actual consumers

- Q1: `always_returns` accepts a validated carrier in state T only; underlying `return_path_evidence` still computes bare quad and then attaches root source-position witness IDs, so its T/F origins are not independently traced through branches.
- Q2: `check_expr` for value-producing `if` now accumulates two **distinct branch** witnesses and stores the checked result in `Sem.type_evidence` for inspection. See `QUAD_NATIVE_TYPE_CONSTRAINT_Q2.md`.

No change to the frozen C0, trust envelope, verifier, VM or Bootstrap Contract is implied by these experimental capabilities.
