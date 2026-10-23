# Proposal: C0 Reference Transition to `Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2`

| | |
|---|---|
| Status | **PROPOSAL — CANDIDATE NOT YET QUALIFIED AS BOOTSTRAP C0.** Nothing in `reference/**` or `bootstrap/**` is altered by this document. |
| Historical C0 | `skulmakov-oss/Semantic@89641da8237f4fcefb50cf1958a50e4d4003aea7` (`v1.2.0`) |
| Historical Exploratory Reference | `skulmakov-oss/Semantic@94c5c50d16c3f2efc7691456755461bedae83b18` |
| New Upstream Candidate C0 | `skulmakov-oss/Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2` |
| Distribution Release Provenance | `skulmakov-oss/Semantic-Dist@v1.3.0-beta.1` (Windows x64 Developer Preview) |
| Parent Authority | `skulmakov-oss/Semantic#1910` |
| Decision Owner | Repository Owner |

---

## 1. Context and Need for Transition

The bootstrap implementation repository (`skulmakov-oss/Semantic-Language`) is tasked with advancing the Semantic compiler toward self-hosting under the authoritative equation:
```text
S = compiler source written substantially in Semantic
C0(S) -> C1.smc
C1(S) -> C2.smc
Canonical(C1.smc) == Canonical(C2.smc)
```

The historical C0 reference pin (`Semantic@89641da8`, release `v1.2.0`) does not admit:
1. Decomposing `text` byte-by-byte (`text_len`, `text_byte_at`, `text_slice` do not exist).
2. Native `u32` arithmetic, indexing, or relational ordering.
3. Modern must-return rules (historical C0 rejected symmetric `if/else` returns).
4. Configurable Calls quotas for trusted compiler workloads.

Without text inspection (BSF-101) and `u32` arithmetic (BSF-103), writing a lexer and parser in Semantic is impossible.

### Important Release vs Oracle Distinction
The publication of `skulmakov-oss/Semantic-Dist@v1.3.0-beta.1` provides a verified Windows x64 distribution package built from upstream commit `1d1e76c7509a03baa0133abaaa5819f5d32582b2`. However, `v1.3.0-beta.1` is a distribution developer preview, **not** a separately qualified upstream bootstrap oracle. Therefore, commit `1d1e76c7509a03baa0133abaaa5819f5d32582b2` is treated as a **Candidate C0** until bootstrap-specific qualification and explicit Owner promotion are complete.

---

## 2. Toolchain Identities

| Environment / Role | Commit SHA / Tag | `source_fingerprint` | Executable SHA-256 |
| :--- | :--- | :--- | :--- |
| **Historical C0 (v1.2.0)** | `89641da8237f4fcefb50cf1958a50e4d4003aea7` | `7f10f50c82216db2` | `cbb435d38d119bbefbc197aee584028e289394a393b51e8bcaac742fb8744eb8` |
| **Exploratory Reference** | `94c5c50d16c3f2efc7691456755461bedae83b18` | `1cd4707ec728c07a` | `efac7456ce51feb24f5ac3ef6e2c5b881552b9227171031ca5f73fab50458485` |
| **Candidate C0 (Candidate Oracle)** | `1d1e76c7509a03baa0133abaaa5819f5d32582b2` | `605bf55cab818ec5` | `a612fc710c0961103aff6f0d078f442214815d8c6a49db2ea163c6bddf3b758e` |
| **Semantic-Dist Installer** | `v1.3.0-beta.1` (`Semantic-Setup-x64.exe`) | — | `9e22d2963994583cf1c1f9bd483fc99794d5c2a40fd3d6f85793f1e8e98bae58` |

---

## 3. Comprehensive Drift Analysis

The candidate `1d1e76c7` is 79 commits ahead of historical C0 (`89641da8`) and 36 commits ahead of the earlier exploratory reference (`94c5c50d`).

```text
Historical C0 (89641da8)
       ↓ +43 commits (SHF-1 text, SHF-3A u32, SHF-2A/2B Bytes, PB01-PB07 hardening)
Exploratory (94c5c50d)
       ↓ +36 commits (Persistent Map, backward liveness, --max-calls, UTF-8 safety, S2043/S2044)
Current Candidate (1d1e76c7)
```

### Detailed Classification of Changes

