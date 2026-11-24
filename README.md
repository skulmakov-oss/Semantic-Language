<img width="902" height="817" alt="Screenshot 2026-10-09 015304" src="https://github.com/user-attachments/assets/569e469f-7ec1-4c78-a655-9f47061d7169" />

# Semantic Language

> **Bootstrap repository for a compiler written substantially in Semantic.**

This repository has one purpose:

> Produce and qualify a compiler written substantially in Semantic until it reaches the
> **C0 → C1 → C2 self-hosting fixed point**.

## Primary engineering rule — Semantic Capability-First

**Use the full power of Semantic.** The compiler must use every *semantically relevant, actually
admitted* native language capability rather than reproducing a Boolean-only or host-language
architecture by habit. Preserve four-state evidence where the domain requires N/F/T/S;
retain `bool` for genuinely binary facts. This is a design and qualification obligation,
not a demand to use every feature in every function. See
[the architecture invariant](docs/ARCHITECTURE.md#31-semantic-capability-first-invariant)
and [the contribution rule](CONTRIBUTING.md#primary-engineering-rule--semantic-capability-first).

## What this repository is

- The home of compiler source `S` written in Semantic (lexer, parser, semantic analysis, IR,
  lowering, SemCode emission).
- The home of bootstrap qualification: reference manifests, differential harnesses, C1/C2
  fixed-point evidence and the Bootstrap Seal.

## What this repository is not

- Not a second language specification. Language semantics, SemCode, the verifier and the VM
  remain owned by [`skulmakov-oss/Semantic`](https://github.com/skulmakov-oss/Semantic).
- Not a rewrite of the verifier, VM, PROMETHEUS host boundary, native backend, UI, Workbench or
  Studio.
- Not a place to add language/runtime capabilities through bootstrap-only glue.

## Relationship to Semantic

| Repository | Role |
| --- | --- |
| [`Semantic`](https://github.com/skulmakov-oss/Semantic) | Rust reference compiler **C0**, language and SemCode authority, `sm-verify`, `sm-vm`, PROMETHEUS |
| **Semantic-Language** | Compiler source `S` in Semantic and bootstrap qualification |

Upstream architectural authority: [`skulmakov-oss/Semantic#1910`](https://github.com/skulmakov-oss/Semantic/issues/1910).

## What self-hosting means here

```text
S  = source of the compiler, written in Semantic
C0 = qualified Rust-hosted reference compiler (skulmakov-oss/Semantic @ exact SHA)

C0(S) -> C1.smc
C1(S) -> C2.smc

Canonical(C1.smc) == Canonical(C2.smc)      (prefer C1.smc == C2.smc)
```

```text
Semantic (Rust C0) --compile S--> C1 --compile S--> C2
                                   \                /
                                    +-- compare ---+
                                          |
                                     fixed point
                                          |
                                    Bootstrap Seal
```

C1 and C2 are both admitted by `sm-verify`. Only admitted C1 is executed (by `sm-vm`) to compile
`S` into C2; C2 is admitted and compared, not executed. The fixed-point comparison is bootstrap
evidence; it never bypasses admission.

**Rewriting the verifier or VM is not a prerequisite for the first self-hosting fixed point.**

## Bootstrap Seal

The record produced when the fixed point passes: C0 reference identity, source-set identity of
`S`, C1/C2 artifact identities, the frozen comparison rule, verifier/runtime contract identities
used, and the explicit list of remaining non-compiler Rust responsibilities. See
[`docs/BOOTSTRAP_CONTRACT.md`](docs/BOOTSTRAP_CONTRACT.md).

## Roadmap (the only milestone vocabulary)

| Stage | Name |
| --- | --- |
| SHF-0 | Bootstrap Contract |
| SHF-1 | Compiler-grade Text |
| SHF-2 | Bytes / Binary Buffers / Binary I/O |
| SHF-3 | Integer / Index / Bit Contracts |
| SHF-4 | Deterministic Compiler Collections |
| SHF-5 | Executable Generics / Monomorphisation |
| SHF-6 | Arena + Typed IDs |
| SHF-7 | Modules / Exports / Project Resolution |
| SHF-8 | SourceMap + Compiler Diagnostics |
| SHF-9 | Semantic-owned SemCode Model + Canonical Encoder |
| SHF-10 | Lexer |
| SHF-11 | Parser |
| SHF-12 | Semantic Analysis / Typecheck |
| SHF-13 | IR / Lowering |
| SHF-14 | SemCode Emitter |
| SHF-15 | C0 Builds C1 |
| SHF-16 | C1 Builds C2 / Fixed Point |
| SHF-17 | Cross-platform Bootstrap Qualification |

Details: [`docs/ROADMAP.md`](docs/ROADMAP.md). Work that is not on this path:
[`docs/FUTURE.md`](docs/FUTURE.md).

## Current phase

**Bootstrap Contract v4 is in force** (`shf0-bootstrap-contract-v4`, succeeding v3). The Semantic-written
compiler in [`compiler/`](compiler/) includes the integrated Quad-native
main-compatibility checkpoint from [PR #63](https://github.com/skulmakov-oss/Semantic-Language/pull/63).

As of 2026-11-14, on `main` commit
[`e296ada`](https://github.com/skulmakov-oss/Semantic-Language/commit/e296adac229df628108772520cd27e5e32244872),
local qualification established **byte-identical `C2 = C3 = C4`** compiler artifacts
(301,087 bytes each, SHA-256
`120ed3c18d1d8d48cadc3f9597bafec87cb45d143e33c4f026e2a66dcdc4df03`).
Every artifact passed the pinned Frozen C0 verifier. The differential corpus passed
131/131 cases and 9/9 mutations; see the
[exact-HEAD qualification report](https://github.com/skulmakov-oss/Semantic-Language/pull/63#issuecomment-6876851138).

**Self-host engineering integration (SHF-15/16, this tree):** the compiler now emits SemCode
byte-identical to the pinned C0 for every accepted corpus program and for its own source.
`C1 = C0(S)`, `C2 = C1(S)` and `C3 = C2(S)` are byte-identical, so the contract comparison
`C1.smc == C2.smc` (`byte-equality-v1`) is measured as equal. Run it with
[`tests/selfhost/selfhost.py`](tests/selfhost/selfhost.py); see
[`docs/selfhost/README.md`](docs/selfhost/README.md) and [`docs/selfhost/C0_PARITY.md`](docs/selfhost/C0_PARITY.md).
Bootstrap Contract v4 binds the canonical source set (`bootstrap/source-set.toml`), the
stored-artifact executor (`smc run-smc` at an exact upstream revision) and the trusted-compiler
envelope (150M steps / 2M calls / 256 frames). The Seal is **not accepted**: it needs an independent
reproduction (distinct host and operator) and the owner's acceptance (BOOTSTRAP_CONTRACT §10.9).

**Proof boundary:** This C2/C3/C4 engineering fixed-point checkpoint is not an
accepted **C1/C2 Bootstrap Seal** under
[Bootstrap Contract v4](docs/BOOTSTRAP_CONTRACT.md). The protocol additionally requires
its declared source-set and reference bindings, the prescribed C1/C2 comparison,
independent reproduction and Seal acceptance evidence. Formal SHF-16 completion,
SHF-17 cross-platform qualification and full language coverage are not claimed here.

C0 reference pin: `skulmakov-oss/Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2`
(Semantic 1.3.0-beta.1 Windows x64 Developer Preview, with explicit limits) —
[`reference/semantic-reference.toml`](reference/semantic-reference.toml). Floating `main` of the
reference repository is never a qualification oracle.

## Where the verifier and VM remain

`sm-verify` and `sm-vm` remain Rust components in `skulmakov-oss/Semantic`. Each of
C1 and C2 is admitted by that verifier, and C1 is executed by that VM to produce C2. See
[`docs/OWNERSHIP.md`](docs/OWNERSHIP.md).

A separate Semantic-written VM implementation exists in [`vm/`](vm/README.md)
(Issue #29). It is an independent, non-canonical experimental track: its presence
does not transfer verifier or VM authority, and it is not a prerequisite for the
contractual bootstrap proof.

## Documents

| Document | Purpose |
| --- | --- |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Two-repository architecture and boundaries |
| [`docs/BOOTSTRAP_CONTRACT.md`](docs/BOOTSTRAP_CONTRACT.md) | C0/S/C1/C2, fixed point, Bootstrap Seal |
| [`docs/BOOTSTRAP_SUBSET.md`](docs/BOOTSTRAP_SUBSET.md) | Minimum source subset needed to express `S` |
| [`docs/HOST_BOUNDARY.md`](docs/HOST_BOUNDARY.md) | Host mechanics vs compiler meaning |
| [`docs/OWNERSHIP.md`](docs/OWNERSHIP.md) | Who owns each surface now and after bootstrap |
| [`docs/QUALIFICATION.md`](docs/QUALIFICATION.md) | Differential evidence discipline |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | SHF-0 … SHF-17 |
| [`docs/FUTURE.md`](docs/FUTURE.md) | Post-Bootstrap work |
| [`docs/LEGACY_PLAN_MIGRATION.md`](docs/LEGACY_PLAN_MIGRATION.md) | Mapping of earlier plans to SHF |
| [`AGENTS.md`](AGENTS.md) | Operating contract for agents |

## License

Apache License 2.0. See [`LICENSE`](LICENSE).
