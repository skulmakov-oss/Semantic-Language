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

## Current status — Self-Hosting Milestone (2026-11-25)

**Bootstrap Contract v4 is in force** (`shf0-bootstrap-contract-v4`). The compiler written in Semantic
([`compiler/`](compiler/)) reproduces itself byte for byte:

| Item | Value |
| --- | --- |
| Source set `S` | nine `compiler/*.sm` files, `sha256:c9e0cb333ac13fd76989a8db67928d8802e220284f8b5e4ca3b942235ac6148f` |
| `C1 = C0(S)` | `2bfd56e161c40c8f746b6fd6530560f55aa026df29e50a81b99ea5ee3da4e939`, 339,746 bytes |
| `C2 = C1(S)`, `C3 = C2(S)` | byte-identical to C1 (`byte-equality-v1`) |
| C1 compiling `S` | `ok`, 101,176,981 steps / 1,092,148 calls / 36 frames (trusted-compiler envelope 150M / 2M / 256) |
| Differential corpus | 140/140 fixtures; C1 and C2 emit SemCode byte-identical to C0 for every accepted program |
| Normative run | fresh checkout of `8d71cc3` (the tree of `main` without the governance envelope), evidence record valid |

- **Contract bindings:** the canonical source set ([`bootstrap/source-set.toml`](bootstrap/source-set.toml)), the stored-artifact executor (`smc run-smc --expect-sha256` at `skulmakov-oss/Semantic@825deca4`) and the trusted-compiler envelope. Every generation is admitted by `sm-verify` before it runs.
- **Reproduce it:** [`tests/selfhost/selfhost.py`](tests/selfhost/selfhost.py); see [`docs/selfhost/README.md`](docs/selfhost/README.md) and [`docs/selfhost/C0_PARITY.md`](docs/selfhost/C0_PARITY.md).

**Bootstrap Seal: `validated` — not reproduced, not accepted.** Under §10.9 of [Bootstrap Contract v4](docs/BOOTSTRAP_CONTRACT.md), the Seal still needs:
- an independent reproduction on a distinct host by a distinct operator;
- owner acceptance.

All review so far has been automated. Not claimed: SHF-17 cross-platform qualification, full language coverage, or a self-hosted verifier or VM.

**Known issue (upstream):** a VM liveness regression in `skulmakov-oss/Semantic` (`aff35531`) affects C0 and the pinned executor.
- **Effect:** a verified artifact can fail closed at run time after a call without destination.
- **Self-hosting runs:** not affected; every run completed `ok`.
- **Fix:** [skulmakov-oss/Semantic#2087](https://github.com/skulmakov-oss/Semantic/pull/2087).

Change history: [`docs/CHANGELOG.md`](docs/CHANGELOG.md).

C0 reference pin: `skulmakov-oss/Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2`
([`reference/semantic-reference.toml`](reference/semantic-reference.toml)). Floating `main` of the
reference repository is never a qualification oracle.

### History: the 2026-11-14 C2 = C3 = C4 checkpoint (superseded)

On 2026-11-14, before C0 code-generation parity, local qualification on
[`e296ada`](https://github.com/skulmakov-oss/Semantic-Language/commit/e296adac229df628108772520cd27e5e32244872)
showed byte-identical `C2 = C3 = C4` artifacts (301,087 bytes, SHA-256
`120ed3c18d1d8d48cadc3f9597bafec87cb45d143e33c4f026e2a66dcdc4df03`;
[report](https://github.com/skulmakov-oss/Semantic-Language/pull/63#issuecomment-6876851138)).
That was an engineering checkpoint of an earlier source lineage, with `C1 != C2`.
It is kept as history and is not the current status. Its identities are not interchangeable with the v4 identities above.

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
| [`docs/CHANGELOG.md`](docs/CHANGELOG.md) | Dated change history |
| [`docs/selfhost/README.md`](docs/selfhost/README.md) | Self-host workflow and checker |
| [`docs/FUTURE.md`](docs/FUTURE.md) | Post-Bootstrap work |
| [`docs/LEGACY_PLAN_MIGRATION.md`](docs/LEGACY_PLAN_MIGRATION.md) | Mapping of earlier plans to SHF |
| [`AGENTS.md`](AGENTS.md) | Operating contract for agents |

## License

Apache License 2.0. See [`LICENSE`](LICENSE).
