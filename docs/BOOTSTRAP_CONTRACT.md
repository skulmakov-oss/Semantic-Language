# Bootstrap Contract

Status: **BOOTSTRAP CONTRACT V4 — executable protocol `shf0-bootstrap-contract-v4` in §10**

This document fixes the vocabulary and invariants of the self-hosting proof. §1–§9 state them at
architecture level; §10 records the executable protocol frozen by Bootstrap Contract v4, whose machine-readable
form is [`bootstrap/contract.toml`](../bootstrap/contract.toml) and whose validator is
[`bootstrap/validate_contract.py`](../bootstrap/validate_contract.py). The Semantic-written compiler reaches the C1/C2 fixed point (see [selfhost/README.md](./selfhost/README.md)); a Bootstrap Seal is accepted only through §10.9.

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
corpus, comparison form, positive/negative/boundary results, mutation-check status, count of
unexplained deltas (required to be 0), and explicit limitations.

## 10. Executable Protocol (`shf0-bootstrap-contract-v4`)

Protocol `shf0-bootstrap-contract-v4` succeeds `shf0-bootstrap-contract-v3`. It keeps C0, the
source-set protocol, the fixed-point rule `byte-equality-v1` (`C1 = C0(S)`, `C2 = C1(S)`,
`C1.smc == C2.smc`), the failure taxonomy and the host boundary, and changes four things:

1. the canonical source set is populated (`bootstrap/source-set.toml`: the nine `compiler/*.sm` files);
2. the stored C1 is executed through an admitted upstream execution path, `[executor]` (§10.5), and C1
   and C2 are admitted by that revision's `sm-verify`;
3. the trusted-compiler envelope for that execution is 150,000,000 steps, 2,000,000 calls and 256 frames
   (§10.5); v3 declared 20,000,000 steps and 50,000 calls, which the measured self-compilation
   (101,176,981 steps, 1,092,148 calls) exceeds;
4. the evidence record binds the executor, the effective quotas, the C1 run and determinism (§10.7), and
   the Bootstrap Seal has explicit states (§10.9).

Historical v3 runs and records stay v3 facts; they are never re-read as v4 evidence.

Protocol `shf0-bootstrap-contract-v3` succeeded `shf0-bootstrap-contract-v2`, which succeeded
`shf0-bootstrap-contract-v1`. It keeps the qualified baseline C0
(`skulmakov-oss/Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2`) and corrects three statements of v2:
`BSF-101` (text inspection) and `BSF-103` (checked `u32` arithmetic and ordering) are bound to
their exact upstream contracts ([BOOTSTRAP_SUBSET.md](./BOOTSTRAP_SUBSET.md) §5); `u32` bitwise
operations, shifts and conversions are not admitted and become candidate `BSF-107`; and the default
VM quotas are distinguished from the declared trusted-compiler envelope (§10.5). The fixed-point
rule, source-set protocol, failure taxonomy, evidence record and host boundary are unchanged.

### 10.1 C0 identity

`skulmakov-oss/Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2`
Reference manifest: [`reference/semantic-reference.toml`](../reference/semantic-reference.toml).

### 10.2 Source-set invariants (`shf0-source-set-v1`)

| Invariant | Value |
|---|---|
| Manifest | `bootstrap/source-set.toml` |
| Root | `compiler/` |
| Membership | exactly the files listed in [`bootstrap/source-set.toml`](../bootstrap/source-set.toml); never a directory scan |
| Path form | relative, POSIX `/`, ASCII, components `[a-z0-9_]+`, file suffix `.sm`; each component (with suffix) at most 100 characters and the whole relative path at most 160 (portable under Windows MAX_PATH with the checkout directory) |
| Rejected paths | absolute, drive-letter, backslash, `.`/`..`/empty components, outside root, any component that is a Windows device name (`con`, `prn`, `aux`, `nul`, `com1`–`com9`, `lpt1`–`lpt9`) |
| Links | every path component must resolve to exactly its literal location (no symlink, Windows junction or other reparse point at any depth) |
| Ordering | strictly ascending by UTF-8 bytes; an unsorted list is rejected, not re-sorted |
| Duplicates | rejected, including case-insensitive collisions |
| Bytes | UTF-8, no BOM, LF only (any CR rejected), final newline required, no NUL |
| Checkout | `.gitattributes` keeps `*.sm` LF on every platform; no normalization in the protocol |

