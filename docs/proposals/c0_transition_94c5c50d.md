# Proposal: C0 reference transition to `Semantic@94c5c50d`

| | |
|---|---|
| Status | **PROPOSAL — the candidate is NOT qualified.** Nothing in `reference/**` or `bootstrap/**` is changed by this document. |
| Current C0 | `skulmakov-oss/Semantic@89641da8237f4fcefb50cf1958a50e4d4003aea7` (`v1.2.0`) |
| Candidate | `skulmakov-oss/Semantic@94c5c50d16c3f2efc7691456755461bedae83b18` (upstream `main`, untagged) |
| Decision owner | Repository owner |

## 1. Why a transition is needed (evidence)

A compiler written in Semantic needs to read its source byte by byte. At the current C0, that is impossible:

| Probe (one-line program) | C0 `89641da8` | Candidate `94c5c50d` |
|---|---|---|
| `text_len("abc")` | `unknown function 'text_len'` | ok |
| `text_byte_at("abc", 0u32)` | `unknown function 'text_byte_at'` | ok |
| `text_slice("abc", 0u32, 1u32)` | `unknown function 'text_slice'` | ok |
| `1u32 + 2u32` | rejected (`f64 arithmetic requires f64 operands, got U32 and U32`) | ok |
| `1u32 < 2u32` | rejected (`relational operators are currently admitted only for same-family i32 operands …`) | ok |

