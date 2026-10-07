# Semantic Language

> **Bootstrap repository for a compiler written substantially in Semantic.**

This repository has one purpose:

> Produce and qualify a compiler written substantially in Semantic until it reaches the
> **C0 → C1 → C2 self-hosting fixed point**.

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

**Architecture established. SHF-0 not started.**

Current reference pin (planning reference, not a qualified oracle):
[`reference/semantic-reference.toml`](reference/semantic-reference.toml).
Floating `main` of the reference repository is never a qualification oracle.

## Where the verifier and VM remain

`sm-verify` and `sm-vm` remain Rust components in `skulmakov-oss/Semantic`. Each of
C1 and C2 is admitted by that verifier, and C1 is executed by that VM to produce C2. See
[`docs/OWNERSHIP.md`](docs/OWNERSHIP.md).

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
