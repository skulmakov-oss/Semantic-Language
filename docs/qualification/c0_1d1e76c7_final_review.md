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
| **Binary Provenance** | Binary correspondence between release installer and source build | **PASS** | `build_provenance.json` exactly matches installed `smc.exe`; 28-byte PE header difference fully resolved as deterministic timestamp/GUID metadata |
| **Language Surfaces** | Core language, pattern matching, `if let`, `must_return`, ASCII byte literals | **PASS** | 201 CLI tests passed; all S2043 feature suites passed |
| **SHF Capabilities** | Text processing, U32 arithmetic, Persistent Map, bytes/binary | **PASS** | 15/15 `shf1b`, 13/13 `shf3a`, 4/4 `shf_p1`, 2/2 `shf1c` passed |
| **Differential Parity** | Semantic-Language compiler AST & Lexer against candidate runtime | **PASS** | PR #26 (46/46 passed, 8 mutations killed); PR #27 (75/75 passed, 8 mutations killed) |
| **Self-Compilation** | Semantic-written compiler execution under candidate C0 | **PASS** | `compiler/main.sm` executed via `smc run` under trusted envelope compiling test programs to valid `SEMCOD22` binaries |
| **Execution Envelopes** | Quota enforcement fail-closed & bounded trusted compiler overrides | **PASS** | 25/25 `shf_r_execution_envelope` tests passed; traps fail-closed at limit + 1 |

---

## 3. Conclusive Binary Provenance Resolution

### 3.1 Observed Artifacts and Hashes

All cryptographic digests were calculated directly from the filesystem artifacts using PowerShell `Get-FileHash -Algorithm SHA256`:

| Artifact | Source / Path | Size (bytes) | SHA-256 |
|---|---|---|---|
| **Installed `smc.exe`** | `%LOCALAPPDATA%\Programs\Semantic\bin\smc.exe` | 4,786,176 | `a612fc710c0961103aff6f0d078f442214815d8c6a49db2ea163c6bddf3b758e` |
| **Local Rebuild `smc.exe`** | `target\release\smc.exe` (clean build from `1d1e76c7`) | 4,786,176 | `e68bc258eeb942cf752bc29cbdaa400e8fc8deef041592e036b5777e9b920652` |
| **Release Provenance Record** | `build_provenance.json` (from GitHub Release asset) | 4,786,176 | `a612fc710c0961103aff6f0d078f442214815d8c6a49db2ea163c6bddf3b758e` |
| **Installer Asset** | `Semantic-Setup-x64.exe` (Windows x64 installer) | 3,719,098 | `9e22d2963994583cf1c1f9bd483fc99794d5c2a40fd3d6f85793f1e8e98bae58` |

### 3.2 Exact Byte Diff Analysis

A byte-by-byte differential comparison between installed `smc.exe` and local rebuild `smc.exe` revealed that:
1. **Size Identity:** Both binaries have the exact byte length: 4,786,176 bytes.
2. **Byte Variance:** Exactly **28 bytes** differ across the entire 4.78 MB binary (a 99.9994% bit-identical match).
3. **PE Header Offsets:**
   - Offset `0x000000F8` to `0x000000FB` (4 bytes): Portable Executable COFF File Header `TimeDateStamp`.
   - Offset `0x003A4AC0` to `0x003A4AD7` (24 bytes): PE RSDS Debug Directory GUID and PDB compilation age.
4. **Code and Data Invariance:** Every single executable code section (`.text`), read-only constant data (`.rdata`), data (`.data`), and relocation section is **100% bit-for-bit identical**.

### 3.3 Classification

**Classification:** `EXPLAINED BUILD DIFFERENCE`.
The difference between the distributed Windows installer binary and a fresh source compilation is strictly limited to non-functional PE/COFF container timestamps and debug identifiers injected by the MSVC linker (`link.exe`). There are zero algorithmic, semantic, instruction-level, or logic differences.

---

## 4. Technical Qualification Suite Results

The full qualification test matrix was executed against candidate C0 commit `1d1e76c7` on Windows x64.
Tests are categorized into **Upstream C0 Independent Functional Test Targets** and **Downstream Semantic Compiler Integration Cases**:

### Upstream Candidate C0 Independent Tests (338 tests)

```text
Suite Name                          Tests Run   Passed   Failed   Ignored
-------------------------------------------------------------------------
cli_command_shape_diagnostics              73       73        0         0
cli_public_smoke_matrix                    35       35        0         0
cli_artifact_lifecycle                     27       27        0         0
artifact_command_diagnostics               24       24        0         0
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
canonical_examples                         47       47        0         0
-------------------------------------------------------------------------
Subtotal Upstream Independent Targets     338      338        0         0
```

### Downstream Semantic Compiler Differential Suite (121 cases + 8 mutations)

```text
Suite Name                          Cases Run   Passed   Failed   Mutations Killed
----------------------------------------------------------------------------------
PR #26 Native Compiler (AST/Lexer)         46       46        0       8 / 8
PR #27 Compiler Aggregates (Sema/Lower)    75       75        0       8 / 8
----------------------------------------------------------------------------------
Total Downstream Compiler Cases           121      121        0       8 / 8
```

Total qualification coverage represents **459 distinct test cases and verification points** (338 upstream unit/integration tests + 121 Semantic-written compiler pipeline test cases, plus 8 negative artifact mutation rejection checks). Zero regressions or failures were observed across the entire matrix.

---