C0 has no operation that decomposes a `text` value, so no lexer, and therefore no `S`, can be compiled by C0. These are registry candidates BSF-101 (text inspection) and BSF-103 (`u32` arithmetic). Under `BOOTSTRAP_CONTRACT.md` §8 this is `CAPABILITY_GAP`: the capability is to be added upstream first. Upstream has done that in SHF-1 (#2003, #2005, #2011) and SHF-3A (#2007, #2009), both after `v1.2.0`.

**Toolchain identities used for the probes**, both built from clean checkouts in separate target directories:

| Build | `smc-cli` SHA-256 | `source_fingerprint` |
|---|---|---|
| C0 | `cbb435d38d119bbefbc197aee584028e289394a393b51e8bcaac742fb8744eb8` | `7f10f50c82216db2` |
| Candidate | `efac7456ce51feb24f5ac3ef6e2c5b881552b9227171031ca5f73fab50458485` | `1cd4707ec728c07a` |

## 2. Drift analysis (`89641da8..94c5c50d`)

**Relationship:**
- `89641da8` is an ancestor of the candidate; the candidate is 43 commits ahead.
- The diff touches 39 files under `crates/`, with 9,760 insertions and 2,522 deletions.
- The candidate's tree is `2439e000a78358eb036206173d6d956799268f4c`, identical to the tree of `Semantic#2039` HEAD `c2fca6fa`.

**Compiler-relevant commits, grouped:**

| Group | Commits | Effect on bootstrap |
|---|---|---|
| Hardening after v1.2.0 | `10d1eafa` … `37f98844` (#1985–#1992: PB-01 … PB-07), `1065f3e9`, `bc9996d1` | Frontend admission, builtin namespace, type analysis, module graph identity, IR and SemCode producer integrity, runtime admission freshness, PROMETHEUS boundaries. **They change compiler and admission behaviour.** |
| SHF-1 compiler text | `5256cc07`, `637c3ad2`, `5193e90d` | Adds `text_len`, `text_byte_at`, `text_slice`, `text_starts_with`, `text_ends_with`, `text_find` and `text_is_empty`. **Required by S (BSF-101).** |
| SHF-3A `u32` | `0f373890`, `50654be8` | `u32` arithmetic and ordering; new header `SEMCOD23` (rev 24). **Required by S (BSF-103).** |
| SHF-2A/2B Bytes and endian | `16737de9`, `3000a943`, `35f0c21a`, `1d8568b6` | `Bytes` value family (header `SEMCOD24`) and endian helpers. Not required by S today. **Binary file output is still deferred** (SHF-2C, `docs/spec/compiler_bytes_v0.md`), so `artifact_write` stays a gap. |
| SHF-R | `d05b9c1b` | `--envelope trusted-compiler` with a raisable Steps limit, plus run metrics. Calls (16,384) and Frames (256) stay fixed. |
| VM performance | `0373bec5`, `e75cc9b7` … `94c5c50d` | Zero-copy dispatch, copy-on-write sequences, verified consumption. They change runtime cost; the semantics are unchanged. |
| Governance and docs only | `8ebd3294`, `d95bee48`, `ce1cbd9f`, `7d3020d4`, `242c684d`, `f0b168f7`, `5f3e2302`, `61faff23`, `42bcd66a`, `e38c5091`, `e11a694d`, `db86eb78`, `d4ef0c0b`, `327c0b6d` | None |

**Bootstrap evidence invalidated by the transition:**
- **Nothing that is FROZEN.** The `[subset]` frozen list is empty, `S` is empty, and no C1, C2 or Seal exists.
- **The compiler-branch evidence** (PR #26/#27) was produced *against the candidate*. Under the current contract it is labelled exploratory. After a transition it must be re-run, bound to the new protocol identifier, as the contract requires.

## 3. Qualification evidence collected so far

| Evidence | Result |
|---|---|
| Upstream CI for exactly `94c5c50d` (GitHub check runs) | **14/14 success**, including `full-qualification`, `pcc-qualification-7hell`, `test-std`, `public-api-guard`, `runtime-release-gates`, `boundary-enforcement`, `check-no-std` and `release-bundle-process`. |
| `cargo test --workspace --all-targets`, Linux (Ubuntu 24.04, WSL2), clean worktree at the exact SHA, separate target directory | **332 suites, all ok, exit 0**; tree clean before and after. |
| `cargo test --workspace --all-targets`, **x86_64-pc-windows-msvc** (C0's qualified platform; rustc 1.97.1), clean worktree at the exact SHA (tree `2439e000…`), separate target directory | **332 suites, all ok, exit 0**; tree clean before and after. |
| Output compatibility on the compiler corpus (`tests/compiler`, 74 cases): compile with C0 and with the candidate | For all **35** programs both compilers accept, the artifacts are **byte-identical**. **5** programs (text inspection or `u32`) are accepted only by the candidate. No case accepted by C0 is rejected by the candidate. |
| Determinism of `smc compile` | Covered by the upstream test `tests/cli_artifact_lifecycle.rs`, which is part of the workspace suites above. |

**What this evidence does not establish:**
- **Release status.** Upstream still says *"current main is development beyond C1 and is not qualified as any new release"*. The candidate has no release tag and no release qualification campaign comparable to `Semantic#1983` for v1.2.0.
- **The candidate's own release limits.** The v1.2.0 limits R1–R4 were qualified for v1.2.0, and their status at the candidate has not been re-qualified.

**Therefore the candidate is not "qualified-reference".** The owner must choose one of these paths:
- **A (preferred).** Upstream qualifies and tags a release at, or descending from, the candidate. Re-pin to that tag's SHA with the same `[qualification]` fields v1.2.0 has.
- **B.** The owner explicitly accepts a Semantic-Language-side qualification record for the candidate: the evidence above, the completed Windows run, and anything else the owner requires. Record it as a new `status` value (for example `owner-qualified-reference`) that the validator must accept explicitly. This is a contract change, not a relabelling.

## 4. Required contract changes (proposed text; not applied)

All of these are protected paths (`reference/**`, `bootstrap/**`). Under `AGENTS.md` §8 they need, in order:
1. an owner-authorized **envelope-only PR** that allows exactly these paths;
2. the payload PR.

They cannot be part of this PR or of the compiler PRs.

### 4.1 `reference/semantic-reference.toml`

```toml
repository = "skulmakov-oss/Semantic"
sha = "<qualified SHA: 94c5c50d16c3f2efc7691456755461bedae83b18 or the tag chosen under path A>"
status = "qualified-reference"          # or the path-B status chosen by the owner
audited = "<date of the transition PR>"

[qualification]
release_tag = "<tag under path A, or none>"
verdict = "<upstream verdict, or the owner decision under path B>"
qualification_campaign = "<upstream issue/PR, or this proposal + the evidence record>"
platform_scope = "x86_64-pc-windows-msvc"   # unchanged unless Linux evidence is promoted explicitly

[qualification.limits]
# Re-state R1-R4 as they hold at the new SHA (re-verified, not copied).

[drift]
previous_c0 = "89641da8237f4fcefb50cf1958a50e4d4003aea7"
commits_from_previous_c0 = 43
compiler_relevant = true
analysis = "docs/proposals/c0_transition_94c5c50d.md"
```

### 4.2 `bootstrap/contract.toml`

- **Protocol:** `protocol = "shf0-bootstrap-contract-v2"`.
- **C0 pin:** `[c0].sha` takes the new SHA.
- **Evidence contracts:** `[evidence].verifier_contract` and `runtime_contract` take the new SHA.
- **Subset:**
  - `[subset].admitted` gains `BSF-101` and `BSF-103`;
  - `candidate` keeps `BSF-102`, `BSF-104`, `BSF-105` and `BSF-106`;
  - `authority` points at the profile **plus** `docs/spec/compiler_text_v0.md` and `docs/spec/compiler_u32_v0.md` at the new SHA.
- **Binary output:** `[capabilities.artifact_write]` stays `available_at_c0 = false`. Binary file output is still deferred upstream (SHF-2C), so the first Seal needs either SHF-2C or an explicitly frozen transport-only hex adapter (see §5.3).

### 4.3 `bootstrap/validate_contract.py`

- **Protocol identifier:** `PROTOCOL` becomes `shf0-bootstrap-contract-v2`.
- **Frozen digest:** add `FROZEN_DIGESTS["shf0-bootstrap-contract-v2"]`, computed from the new frozen values. Keep the v1 entry so historical v1 records stay checkable.
- **Status (path B only):** accept the new reference `status`.

### 4.4 `docs/BOOTSTRAP_SUBSET.md` and `docs/BOOTSTRAP_CONTRACT.md` §10.1

- Move BSF-101 and BSF-103 into the admitted table, with the authority SHA, required compiler use, evidence forms and stages that §4 requires.
- Update the §10.1 C0 paragraph.

Both files are inside the frozen digest, so they change together with the contract.

## 5. Plan: registering the canonical source set S

### 5.1 When

Only **after** the transition is merged. Registering `compiler/` under the v1 contract would bind `S` to a C0 that can't compile it, which is `CAPABILITY_GAP` at once.

### 5.2 What

**The proposed `S`:** exactly the compiler modules (`compiler/*.sm`), listed explicitly in UTF-8 byte order as the protocol requires. At PR #27 head `67a7c67`, that is:

```toml
protocol = "shf0-source-set-v1"
root = "compiler"
files = [
    "compiler/ast.sm", "compiler/bytes.sm", "compiler/driver.sm", "compiler/lexer.sm",
    "compiler/lower.sm", "compiler/main.sm", "compiler/parser.sm", "compiler/sema.sm",
    "compiler/semcode.sm",
]
```

**Validation:** these paths pass the repository's own `check_paths` and `check_content`, with no errors. The identity computed by `bootstrap/validate_contract.py::source_set_identity` is `sha256:6cf0401807619912a12e8682b943f8b6e57c1856a653a3477b417da3e134bc94`. It changes with every compiler commit, which is expected; it is fixed at registration time.

### 5.3 Prerequisites before registration is useful for C1/C2

1. **The compiler interface.** `[compiler]` states `Compile(source_set, …)` over a *source set*, while today's compiler reads one file. It needs either multi-file input inside Semantic (`Import`, or explicit module concatenation done in Semantic, never by the host), or a contract rule that `S` has one entry module.
2. **The artifact transport.** The compiler emits lowercase hex because no binary write exists, even at the candidate. Either upstream SHF-2C (binary output) lands, or the owner freezes a transport-only hex decoder as a declared host capability under `HOST_BOUNDARY.md` §8, with positive, denied, malformed and boundary tests.
3. **The Calls budget.** Measured: compiling the compiler's own `lexer.sm` with the compiler stops at `quota_exceeded: Calls 16385` (1.71M steps; see `docs/COMPILER.md` §6). The C1(S) step can't succeed until the per-function compilation-unit design (or another call-reducing change) is in place, or the owner decides on the quota.

## 6. Recommended sequence

1. ~~Finish the Windows-msvc run and record it in §3.~~ Done: 332 suites ok.
2. Owner decision: path A or B.
3. Envelope-only PR allowing `reference/**`, `bootstrap/**` and the two `docs/BOOTSTRAP_*.md` files.
4. Transition payload PR (§4), re-validated with `validate_contract.py`.
5. Re-run the compiler qualification bound to protocol v2.
6. Source-set registration (§5) once 5.3(1) is settled.
