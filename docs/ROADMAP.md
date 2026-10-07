# Self-Hosting Roadmap — SHF-0 … SHF-17

Status: canonical roadmap (SHF reset). **No SHF stage is started.**

This is the only milestone vocabulary of this repository. Stage names and dependencies follow
the upstream authority `skulmakov-oss/Semantic#1910`. A stage is not started merely because it
is documented here; starting a stage requires an explicit owner GO.

## Groups

| Group | Stages | Where the work lands |
|---|---|---|
| FOUNDATION | SHF-0 | Bootstrap contract in this repository |
| FOUNDATION | SHF-1 … SHF-9 | May require changes in `Semantic` and/or both repositories |
| COMPILER | SHF-10 … SHF-14 | Primarily `Semantic-Language` compiler implementation |
| BOOTSTRAP / QUALIFICATION | SHF-15 … SHF-17 | Bootstrap construction and proof |

Group names are labels only; they are not milestones.

## Dependency graph

```text
SHF-0 ──> every stage

SHF-1 Text ─────┐
SHF-2 Bytes ────┼──> SHF-10 Lexer ──┐
SHF-3 Int/Bit ──┤                    │
SHF-4 Collect ──┘                    │
                                     v
SHF-5 Generics ──> SHF-6 Arena ──> SHF-11 Parser <── SHF-7 Modules, SHF-8 Diagnostics
(SHF-3, SHF-4 also -> SHF-6)         │
                                     v
                     SHF-12 Sema (also needs SHF-4, SHF-5)
                                     │
                                     v
                     SHF-13 IR / Lowering (also needs SHF-5, SHF-6)
                                     │
SHF-2, SHF-3 ──> SHF-9 SemCode model │
                        │            v
                        └──> SHF-14 Emitter
                                     │
                                     v
                     SHF-15 C0 builds C1 (also needs SHF-7, SHF-8)
                                     │
                                     v
                     SHF-16 C1 builds C2 / fixed point
                                     │
                                     v
                     SHF-17 Cross-platform qualification
```

Capability gaps found in any stage are fixed upstream first (see [OWNERSHIP.md](./OWNERSHIP.md));
no stage may compensate for a missing language contract with bootstrap-only Rust logic.

---

## SHF-0 — Bootstrap Contract

- **Goal:** freeze what counts as self-hosting before implementation begins.
- **Owner repository:** Semantic-Language (authority: `Semantic#1910`).
- **Dependencies:** none.
- **Deliverable:** executable bootstrap protocol from [BOOTSTRAP_CONTRACT.md](./BOOTSTRAP_CONTRACT.md):
  canonical source tree for `S`, source-set identity, normalization rule, permitted host
  capabilities, deterministic input set, failure taxonomy, qualified reference pin, initial
  [BOOTSTRAP_SUBSET.md](./BOOTSTRAP_SUBSET.md) entries.
- **Qualification:** protocol reviewed against `#1910`; reference pin verified clean at exact SHA.
- **Exit condition:** no implementation can claim self-hosting without satisfying the protocol.
- **Non-goals:** compiler code; capability implementation.

## SHF-1 — Compiler-grade Text

- **Goal:** deterministic inspection of UTF-8 source data sufficient for lexing.
- **Owner repository:** Semantic (language/runtime text contract); Semantic-Language consumes.
- **Dependencies:** SHF-0.
- **Deliverable:** admitted text contract (byte/character inspection, slicing, search) usable by `S`.
- **Qualification:** source → typecheck → IR → SemCode → verifier → VM agreement; boundary and
  invalid-UTF-8 cases.
- **Exit condition:** a Semantic program tokenizes a representative corpus with no host-side tokenization.
- **Non-goals:** locale-aware text processing; Unicode normalization beyond the contract.

## SHF-2 — Bytes / Binary Buffers / Binary I/O

- **Goal:** byte values, binary buffers and bounded binary file I/O.
- **Owner repository:** Semantic (value types and capability); Semantic-Language consumes.
- **Dependencies:** SHF-0.
- **Deliverable:** `Bytes`-class contract and a declared binary I/O capability (see [HOST_BOUNDARY.md](./HOST_BOUNDARY.md)).
- **Qualification:** exact golden binary round trip; bounds and failure cases.
- **Exit condition:** a Semantic program emits an exact golden binary file and reads it back without Rust-side byte assembly.
- **Non-goals:** general filesystem API; networking.

