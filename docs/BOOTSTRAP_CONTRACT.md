# Bootstrap Contract

Status: **architecture-level contract — SHF-0 NOT STARTED**

This document fixes the vocabulary and invariants of the self-hosting proof at architecture
level. SHF-0 (see [ROADMAP.md](./ROADMAP.md)) turns it into an executable bootstrap protocol:
concrete source tree, normalization rule, capability list and failure handling. Nothing here
claims that SHF-0 has been performed.

## 1. Terms

| Term | Definition |
|---|---|
| `S` | The source set of the self-hosted compiler, written in Semantic, identified by a source-set identity (§3). |
| C0 | The qualified Rust-hosted reference compiler: `skulmakov-oss/Semantic` at an exact SHA recorded in [`reference/semantic-reference.toml`](../reference/semantic-reference.toml). |
| C1 | `C0(S)`, a SemCode artifact. |
| C2 | `C1(S)`, a SemCode artifact produced by executing admitted C1. |
| Fixed point | `Canonical(C1.smc) == Canonical(C2.smc)`; byte equality `C1.smc == C2.smc` is preferred where the format permits. |
| Bootstrap Seal | The provenance record created when the fixed point passes (§7). |

## 2. Reference identity

- C0 is identified only by `repository + exact SHA`.
- Floating `main` is never a qualification oracle.
- A local checkout is valid only if its HEAD equals the pinned SHA and its tree is clean.
- Changing the pin requires an explicit drift analysis recording the commits between the old
  and new SHA and which bootstrap evidence they invalidate.

## 3. Source-set identity

`S` is identified by a deterministic digest over its declared file list and contents, with the
file list in a canonical order and paths relative to the repository root. The exact digest
construction is an SHF-0 deliverable.

## 4. Comparison rule

- The comparison rule (byte identity or a named canonicalization) is frozen **before** the
  final C1/C2 run and cannot be weakened after observing a failure.
- Every difference is either zero or an explained, recorded delta. The required count of
  unexplained deltas is zero.
- Fixed-point equality is bootstrap evidence. It does not bypass verifier admission: C1 and C2
  are both admitted by `sm-verify`. Only admitted C1 is executed (by `sm-vm`) to produce C2; C2 is
  admitted and compared, not executed, as part of the fixed-point proof.

## 5. Host boundary

Host adapters perform mechanics only. Permitted mechanics and forbidden compiler logic are
defined in [HOST_BOUNDARY.md](./HOST_BOUNDARY.md). Summary:

- **Permitted:** read declared input bytes; write declared artifact bytes; expose declared
  capabilities; transport arguments/results; run the existing verifier and VM.
- **Forbidden:** tokenizing, parsing, binding, typechecking, lowering, choosing IR or opcodes,
  constructing compiler output, reinterpreting diagnostics, bypassing admission.

## 6. Deterministic input set

The compiler's observable output may depend only on declared inputs: `S`, the C0 reference
identity, declared configuration, and declared capability results.

Forbidden hidden inputs:

- wall clock / timestamps;
- random state;
- filesystem enumeration order;
- undocumented environment variables;
- machine-local absolute paths in canonical output;
- process or thread identity;
- locale;
- unrecorded process-global state.

Any legitimately needed external value must become a declared input and part of the evidence.

## 7. Bootstrap Seal

Recorded only when the fixed point passes with zero unexplained deltas. It contains at least:

- C0 reference repository and SHA;
- source-set identity of `S`;
- C1 and C2 artifact identities;
- the frozen comparison rule;
- verifier and runtime contract identities used;
- platforms on which the proof ran;
- the explicit list of remaining non-compiler Rust responsibilities (oracle, host mechanics,
  verifier/runtime Foundation).

## 8. Failure taxonomy

| Class | Meaning | Required action |
|---|---|---|
| `REFERENCE_MISMATCH` | Checkout HEAD differs from pin or tree is dirty | Stop; do not qualify |
| `CAPABILITY_GAP` | `S` needs a language/runtime capability Semantic lacks | Add upstream first; never in bootstrap glue |
| `ADMISSION_REJECT` | `sm-verify` rejects C1 or C2 | Stop; fix the compiler, never relax admission |
| `FIXED_POINT_DELTA` | `Canonical(C1) != Canonical(C2)` | Investigate; unexplained deltas block the Seal |
| `NONDETERMINISM` | Repeated runs on identical inputs differ | Find the hidden input; block the Seal |
| `HOST_LOGIC_LEAK` | Host performs compiler-domain work | Remove; evidence collected through it is invalid |
| `REFERENCE_DEFECT` | C0 itself is wrong | Fix upstream first, then re-pin |

## 9. Evidence requirements

Each qualification run records: reference repository and SHA, source-set identity, input
corpus, comparison form, positive/negative/boundary results, mutation proof, and unexplained
delta count (required: 0). See [QUALIFICATION.md](./QUALIFICATION.md).