**Identity:** `sha256` over the framed stream

```text
"SHF0-SOURCE-SET\0v1\0" || u64be(file_count)
  || for each file in manifest order: u32be(len(path)) || path || u64be(len(content)) || content
```

rendered as `sha256:<64 lowercase hex>`. Any path or content change changes the identity; the
framing makes path/content boundaries unambiguous.

### 10.3 Compiler interface

`Compile(source_set, declared_config, declared_capability_results) -> (artifact, diagnostics, status)`
with status `ok | rejected | failed`. Protocols v3 and v4 admit no configuration inputs. The artifact is
SemCode under the upstream `sm-format` contract at C0. CLI spelling is not part of the contract.

### 10.4 Fixed point

`C1 = C0(S)`, `C2 = C1(S)`. C1 and C2 are both admitted by `sm-verify`; only the stored, admitted C1 is
executed (on `sm-vm`, through `[executor]`, §10.5). Comparison rule: **`byte-equality-v1`** — `C1.smc` and `C2.smc` are equal
as complete byte sequences. Basis: at C0 the SemCode format has canonical framing, its `DBG0`
section holds only `(pc, line, col)`, no time/path/random field exists, and
`tests/cli_artifact_lifecycle.rs` asserts byte-deterministic compile output.

### 10.5 Host capabilities and execution quotas

See [HOST_BOUNDARY.md](./HOST_BOUNDARY.md) §9. Default environment inputs: none.

Execution quotas are defined upstream by `skulmakov-oss/Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2:docs/spec/quotas.md`:

- VerifiedLocal defaults: Steps 100,000, Calls 16,384, Frames 256.
- Declared trusted-compiler envelope (v4): `--envelope trusted-compiler --max-steps 150000000 --max-calls 2000000 --max-frames 256`. These are explicit selections for running the self-hosted compiler through `[executor]`, not defaults; VerifiedLocal application defaults are unchanged.

**Stored-artifact execution (`[executor]`).** C1 is executed by `smc run-smc` (stored-artifact
application mode, `docs/spec/cli.md`) built from `skulmakov-oss/Semantic@825deca47f7e415e27b1e5f845a1c3030ea92300`. It
reads the stored bytes once, requires their SHA-256 to equal the declared C1 identity
(`--expect-sha256`), admits exactly those bytes with `verify_semcode_token_with_quotas` under the one
effective configuration, and runs the admitted token on `sm-vm` with the `cli-read-only` profile
(`fs.read` under `--root`, `args`, `stdout`). It never compiles. This revision is the verifier and
runtime authority of v4 (`verifier_contract` / `runtime_contract`, §10.7). It is not C0: C0 stays the
compiler that produces C1, and its pin (`reference/semantic-reference.toml`) is unchanged.

**What the limits are.** Steps, calls and frames are VM quotas enforced by `sm-vm` inside the executor;
exceeding one stops the run (`quota_exceeded`) and no evidence is produced. They are not a memory or
wall-clock bound. Host process memory, a watchdog and process containment are host settings of the run,
not part of this protocol. The S2 harness settings used in engineering runs (job commit limit 2 GiB,
watchdog 3,600 s, active-process limit 1) are recorded as engineering evidence only; declaring them here
is not an S2 qualification. There is no unlimited execution and no automatic resource escalation.

At C0, `quotas.md` lists the `--max-calls` override as a P3 candidate; that upstream status is an
inherited limitation.

### 10.6 Failure taxonomy

All classes stop qualification (`continue = false`); evidence from a failing run is not valid.
§8 classes plus two protocol classes:

| Class | Trigger |
|---|---|
| `SOURCE_SET_INVALID` | manifest or file bytes violate `shf0-source-set-v1`; identity is undefined |
| `CONTRACT_DRIFT` | evidence bound to a different protocol, comparison rule or C0 |

### 10.7 Evidence record

An evidence record is a TOML/JSON mapping produced by a qualifying run. Required fields:

- binding: `contract_protocol`, `c0_identity`, `comparison_rule`;
- fixed point: `source_set_identity`, `c1_artifact_hash`, `c1_verifier_binding`,
  `c2_artifact_hash`, `c2_verifier_binding`, `comparison_result`;
