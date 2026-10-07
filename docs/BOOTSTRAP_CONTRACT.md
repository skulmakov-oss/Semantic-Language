# Bootstrap Contract

Status: **SHF-0 IN PROGRESS (issue #11) — executable protocol `shf0-bootstrap-contract-v1` in §10**

This document fixes the vocabulary and invariants of the self-hosting proof. §1–§9 state them at
architecture level; §10 records the executable protocol frozen by SHF-0, whose machine-readable
form is [`bootstrap/contract.toml`](../bootstrap/contract.toml) and whose validator is
[`bootstrap/validate_contract.py`](../bootstrap/validate_contract.py). No compiler exists yet.

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

## 10. Executable protocol (SHF-0)

Protocol `shf0-bootstrap-contract-v1`. Changing any value below is a contract revision with a
new protocol identifier, never an edit made to fit a failed run (`CONTRACT_DRIFT`).
The validator enforces this mechanically: it binds the protocol identifier to a sha256 of the
canonical form of every frozen value: all of `bootstrap/contract.toml` except the evolving
`[subset]` state lists (admitted / candidate / frozen, checked through the registry linkage),
together with the whole C0 reference manifest `reference/semantic-reference.toml`.

### 10.1 C0

`skulmakov-oss/Semantic@89641da8237f4fcefb50cf1958a50e4d4003aea7` — the `v1.2.0` Stable
Foundation release, verdict "ORACLE QUALIFIED WITH EXPLICIT LIMITS" (qualification campaign
`Semantic#1983`, platform `x86_64-pc-windows-msvc`, limits R1–R4). Upstream development `main`
(`5f3e2302…`) is 16 commits ahead with compiler-relevant changes and does **not** inherit that
qualification. Details and contract paths: [`reference/semantic-reference.toml`](../reference/semantic-reference.toml).

SemCode format ownership at C0 is taken from `CONSTRAINTS.md` (`sm-format` owns the binary
format); the "owner: `sm-ir`" wording in `docs/spec/semcode.md` is historical per that file.

### 10.2 Source set `S` (`shf0-source-set-v1`)

| Rule | Value |
|---|---|
| Root | `compiler/` |
| Membership | exactly the files listed in [`bootstrap/source-set.toml`](../bootstrap/source-set.toml); never a directory scan |
| Path form | relative, POSIX `/`, ASCII, components `[a-z0-9_]+`, file suffix `.sm` |
| Rejected paths | absolute, drive-letter, backslash, `.`/`..`/empty components, outside root |
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
with status `ok | rejected | failed`. Protocol v1 admits no configuration inputs. The artifact is
SemCode under the upstream `sm-format` contract at C0. CLI spelling is not part of the contract.

### 10.4 Fixed point

`C1 = C0(S)`, `C2 = C1(S)`. C1 and C2 are both admitted by `sm-verify`; only admitted C1 is
executed (on `sm-vm`). Comparison rule: **`byte-equality-v1`** — `C1.smc` and `C2.smc` are equal
as complete byte sequences. Basis: at C0 the SemCode format has canonical framing, its `DBG0`
section holds only `(pc, line, col)`, no time/path/random field exists, and
`tests/cli_artifact_lifecycle.rs` asserts byte-deterministic compile output.

### 10.5 Host capabilities

See [HOST_BOUNDARY.md](./HOST_BOUNDARY.md) §9. Default environment inputs: none.

### 10.6 Failure taxonomy

All classes stop qualification (`continue = false`); evidence from a failing run is not valid.
§8 classes plus two protocol classes:

| Class | Trigger |
|---|---|
| `SOURCE_SET_INVALID` | manifest or file bytes violate `shf0-source-set-v1`; identity is undefined |
| `CONTRACT_DRIFT` | evidence bound to a different protocol, comparison rule or C0 |

### 10.7 Evidence record

Required fields: `contract_protocol`, `c0_identity`, `source_set_identity`, `c1_artifact_hash`,
`c1_verifier_binding`, `c2_artifact_hash`, `c2_verifier_binding`, `comparison_rule`,
`comparison_result` (upstream shape: `docs/security/artifact_provenance_and_signing_policy_v0.md`
§7). A record whose protocol, comparison rule or C0 differ from the contract is `CONTRACT_DRIFT`.
A record supports the Bootstrap Seal only if its values hold, not merely its keys: hashes are
`sha256:<64 lowercase hex>`, `source_set_identity` equals the identity of the current `S`, both
verifier bindings are `admitted` (else `ADMISSION_REJECT`), `comparison_result` is `equal`, and
under `byte-equality-v1` the C1 and C2 hashes are identical (else `FIXED_POINT_DELTA`).