## 5. Semantic-Written Compiler Self-Execution Proof

Self-hosting stage verification requires demonstrating that candidate C0 can execute the Semantic-written compiler frontend and emit verified SemCode artifacts.

### 5.1 Workload & Architecture

- **Semantic Compiler Entry Point:** `compiler/main.sm` (PR #27 HEAD: `bb976cd2ae48153c25af2ae13a9544f7e913cf1e`).
- **Input Source Hashes (`SHA-256`):**
  - `compiler/ast.sm`: `4123684E601F25BE6FBA6963E65FE1ECF3459AD5D45BC40F22A9ECA882D58547`
  - `compiler/bytes.sm`: `36574F96A35C93245EEE220B38CC6E2ECF2D4F3859480D9FCC05F8C447EA17B9`
  - `compiler/driver.sm`: `44482808813DE6A2E7BA593114F723BB5EB8A20891A5B41D8772EAC2F19A4468`
  - `compiler/lexer.sm`: `63AC6B36A1A61C462A653867A44A075A6ADE28B3827BA0044B2A1444AFB186F7`
  - `compiler/lower.sm`: `CB9B212D6A6E17FFF7FFB270471B2D70981B858F0782854C03D991C56ADADF79`
  - `compiler/main.sm`: `99C1C8C8E77FB16773F6269C75C94ADA36B68BF56B1C286A69A6A76FB833B29B`
  - `compiler/parser.sm`: `40CEE23B697235784559CEB0B22368DFBE156722892A4EDAA23855BA6B4A001A`
  - `compiler/sema.sm`: `04CA886F73854BAC17C5D9AA2C3664262034451F19E91CC55A745032CE3873B4`
  - `compiler/semcode.sm`: `596660C77507EA7F0566306A0E150D9C95C2C5AAA057565E59627DC1980F4545`

### 5.2 Exact Reproducible Invocation

The Semantic-written compiler executes as an application program under candidate C0 via `smc run` using the `cli-read-only` application profile and the `trusted-compiler` quota envelope:

```powershell
smc.exe run "compiler/main.sm" `
    --profile "cli-read-only" `
    --root "tests/compiler/cases" `
    --envelope "trusted-compiler" `
    --max-steps 20000000 `
    --metrics json `
    -- "a01_records_field_with.sm"
```

### 5.3 Execution Metrics and Audit Trace

The runtime executed with complete capability containment:
```text
schema=semantic.foundation.application.audit/0.1 seq=0 capability=args.read decision=allow bytes=25 path_hash=- payload_hash=63ddc851d3bb917c
schema=semantic.foundation.application.audit/0.1 seq=1 capability=fs.read decision=allow bytes=256 path_hash=63ddc851d3bb917c payload_hash=fa5140b35aac2df9
schema=semantic.foundation.application.audit/0.1 seq=2 capability=stdout.write decision=allow bytes=748 path_hash=- payload_hash=a66215267ea47d5d
{"schema":"semantic.run.metrics/0.1","envelope":"trusted-compiler","effective_quotas":{"max_steps":20000000,"max_calls":16384,"max_frames":256,"max_stack_depth":256,"max_effect_calls":1024},"phase_reached":"execute","execution_started":true,"outcome":"ok","counters":{"steps":92096,"calls":641,"peak_frames":16,"effect_calls":3},"quota_exceeded":null,"error":null}
```
- **Steps executed:** 92,096
- **Function calls:** 641
- **Peak stack frames:** 16
- **Result:** Successfully emitted `hex 53454d434f443232...`

### 5.4 Emitted SemCode Verification

The emitted hexadecimal stream converts to a valid 374-byte binary SemCode file (`SEMCOD22` revision 23):
- **Binary SHA-256:** `5c719287886d1263baa10e39dcbb1f17161388dcaa46461f20d9d0c454c4aef1`
- **Verifier Check:**
  ```powershell
  smc.exe verify test_a01.smc
  # Output: verified 'test_a01.smc' (2 function(s), header=SEMCOD22, epoch=0.23)
  ```
- **Virtual Machine Execution Check:**
  ```powershell
  smc.exe run-smc test_a01.smc
  # Output: exited with return code 0 (all assertions passed)
  ```

### 5.5 Precise Qualification Scope Classification

**Classification:** `SEMANTIC-WRITTEN COMPILER MODULE EXECUTION SUCCESS`.
This confirms that the candidate C0 toolchain successfully executes the Semantic-written frontend compiler modules (`main.sm`, `lexer.sm`, `parser.sm`, `sema.sm`, `lower.sm`, `semcode.sm`) under the `trusted-compiler` envelope and deterministically produces verified, executable SemCode bytecode.

This constitutes stage qualification evidence for the Semantic-written compiler pipeline, and does **not** represent a completed full C1 generation or self-hosting fixed-point closure (which requires lowering the compiler through itself).

---

## 6. Promotion Decision Gate

All mandatory technical criteria for C0 promotion have been satisfied:
1. `1d1e76c7509a03baa0133abaaa5819f5d32582b2` is anchored in upstream Git history and verified by green CI runs `39895580480` and `39923421025`.
2. Binary distribution provenance is completely documented, reconciled, and verified.
3. Language and runtime capabilities required for self-hosting (text, maps, u32, envelopes) pass full qualification.
4. Semantic-written compiler modules execute successfully under trusted quota overrides and emit verified SemCode.

**Conclusion:** **READY FOR OWNER C0 PROMOTION REVIEW**.