- Seal provenance (§7): `platform` (one of the C0 qualified platforms), `verifier_contract` /
  `runtime_contract` (the pinned C0 `sm-verify` / `sm-vm`), `remaining_rust_responsibilities`
  (exactly `oracle`, `host_mechanics`, `verifier_runtime_foundation`, each once);
- qualification evidence (§9): `input_corpus` (sha256), `input_corpus_size` (positive integer),
  `positive_cases` / `negative_cases` / `boundary_cases` (each `{count > 0, result = pass}`,
  counts summing to `input_corpus_size`), `mutation_proof` = `detected`,
  `unexplained_deltas` = `0`, `limitations` (non-empty list of what the evidence does not prove);
- C0 checkout attestation: `c0_checkout_head` (= C0 SHA) and `c0_checkout_clean` (= true);
- stored execution (v4): `executor_identity` (`<repository>@<sha>:crates/smc-cli` of `[executor]`),
  `effective_quotas` (exactly the declared envelope), `c1_execution` (`{outcome = ok, counters = {steps,
  calls, peak_frames}}` of the run of C1 on `S`, each counter within the envelope) and `determinism`
  (= `identical`: repeated compilations by the same generation gave identical output).

In v4, `verifier_contract` / `runtime_contract` name `crates/sm-verify` / `crates/sm-vm` of the
`[executor]` revision.

**Record validation (what SHF-0 proves).** `check_evidence(repo, record)` is the only record
gate. It loads the canonical contract files from the repository, requires them to validate,
computes the current `S` identity itself, and then checks every internally checkable invariant:
exact C0 and protocol binding (else `CONTRACT_DRIFT`), current `S` identity, hash formats,
both verifier bindings `admitted` (else `ADMISSION_REJECT`), `comparison_result` = `equal` and
identical C1/C2 hashes under `byte-equality-v1` (else `FIXED_POINT_DELTA`), complete corpus and
case evidence, platform / contracts / responsibilities, and the attested checkout values (a head
other than C0 or a non-`true` clean flag is `REFERENCE_MISMATCH`). It fails closed on malformed
input. Passing it means the record is well-formed and internally consistent — nothing more.

**Production (what a producer MUST do).** The checkout fields are producer attestations, not
cryptographic proof of execution provenance. A producer MUST run `verify_reference_checkout()`
on the C0 checkout immediately before the qualifying run and record its outcome. A post-run
record cannot prove how it was produced, so `check_evidence` validates the attested values but
cannot establish that the check actually ran. Upstream release artifacts are unsigned; SHF-0
adds no signing.

**Seal acceptance (what is additionally required).** Self-attested evidence alone is NOT
sufficient for the Bootstrap Seal. The Seal is accepted only when the producing run is bound to
the canonical reference-check procedure AND an independent reproduction from the exact qualified
C0 and canonical `S` confirms the claimed artifact identities. A reproduction mismatch is a
qualification failure (`FIXED_POINT_DELTA` or `REFERENCE_MISMATCH`) and cannot be waived by the
stored attestation.

### 10.8 Stage ownership of evidence

| Stage | Responsibility |
|---|---|
| SHF-0 | freezes the provenance requirements; validates record structure and internal consistency |
| SHF-15 | produces C1 from the verified exact C0 and canonical `S`; binds evidence to the actual producing run |
| SHF-16 | produces C2; performs the fixed-point proof; independently reproduces the required evidence; gates the Bootstrap Seal |

### 10.9 Bootstrap Seal states

A Seal candidate (`shf0-bootstrap-seal-v1`) holds the evidence record, the producing run's `host_id` and
`operator`, the reproductions and the acceptance. `check_seal` reports the furthest state reached; a
state is reached only when every earlier one holds:

| State | Condition |
|---|---|
| `validated` | the evidence record passes `check_evidence` |
| `reproduced` | a reproduction by a distinct operator on a distinct host, from the same C0 and `S`, reports identical `c0_identity`, `source_set_identity`, C1 and C2 identities and comparison result |
| `accepted` | the owner accepted the reproduced Seal |

A reproduction that reports different identities invalidates the candidate (`FIXED_POINT_DELTA`). A second
run on the same host or by the same operator is repeatability evidence, not a reproduction. Host ids,
operator names and the acceptance are attestations; the gate checks their values, not who wrote them.