#### 1. Source-Language & Semantic Changes
- **Single Must-Return Rule (S2043 CP04 / `30f395d3`):** Unifies `check` and `compile`. A function whose `if/else` branches both return is now admitted as complete. (Directly impacts PR #26 test `n25`).
- **ASCII Byte Literals (`50e5197d`):** `b'x'` parsed as exact `u32` literal.
- **UTF-8 Safe Numeric Suffix Scanning (`88129837`):** Prevents panics on non-ASCII characters following numeric literals.
- **Executable `if let` (`48349504`):** Extends executable pattern matching to `if let` expressions.
- **Text Literal Diagnostics Boundary (S2044 / `bae35c24`, `54c5dd00`, `1d1e76c7`):** Strict diagnostic boundaries for oversized text literals and bundle imports.

#### 2. Compiler & Frontend Changes
- **Machine-readable Language Surface (`ce567c24`):** `smc language describe --format json` produces a verified catalog of frontend admission stages.
- **Attributed Bundle Diagnostics (`a49c76c1`, `a6a8c78c`):** Bundle errors attributed to exact source files and relative paths.

#### 3. SemCode Format & Producer Changes
- Header floor: `SEMCOD22` (rev 23).
- Format promotions admitted: `SEMCOD23` (rev 24, plain `u32`), `SEMCOD24` (rev 25, `Bytes`), `SEMCOD25` (rev 26, endian conversions).
- Producer dead-join removal (CP04 / `c2956f84`): Cleans up join points in emitted bytecode.

#### 4. Verifier & Admission Changes
- Retains fail-closed verification of ADT descriptors (`ADT0`), function signatures (`SIG0`), and ownership tables (`OWN0`).
- Strict validation of parameter register counts and definite assignment.

#### 5. VM Runtime & Value Representation Changes
- **Persistent Ordered Map (`29f16050`, `802cc5ab`, `fd43fc23`):** VM runtime uses persistent structural Map storage with deterministic iteration.
- **Register Backward Liveness & Reaching Definitions (`aff35531`, `5254a4c8`, `e21d583b`, `188dec7e`):** Immediate release of registers on last use and CFG-aware record updates.

#### 6. Quota & Resource-Contract Changes
- **Bounded `--max-calls` Override (`743a3209`):** Under `--envelope trusted-compiler`, users may specify `--max-calls <N>` (bounded by effective max steps). Resolves the 16,384 call bottleneck on self-compilation workloads while keeping runtime metering strictly active.

---

## 4. Impact on Existing Pull Requests

| PR | Title / Scope | Effect of Drift | Required Action |
| :--- | :--- | :--- | :--- |
| **#28** | `docs/c0-transition-proposal` | Obsolete (targeted `94c5c50d`) | Replaced with this proposal targeting `1d1e76c7`. |
| **#26** | `feat/semantic-native-compiler` | Test `n25` fails under obsolete return rule; lacks `--max-calls` awareness | Update `n25` to reflect unified must-return rule; verify compiler with candidate `1d1e76c7`. |
| **#27** | `feat/compiler-aggregates` | Empty `OWN0` emission admitted but unproved for borrows; Map uses persistent storage; Calls budget requires override | Preserve stacked dependency on #26; document `OWN0` scope; evaluate `--max-calls` on self-compilation. |
| **#33** | `feat/semantic-vm` | Semantic-written VM lacks persistent Map representation and modern opcodes | Preserve on separate experimental track; document compatibility gaps; do NOT include on C0→C1→C2 critical path. |

---

## 5. Frozen Bootstrap Contract Revision Plan

The SHF-0 bootstrap contract (`bootstrap/contract.toml`) and `docs/BOOTSTRAP_SUBSET.md` are frozen under `shf0-bootstrap-contract-v1`. They cannot be silently edited.

When the repository Owner issues promotion authorization, the transition will occur via a dedicated two-PR governance sequence:

### Step 1: Envelope Transition PR (Governance)
- Switches `.harness/current.task.yaml` to authorize changes to `reference/**` and `bootstrap/**`.
- Authorizes `task.type: governance`.

### Step 2: Contract Revision Payload PR
- Updates protocol identifier to `shf0-bootstrap-contract-v2`.
- Updates `[c0].sha` to `1d1e76c7509a03baa0133abaaa5819f5d32582b2`.
- Moves `BSF-101` (Text inspection) and `BSF-103` (`u32` arithmetic) into the admitted subset.
- Updates `FROZEN_DIGESTS` in `bootstrap/validate_contract.py`.
- Re-seals `docs/BOOTSTRAP_SUBSET.md` and `docs/BOOTSTRAP_CONTRACT.md`.

---

## 6. C0 Candidate Qualification Procedure

Before the Owner executes the transition, Candidate C0 must satisfy:
1. `cargo test --workspace --all-targets` passes on clean checkout of `1d1e76c7`.
2. Positive and negative language surface probes pass against `smc`.
3. Deterministic SemCode emission verified across repeated compilations.
4. Host boundary capability checks pass (`cli-read-only`, `pure` envelopes).
5. Quota enforcement (`--max-calls`, `--max-steps`) functions strictly fail-closed.

**Current Verdict:** `CANDIDATE — NOT YET QUALIFIED AS BOOTSTRAP C0` (Pending formal qualification campaign and Owner promotion gate).
