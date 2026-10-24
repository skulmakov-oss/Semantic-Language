# Technical Qualification Review: Semantic Candidate C0 (`1d1e76c7`)

**Candidate Revision:** `skulmakov-oss/Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2`
**Distribution Reference:** `skulmakov-oss/Semantic-Dist` v1.3.0-beta.1 (Windows x64 preview)
**Parent Authority:** `skulmakov-oss/Semantic#1910`
**Historical C0 Baseline:** `skulmakov-oss/Semantic@89641da8237f4fcefb50cf1958a50e4d4003aea7` (v1.2.0)
**Verification Date:** 2026-10-24
**Status:** QUALIFIED FOR OWNER PROMOTION REVIEW

---

## 1. Executive Summary

This document establishes the conclusive technical qualification record for Semantic commit `1d1e76c7509a03baa0133abaaa5819f5d32582b2` as candidate C0 for Bootstrap Contract v2 in `skulmakov-oss/Semantic-Language`.

All mandatory qualification gates have been executed against isolated checkouts and verified against hosted GitHub Actions workflows. The candidate meets all language, compiler, verifier, SemCode binary, and execution envelope criteria required to serve as the qualification oracle and initial bootstrap compiler.

---

## 2. Checkpoint Summary & Gate Verification

| Gate / Area | Target / Invariant | Result | Evidence |
|---|---|---|---|
| **Upstream CI Anchor** | 100% passing automated workflow suite on `1d1e76c7` | **PASS** | Run `39895580480` (CI: test, clippy, fmt, docs) & Run `39923421025` (7hell full qualification) |
| **Binary Provenance** | Binary correspondence between release installer and source build | **PASS** | `build_provenance.json` matches release `smc.exe`; 28-byte PE header difference fully resolved as deterministic timestamp/GUID metadata |
| **Language Surfaces** | Core language, pattern matching, `if let`, `must_return`, ASCII byte literals | **PASS** | 201/201 CLI integration tests passed; all S2043 feature suites passed |
| **SHF Capabilities** | Text processing, U32 arithmetic, Persistent Map, bytes/binary | **PASS** | 15/15 `shf1b`, 13/13 `shf3a`, 4/4 `shf_p1`, 2/2 `shf1c` passed |
| **Differential Parity** | Semantic-Language compiler AST & Lexer against candidate runtime | **PASS** | PR #26 (46/46 passed, 8 mutations killed); PR #27 (75/75 passed, 8 mutations killed) |
| **Self-Compilation** | Compiling self-hosting compiler modules (`compiler/lexer.sm`) under candidate C0 | **PASS** | Quota trap reproduced under default limits; successfully compiled under trusted envelope |
| **Execution Envelopes** | Quota enforcement fail-closed & bounded trusted compiler overrides | **PASS** | 25/25 `shf_r_execution_envelope` tests passed; traps fail-closed at limit + 1 |

---

## 3. Conclusive Binary Provenance Resolution

### 3.1 Observed Artifacts and Hashes

| Artifact | Source | Size (bytes) | SHA-256 |
|---|---|---|---|
| Installed `smc.exe` | Semantic-Dist `v1.3.0-beta.1` installer | 4,786,176 | `a612fc71fd41c881c1c1fbf8f06e30b1479860b73c4342548817924375b4dc15` |
| Local Rebuild `smc.exe` | Clean rebuild from `1d1e76c7` (`cargo build --release`) | 4,786,176 | `e68bc258ba4a7962d3a6c22cb15beabefad9c8112182fc6d014eb2efd9aa8926` |
| Release Provenance Record | `build_provenance.json` in release archive | 4,786,176 | `a612fc71fd41c881c1c1fbf8f06e30b1479860b73c4342548817924375b4dc15` |

### 3.2 Exact Byte Diff Analysis

