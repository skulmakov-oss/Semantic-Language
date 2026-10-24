# Proposal: C0 Reference Transition to `Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2`

| Attribute | Details |
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
3. Modern must-return completeness rules (historical C0 rejected symmetric `if/else` returns).
4. Configurable Calls quotas for trusted compiler workloads.

Without text inspection (BSF-101) and `u32` arithmetic (BSF-103), writing a self-hosted lexer and parser in Semantic is impossible.

### Important Release vs Oracle Distinction
The publication of `skulmakov-oss/Semantic-Dist@v1.3.0-beta.1` provides a verified Windows x64 distribution package built from upstream commit `1d1e76c7509a03baa0133abaaa5819f5d32582b2`. However, `v1.3.0-beta.1` is a distribution developer preview, **not** a separately qualified upstream bootstrap oracle. Therefore, commit `1d1e76c7509a03baa0133abaaa5819f5d32582b2` is treated as a **Candidate C0** until bootstrap-specific qualification and explicit Owner promotion are complete.

---

## 2. Toolchain and Executable Binary Investigation

### Executable Identity Matrix

| Source / Path | Full Path / Asset Name | File Size (Bytes) | SHA-256 Checksum | `source_fingerprint` | Provenance / Role |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Historical C0** | `cbb435d3...` | 4,763,648 | `cbb435d38d119bbefbc197aee584028e289394a393b51e8bcaac742fb8744eb8` | `7f10f50c82216db2` | Historical qualified C0 (`v1.2.0`) |
| **Historical Exploratory** | `efac7456...` | 4,775,936 | `efac7456ce51feb24f5ac3ef6e2c5b881552b9227171031ca5f73fab50458485` | `1cd4707ec728c07a` | Exploratory baseline (`94c5c50d`) |
| **Published Release Manifest** | `build_provenance.json` | 4,786,176 | `a612fc710c0961103aff6f0d078f442214815d8c6a49db2ea163c6bddf3b758e` | `605bf55cab818ec5` | Build provenance manifest packaged inside installer |
| **Installed User Toolchain** | `%LOCALAPPDATA%\Programs\Semantic\bin\smc.exe` | 4,786,176 | `a612fc710c0961103aff6f0d078f442214815d8c6a49db2ea163c6bddf3b758e` | `605bf55cab818ec5` | Executable installed by `Semantic-Setup-x64.exe` |
| **Clean Local Worktree Rebuild** | `target\release\smc.exe` | 4,786,176 | `e68bc258eeb942cf752bc29cbdaa400e8fc8deef041592e036b5777e9b920652` | `605bf55cab818ec5` | Fresh build from clean checkout of `1d1e76c7` using rustc 1.97.1 |
| **Semantic-Dist Release Notes String** | GitHub Release text | 4,786,176 | `1eef234b6e5e8e89ea2db66a6a2468bb5a6cfec2808c1d564fa74c860c23a7cb` | `605bf55cab818ec5` | Hosted CI Run 40023574147 build artifact |

### Binary Identity Resolution
The investigation into the reported hashes reveals:
1. **Source Identity is Constant:** Every candidate executable reports identical `toolchain_version: 0.1.0`, identical `source_fingerprint: 605bf55cab818ec5`, and identical features (`DEBUG_SYMBOLS`, `PROFILE_LOGOS`, `PROFILE_RUST`, `STD`).
2. **Binary Content & PE Header Differences:** In Windows PE/COFF binaries, MSVC/Rust toolchain linkers embed compilation timestamps, random build UUIDs, and linker debug directories. As a result, two builds from identical git source commits produced at different times or on different machines (e.g. GitHub Actions hosted runner vs local MSVC host) yield different outer SHA-256 hashes despite bit-for-bit behavioral and source equivalence.
3. **Released Manifest Integrity:** The `build_provenance.json` uploaded with the release and embedded in the installer records `a612fc710c0961103aff6f0d078f442214815d8c6a49db2ea163c6bddf3b758e`, which exactly matches the installed `smc.exe` on disk. The hash `1eef234b...` was produced by an earlier intermediate CI run before the final packaging run.

---

## 3. Comprehensive Drift Analysis (79 Upstream Commits)

Candidate `1d1e76c7` is 79 commits ahead of historical C0 (`89641da8`) and 36 commits ahead of exploratory reference (`94c5c50d`).

### Detailed Classification of Changes