## SHF-3 — Integer / Index / Bit Contracts

- **Goal:** complete deterministic integer, index and bit operations for IDs, offsets, masks and opcodes.
- **Owner repository:** Semantic; Semantic-Language consumes.
- **Dependencies:** SHF-0.
- **Deliverable:** admitted integer/index/bit operation set with defined overflow behavior.
- **Qualification:** identical results across source → typecheck → IR → SemCode → verifier → VM; exhaustive or boundary vectors.
- **Exit condition:** the operations needed by `S` agree across the whole pipeline.
- **Non-goals:** arbitrary-precision arithmetic unless `S` proves it necessary.

## SHF-4 — Deterministic Compiler Collections

- **Goal:** symbol tables, sets, work queues and maps with deterministic traversal.
- **Owner repository:** Semantic; Semantic-Language consumes.
- **Dependencies:** SHF-0.
- **Deliverable:** admitted deterministic `Sequence` / `Map` / `Set` contracts.
- **Qualification:** identical traversal order across runs and platforms.
- **Exit condition:** two compilations over identical inputs traverse all observable map/set structures in the same order.
- **Non-goals:** concurrent collections; hash-order-dependent APIs.

## SHF-5 — Executable Generics / Monomorphisation

- **Goal:** reusable generic data structures and helpers (`Arena(T)`, `Stack(T)`, …) that execute.
- **Owner repository:** Semantic; Semantic-Language consumes.
- **Dependencies:** SHF-0.
- **Deliverable:** executable generics with deterministic monomorphisation.
- **Qualification:** generic structure and helper run through the verified pipeline; negative type cases.
- **Exit condition:** no hidden Rust specialization in the generic path.
- **Non-goals:** trait objects, associated types, higher-kinded types (not automatic blockers).

## SHF-6 — Arena + Typed IDs

- **Goal:** compiler data model using arenas and typed IDs instead of pointer graphs.
- **Owner repository:** both — data-model contract upstream if it needs language support; usage in Semantic-Language.
- **Dependencies:** SHF-0, SHF-3, SHF-4, SHF-5.
- **Deliverable:** arena + typed-ID library usable for AST and IR.
- **Qualification:** build, traverse, transform and emit a non-trivial AST/IR entirely in Semantic.
- **Exit condition:** no GC and no host-owned AST objects needed.
- **Non-goals:** freezing APIs around temporary container limitations.

## SHF-7 — Modules / Exports / Project Resolution

- **Goal:** compiler source split across modules that resolves identically everywhere.
- **Owner repository:** Semantic (module semantics); Semantic-Language consumes.
- **Dependencies:** SHF-0.
- **Deliverable:** admitted module/export/project resolution sufficient for a multi-module `S`.
- **Qualification:** identical resolution and ordering across platforms; negative import cases.
- **Exit condition:** `S` is split across multiple modules and resolves identically on supported platforms.
- **Non-goals:** package registry; versioned dependency management.

## SHF-8 — SourceMap + Compiler Diagnostics

- **Goal:** the compiler owns the meaning of its errors.
- **Owner repository:** both — diagnostic contract upstream; compiler diagnostics in Semantic-Language.
- **Dependencies:** SHF-0.
- **Deliverable:** source map and structured deterministic diagnostics produced by Semantic code.
- **Qualification:** lexer/parser/type errors compared against reference diagnostics where contractual.
- **Exit condition:** no Rust-side compiler diagnosis in the bootstrap path.
- **Non-goals:** IDE presentation; diagnostic rendering beyond the contract.

## SHF-9 — Semantic-owned SemCode Model + Canonical Encoder

- **Goal:** Semantic code builds the actual SemCode artifact for the upstream format contract.
- **Owner repository:** both — format contract upstream (`sm-format`); model and encoder in Semantic-Language.
- **Dependencies:** SHF-0, SHF-2, SHF-3.
- **Deliverable:** SemCode model and canonical encoder written in Semantic.
- **Qualification:** byte-for-byte comparison with C0 output where canonical; artifact admitted by `sm-verify` and executed by `sm-vm`.
- **Exit condition:** a Semantic program emits a SemCode artifact accepted by the existing verifier and VM.
- **Non-goals:** a second SemCode format; verifier changes.

## SHF-10 — Lexer

