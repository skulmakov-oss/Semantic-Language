# Bootstrap Method

Status: initial method

## Objective

Move one stable Semantic contract at a time from the Rust reference implementation into Semantic itself while preserving observable behavior and keeping regressions diagnosable.

## Slice lifecycle

Every bootstrap slice follows this sequence:

```text
1. select contract
2. capture reference behavior
3. define canonical comparison form
4. implement Semantic mirror
5. run differential corpus
6. mutation/adversarial qualification
7. freeze proven contract
8. record migration state
```

## Entry criteria

A slice may start when:

- its owner and dependency boundary are known;
- reference behavior can be observed deterministically enough to compare;
- no known open issue invalidates the exact contract being mirrored;
- required upstream inputs have a stable representation;
- expected outputs can be normalized or compared exactly.

The reference repository does not need a zero-issue backlog.

## Exit criteria

A slice is `QUALIFIED` only when:

- positive reference cases match;
- negative/error cases match where they are part of the contract;
- boundary cases are represented;
- the comparison detects at least one deliberate mutation or known divergence;
- unexplained deltas are zero;
- the proven scope is documented narrowly;
- no claim is made beyond the evidence actually collected.

## Comparison hierarchy

Prefer stronger evidence when the contract supports it:

1. **byte-for-byte identity**;
2. exact structured identity;
3. normalized structured identity;
4. equivalent diagnostics/outcomes;
5. equivalent externally observable runtime behavior.

Never weaken comparison simply to make a slice pass.

## Divergence handling

When reference and bootstrap differ:

```text
DIFFERENCE
   │
   ├─ bootstrap defect → fix bootstrap
   ├─ reference defect → fix/reference issue first
   ├─ intentional language change → explicit contract decision
   └─ unknown → STOP that slice and investigate
```

A bootstrap implementation must not silently become a competing specification.

## Upstream change handling

If the Rust reference changes a contract already under bootstrap:

1. identify affected bootstrap slices;
2. determine whether the change is semantic, representational, or qualification-only;
3. invalidate only the evidence actually affected;
4. update the mirror;
5. re-run differential qualification;
6. record the new reference point.

Completed unrelated slices remain valid unless dependency evidence proves otherwise.

## First-slice bias

Early bootstrap work should prefer modules with:

- small deterministic state space;
- narrow dependencies;
- high testability;
- stable semantics;
- little or no host interaction;
- exact comparison possibilities.

VM, host effects, and rapidly changing verifier contracts should generally migrate later than pure Foundation/source-model components.

## Anti-patterns

Do not:

- port files mechanically without first identifying their contract;
- copy Rust implementation details that are not Semantic semantics;
- declare equivalence from a few happy-path tests;
- use UI work as a bootstrap dependency;
- delete the Rust oracle before the replacement is qualified;
- mix new language design into a migration PR without an explicit design decision;
- let bootstrap-only convenience APIs become accidental public language contracts.