#### 1. Source-Language & Semantic Changes
- **Unified Must-Return Rule (S2043 CP04 / commit `30f395d3`):** Unifies `check` and `compile` control-flow analysis. A function whose `if/else` branches both return is now admitted as complete. (Directly incorporated into PR #26 positive case `p16` / PR #27 case `p18`).
- **ASCII Byte Literals (`50e5197d`):** `b'x'` parsed as exact `u32` literal.
- **UTF-8 Safe Numeric Suffix Scanning (`88129837`):** Prevents crashes on non-ASCII characters in numeric lexing.
- **Executable `if let` (`48349504`):** Extends executable pattern matching to `if let` expressions.
- **Diagnostic Attribution (S2044 / `bae35c24`, `54c5dd00`, `1d1e76c7`):** Strict diagnostic boundaries and file-relative source attribution for imported text bundles.

#### 2. Compiler & Frontend Changes
- **Machine-readable Language Surface (`ce567c24`):** `smc language describe --format json` exposes a verified catalog of frontend capabilities and admission stages.

#### 3. SemCode Format & Producer Changes
- Emits `SEMCOD22` (rev 23) by default, promoting to `SEMCOD23` (rev 24) when `u32` operations are present.
- Producer dead-join removal (CP04 / `c2956f84`): Removes redundant join blocks.

#### 4. Verifier & Admission Changes
- Retains fail-closed verification of ADT descriptors (`ADT0`), function signatures (`SIG0`), and ownership tables (`OWN0`).

#### 5. VM Runtime & Value Representation Changes
- **Persistent Ordered Map (`29f16050`, `802cc5ab`, `fd43fc23`):** VM runtime uses persistent structural Map storage with deterministic collision order.
- **Register Backward Liveness (`aff35531`, `5254a4c8`):** Immediate release of registers on last use and CFG-aware record updates.

#### 6. Quota & Resource-Contract Changes
- **Bounded `--max-calls` Override (`743a3209`):** Under `--envelope trusted-compiler`, users may specify `--max-calls <N>` (bounded by effective max steps). Resolves the 16,384 call bottleneck on self-compilation workloads while keeping runtime metering strictly active.

---

## 4. Technical Qualification Evidence & Verification

### Test Suite Execution Summary
Executed from clean candidate checkout on `x86_64-pc-windows-msvc`:

| Suite Name | Target Crate / Test | Cases | Result |
| :--- | :--- | :--- | :--- |
| **smc-cli unit tests** | `crates/smc-cli` | 201 | **201 passed; 0 failed** |
| **Build identity guards** | `tests/build_identity_fail_closed.rs` | 13 | **13 passed; 0 failed** |
| **Language surface** | `tests/s2043_language_surface.rs` | 5 | **5 passed; 0 failed** |
| **Must-return completeness** | `tests/s2043_must_return.rs` | 3 | **3 passed; 0 failed** |
| **Executable if-let** | `tests/s2043_if_let.rs` | 11 | **11 passed; 0 failed** |
| **ASCII byte literals** | `tests/s2043_ascii_byte_literal.rs` | 7 | **7 passed; 0 failed** |
| **Compiler text primitives**| `tests/shf1b_compiler_text_qualification.rs` | 15 | **15 passed; 0 failed** |
| **Lexer probe** | `tests/shf1c_lexer_probe.rs` | 2 | **2 passed; 0 failed** |
| **u32 arithmetic & bounds** | `tests/shf3a_u32_qualification.rs` | 13 | **13 passed; 0 failed** |
| **Persistent Map** | `tests/shf_p1_persistent_map.rs` | 4 | **4 passed; 0 failed** |
| **Execution envelope & quotas**| `tests/shf_r_execution_envelope.rs` | 25 | **25 passed; 0 failed** |
| **ADT0 admission & encoding** | `tests/ssf09_d2_2_adt0_admission.rs` | 35 | **35 passed; 0 failed** |
| **Golden SemCode artifacts**| `tests/golden_semcode.rs` | 4 | **4 passed; 0 failed** |

### Differential Qualification with Native Compiler (PR #26 & #27)
Differential evaluation against candidate `smc.exe`:
- **PR #26 (Native Compiler Core):** **46/46 cases passed; 8 mutations caught; 0 failures.**
- **PR #27 (Aggregates & Extensions):** **75/75 cases passed; 8 mutations caught; 0 failures.**

### Trusted-Compiler Resource & Calls Quota Qualification
1. **Default Envelope:** Quotas enforced at Steps: 100,000; Calls: 16,384; Frames: 256.
2. **Explicit Override:** `--max-calls 50000` successfully elevates user call limits when combined with sufficient steps.
3. **Fail-Closed Protection:**
   - Setting `--max-calls 0` or non-numeric rejected before execution.
   - Setting `--max-calls` greater than effective steps rejected before execution (`--max-calls must not exceed effective max_steps`).
   - Omitting `--envelope trusted-compiler` while passing `--max-calls` rejected (`--max-calls requires --envelope trusted-compiler`).
4. **Self-Compilation Benchmark:**
   - Compiling complete `compiler/lexer.sm` AST workload (10.9 KB) under default Calls limit halts with exact fail-closed trap: `quota exceeded: Calls limit=16384 used=16385`.
   - Compiling under `--max-calls 50000 --max-steps 20000000` succeeds completely: produces 18.6 KB SemCode artifact, executing 2,348,411 steps and 16,503 calls.

---

## 5. Bootstrap Contract v2 Migration Architecture

The SHF-0 bootstrap contract (`bootstrap/contract.toml`) and `docs/BOOTSTRAP_SUBSET.md` remain protected and frozen under `shf0-bootstrap-contract-v1`.

### Required Migration Changes for Contract v2
1. **Contract Identifier:** Bump from `shf0-bootstrap-contract-v1` to `shf0-bootstrap-contract-v2`.
2. **C0 Reference Pin:** Update `[c0].sha` to `1d1e76c7509a03baa0133abaaa5819f5d32582b2`; record distribution provenance `Semantic-Dist v1.3.0-beta.1`.
3. **Admitted Language Capabilities:**
   - Formal admission of `BSF-101` (Text inspection: `text_len`, `text_byte_at`, `text_slice`).
   - Formal admission of `BSF-103` (Unsigned 32-bit integers: `u32` arithmetic, bitwise shifts, ordering).
   - Unified must-return completeness rule (symmetric `if/else` returns admitted).
4. **Binary I/O Capability & Allocations:** Formal admission of binary byte transport and executable file transforms under `cli-file-transform`.
5. **Runtime Quota Envelope:** Formalize bounded `--max-calls` under `trusted-compiler` as the standard self-hosting execution quota.
6. **SemCode Floor:** Formalize `SEMCOD22` (rev 23) as minimum format revision and `SEMCOD23` (rev 24) for `u32` bytecode.
7. **Validator Script:** Update `bootstrap/validate_contract.py` to verify v2 schema, new C0 SHA, and updated frozen doc digests.

### Governance Transition Topology
To respect the immutable trust root in `.github/workflows/harness-trusted.yml`, contract migration will execute as a three-phase transition:
1. **PR 1 (Governance Transition PR):** Switches `.harness/current.task.yaml` to `task.type: governance`, allowing `reference/**` and `bootstrap/**` while forbidding ordinary payload code.
2. **PR 2 (Contract v2 Payload PR):** Lands `bootstrap/contract.toml`, `bootstrap/validate_contract.py`, `reference/semantic-reference.toml`, and sealed documentation updates.
3. **PR 3 (Restore Stable Ordinary Envelope):** Restores `task.type: implementation` with Surface A permissions, permanently relocking governance and bootstrap surfaces.

---

## 6. Formal Promotion Gate & Owner Decision Required

**Current Status:**
- `TECHNICAL QUALIFICATION EVIDENCE — PASS`
- `QUALIFICATION RECORD — FROZEN IN docs/qualification/c0_1d1e76c7_final_review.md`
- `C0 CANDIDATE STATUS — READY FOR OWNER C0 PROMOTION REVIEW`

The candidate reference passes all reference, compiler, differential, and self-compilation benchmarks, and its binary distribution provenance is conclusively resolved. Full technical evidence is recorded in [`docs/qualification/c0_1d1e76c7_final_review.md`](../qualification/c0_1d1e76c7_final_review.md). Under the authority hierarchy of this repository:
- Candidate `1d1e76c7` is technically qualified and positioned for formal C0 adoption.
- No protected surfaces (`bootstrap/**`, `reference/**`, `.github/**`) have been modified in this proposal.
- Promotion requires an explicit, separate **OWNER C0 PROMOTION GO** authorizing the three-step governance transition sequence.