- **Goal:** Semantic-owned tokenization for the admitted subset.
- **Owner repository:** Semantic-Language.
- **Dependencies:** SHF-0, SHF-1, SHF-2, SHF-3, SHF-4.
- **Deliverable:** lexer in Semantic.
- **Qualification:** token + span identity with C0; malformed input equivalence.
- **Exit condition:** lexer unit `QUALIFIED`.
- **Non-goals:** incremental lexing.

## SHF-11 — Parser

- **Goal:** Semantic-owned AST on arena/typed IDs.
- **Owner repository:** Semantic-Language.
- **Dependencies:** SHF-10, SHF-6, SHF-7, SHF-8.
- **Deliverable:** parser in Semantic.
- **Qualification:** normalized AST equivalence; parse diagnostics where contractual; no grammar widening.
- **Exit condition:** parser unit `QUALIFIED`.
- **Non-goals:** error-recovery features beyond the contract.

## SHF-12 — Semantic Analysis / Typecheck

- **Goal:** Semantic-owned name resolution, binding and type analysis.
- **Owner repository:** Semantic-Language.
- **Dependencies:** SHF-11, SHF-4, SHF-5.
- **Deliverable:** semantic analysis in Semantic.
- **Qualification:** type/binding facts and diagnostics equivalence; negative cases.
- **Exit condition:** sema unit `QUALIFIED`; no Rust semantic-analysis fallback.
- **Non-goals:** query-based incremental analysis.

## SHF-13 — IR / Lowering

- **Goal:** Semantic-owned lowering to the upstream IR contract.
- **Owner repository:** Semantic-Language.
- **Dependencies:** SHF-12, SHF-5, SHF-6.
- **Deliverable:** IR model and lowering in Semantic.
- **Qualification:** canonical normalized IR equivalence; deterministic register/label assignment.
- **Exit condition:** lowering unit `QUALIFIED`.
- **Non-goals:** new optimization passes.

## SHF-14 — SemCode Emitter

- **Goal:** Semantic-owned emission from IR to canonical SemCode.
- **Owner repository:** Semantic-Language.
- **Dependencies:** SHF-13, SHF-9.
- **Deliverable:** emitter in Semantic.
- **Qualification:** byte identity with C0 where canonical; verifier admission.
- **Exit condition:** emitter unit `QUALIFIED`; Rust no longer chooses opcodes/layout for Semantic output.
- **Non-goals:** incremental emission.

## SHF-15 — C0 Builds C1

- **Goal:** the reference compiler compiles `S` into C1.
- **Owner repository:** Semantic-Language.
- **Dependencies:** SHF-14, SHF-7, SHF-8.
- **Deliverable:** `C1.smc` with recorded provenance.
- **Qualification:** C1 admitted by `sm-verify`; C1 compiles the admitted subset with differential checks against C0.
- **Exit condition:** C1 runs on the existing VM with no Rust compiler-domain fallback.
- **Non-goals:** performance targets.

## SHF-16 — C1 Builds C2 / Fixed Point

- **Goal:** prove the fixed point.
- **Owner repository:** Semantic-Language.
- **Dependencies:** SHF-15.
- **Deliverable:** `C2.smc`; comparison under the rule frozen before the run; Bootstrap Seal.
- **Qualification:** `Canonical(C1.smc) == Canonical(C2.smc)` (prefer byte equality); zero unexplained deltas; both admitted.
- **Exit condition:** Bootstrap Seal recorded.
- **Non-goals:** persistent or incremental compiler machinery (see FUTURE.md).

## SHF-17 — Cross-platform Bootstrap Qualification

- **Goal:** prove the bootstrap does not depend on one host.
- **Owner repository:** Semantic-Language.
- **Dependencies:** SHF-16.
- **Deliverable:** fixed-point evidence on representative targets (e.g. Windows x86-64, Linux x86-64, Linux ARM64, macOS ARM64, as CI permits).
- **Qualification:** identical canonical artifacts, diagnostics, module ordering and collection traversal; verifier-first execution on every target.
- **Exit condition:** Seal extended with platform evidence.
- **Non-goals:** packaging; any track listed in FUTURE.md.

---

## Not on the critical path

Verifier rewrite, VM rewrite, Instant Pipeline, SRI, TypeScript tooling, native backend, UI,
Workbench, Studio and `Semantic#1909` are not prerequisites for the fixed point. See
[FUTURE.md](./FUTURE.md).