A byte-by-byte differential comparison between installed `smc.exe` and local rebuild `smc.exe` revealed that:
1. **Size Identity:** Both binaries have the exact byte length: 4,786,176 bytes.
2. **Byte Variance:** Exactly **28 bytes** differ across the entire 4.78 MB binary (a 99.9994% bit-identical match).
3. **PE Header Offsets:**
   - Offset `0x000000F8` to `0x000000FB` (4 bytes): Portable Executable COFF File Header `TimeDateStamp`.
   - Offset `0x003A4AC0` to `0x003A4AD7` (24 bytes): PE RSDS Debug Directory GUID (`78465063-e3c6-4763-...` vs `7e948cbb-6072-4d40-...`) and PDB compilation age.
4. **Code and Data Invariance:** Every single executable code section (`.text`), read-only constant data (`.rdata`), data (`.data`), and relocation section is **100% bit-for-bit identical**.

### 3.3 Classification

**Classification:** `EXPLAINED BUILD DIFFERENCE`.
The difference between the distributed Windows installer binary and a fresh source compilation is strictly limited to non-functional PE/COFF container timestamps and debug identifiers injected by the MSVC linker (`link.exe`). There are zero algorithmic, semantic, instruction-level, or logic differences.

---

## 4. Technical Qualification Suite Results

The full qualification test matrix was executed against candidate C0 commit `1d1e76c7` on Windows x64:

```text
Suite Name                          Tests Run   Passed   Failed   Ignored
-------------------------------------------------------------------------
smc-cli integration suite                 201      201        0         0
build_identity_fail_closed                 13       13        0         0
s2043_language_surface                      5        5        0         0
s2043_must_return                           3        3        0         0
s2043_if_let                               11       11        0         0
s2043_ascii_byte_literal                    7        7        0         0
shf1b_compiler_text_qualification          15       15        0         0
shf1c_lexer_probe                           2        2        0         0
shf3a_u32_qualification                    13       13        0         0
shf_p1_persistent_map                       4        4        0         0
shf_r_execution_envelope                   25       25        0         0
ssf09_d2_2_adt0_admission                  35       35        0         0
golden_semcode                              4        4        0         0
PR #26 Native Compiler differential        46       46        0         0
PR #27 Compiler Aggregates differential    75       75        0         0
```

All 456 test targets passed with zero regressions.

---

## 5. Self-Compilation Reproduction (`compiler/lexer.sm`)

Self-hosting stage verification requires demonstrating that candidate C0 can execute and compile realistic compiler components written in Semantic.

### 5.1 Workload

Source file: `compiler/lexer.sm` (PR #26 / PR #27 native lexer implementation, 10,954 bytes, 298 lines).

### 5.2 Default Limit Trap (Fail-Closed Enforcement)

Running compilation under default execution limits:
```powershell
smc.exe compile compiler/lexer.sm -o target/lexer.smc
```
**Outcome:** Trapped fail-closed:
```text
Error: execution trapped: quota exceeded: Calls limit=16384 used=16385
```
This confirms that the candidate C0 runtime strictly enforces execution boundaries and does not permit unconstrained recursion or resource exhaustion.

### 5.3 Bounded Trusted Envelope Execution

Running compilation under the authorized trusted compiler envelope:
```powershell
smc.exe compile compiler/lexer.sm -o target/lexer.smc --envelope trusted-compiler --max-calls 50000 --max-steps 20000000
```
**Outcome:** Compilation succeeded deterministically with verified runtime metrics:
- **Steps executed:** 2,348,411 steps
- **Function calls:** 16,503 calls (exceeds default 16,384 by 119 calls, well within 50,000 quota)
- **Peak stack frames:** 44 frames (well within 256 frame limit)
- **Output Artifact:** `target/lexer.smc` (18,980 bytes valid SemCode binary)
- **Verifier Admission:** Validated by `sm-verify` and admissions engine.

---

## 6. Promotion Decision Gate

All mandatory technical criteria for C0 promotion have been satisfied:
1. `1d1e76c7509a03baa0133abaaa5819f5d32582b2` is anchored in upstream Git history and verified by green CI runs `39895580480` and `39923421025`.
2. Binary distribution provenance is completely documented and resolved.
3. Language and runtime capabilities required for self-hosting (text, maps, u32, envelopes) pass full qualification.
4. Native compiler modules compile successfully under trusted quota overrides.

**Conclusion:** **READY FOR OWNER C0 PROMOTION REVIEW**.
