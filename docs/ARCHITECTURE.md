# Semantic Bootstrap Architecture

Status: canonical architecture (SHF reset)

## 1. Purpose

`Semantic-Language` exists to produce and qualify a compiler written substantially in Semantic
until it reaches the C0 → C1 → C2 fixed point defined in
[BOOTSTRAP_CONTRACT.md](./BOOTSTRAP_CONTRACT.md).

Upstream architectural authority: `skulmakov-oss/Semantic#1910`.

## 2. Two-repository architecture

```text
              skulmakov-oss/Semantic
              ----------------------
                Rust reference C0
                language authority
                SemCode authority
                sm-verify
                sm-vm
                PROMETHEUS
                     |
                     v
            skulmakov-oss/Semantic-Language
            -------------------------------
             compiler source S in Semantic
                     |
                 C0(S)
                     v
                   C1.smc
                     |
                 C1(S)
                     v
                   C2.smc
                     |
                     v
         Canonical(C1) == Canonical(C2)
                     |
                     v
                Bootstrap Seal
```

The reference is always identified as `skulmakov-oss/Semantic` + exact Git SHA, recorded in
[`reference/semantic-reference.toml`](../reference/semantic-reference.toml). A local checkout
or floating `main` is never the oracle by itself.

## 3. Compiler ownership boundary

`Semantic-Language` owns compiler work expressed in Semantic: lexer, parser, AST, name
resolution, typechecking, IR, lowering, SemCode construction, compiler diagnostics, and the
bootstrap qualification that proves them.

`Semantic` owns the language and runtime contracts the compiler is written in and compiles to.
When `S` needs a capability that Semantic does not provide (compiler-grade text, bytes,
integer/bit completeness, deterministic collections, executable generics, module semantics,
binary I/O), that capability is added upstream first and qualified there. It is never smuggled
into bootstrap-only Rust glue. See [OWNERSHIP.md](./OWNERSHIP.md).

## 3.1. Semantic capability-first invariant

**Primary implementation invariant:** the compiler `S`, and other Semantic-written
components of this repository, MUST fully use all semantically relevant capabilities
that are actually available and admitted in the exact pinned Semantic source/runtime
contract. Language design must drive implementation; familiar limitations of
Boolean-only or host-language compiler patterns must not be imposed on Semantic.

The engineering decision sequence is:

1. Identify the semantic domain and the exact pinned-C0 source-level capabilities it
   admits. Prefer native typed operations, four-state logic, deterministic collections,
   records/ADTs, matching, text/binary operations and other relevant facilities when
   they precisely express that domain. Never infer availability from a Rust API, an
   unmerged experimental PR or a design-only specification.
2. Where information about **one proposition** can be absent, negative, positive or
   contradictory, preserve the four states N/F/T/S and the distinction between truth
   operations and knowledge operations. Do not collapse N into F, S into T, or convert
   evidence into a Boolean authorization decision without an explicit validated policy.
   Evidence, witness identity, provenance authenticity and permission are distinct.
3. Keep truly binary facts binary: token classification, name membership and concrete
   structural checks can correctly use `bool`. Artificially manufacturing negative
   witnesses or mandating `quad` for every predicate violates this rule.
4. Reuse existing admitted Semantic semantics rather than building a second hidden
   implementation in Rust, Python, host callbacks, ad hoc binary flags or private
   format encodings. Missing language/runtime functionality belongs upstream under
   an explicit contract and owner authorization.
5. Preserve deterministic outputs, admission, canonical SemCode, diagnostics, source
   provenance and bootstrap invariants. Improve performance *within* proven semantics,
   not by deleting checks or increasing quotas to conceal an architectural mismatch.

For substantive compiler changes, qualification must record which native capabilities
were used, why they fit their semantic roles, which alternatives were rejected and
what positive, negative, boundary and adversarial tests demonstrate the behavior.
An intentional exception must be justified and reviewed; "the usual Boolean compiler
pattern" is not sufficient justification.

**Scope:** "fully use Semantic" means do not artificially restrict its **applicable**
expressiveness. It does not require every language feature in every function and does
not amend the upstream language specification, active Harness, C0 reference, frozen
Bootstrap Contract, verifier/VM authority or the C1/C2 proof. Experimental quad-native
compiler work and future opaque Evidence designs remain separately qualified.

## 4. Host / runtime boundary

The host performs mechanics; Semantic performs compiler meaning. The host may move declared
bytes, expose declared capabilities and run the existing verifier/VM. It may not tokenize,
parse, bind, typecheck, lower, select opcodes or construct compiler output. See
[HOST_BOUNDARY.md](./HOST_BOUNDARY.md).

## 5. SemCode boundary

The SemCode format contract remains upstream (`sm-format` / `sm-emit` ownership in the
reference). SHF-9 builds a Semantic-owned model and canonical encoder **for that same
contract**; it does not define a second format. Once Semantic claims emission ownership, Rust
must not choose opcode sequences, layout or sections on its behalf.

## 6. Verifier boundary

`sm-verify` remains the Rust admission authority. Every C1 and C2 artifact is admitted through
it. Fixed-point equality does not substitute for admission, and raw (unverified) execution is
never a successful qualification path.

## 7. VM boundary

`sm-vm` remains the Rust execution authority. The first fixed point runs on the existing
admitted runtime. A Semantic VM is not part of the self-hosting critical path.

Four execution concerns are kept distinct:

1. **Rust VM (`sm-vm`)**: the bootstrap host and the reference execution authority, identified
   as `skulmakov-oss/Semantic` + exact SHA. It executes every artifact of the C0 → C1 → C2
   proof and is the behavioural oracle for any other execution engine.
2. **Semantic VM (`vm/**`, SEM-VM-001, Issue #29)**: an independent virtual machine written in
   Semantic, under development. It decodes and executes admitted SemCode with its own engine;
   during bootstrap it runs as a guest program of `sm-vm`. It holds no execution authority:
   its results are differential evidence against `sm-vm`, and admission stays with `sm-verify`.
3. **The C0 → C1 → C2 proof** (§8) is unchanged and remains valid under its frozen protocol
   ([BOOTSTRAP_CONTRACT.md](./BOOTSTRAP_CONTRACT.md)). The Semantic VM is not part of it.
4. **Any future proof involving the Semantic VM** (for example, executing C1 on it) requires a
   separately authorized contract revision. Nothing in `vm/**` changes the Bootstrap Seal
   definition.

## 8. C0 / C1 / C2 path

1. C0 (reference compiler at the pinned SHA) compiles `S` to `C1.smc`.
2. `C1.smc` is admitted by `sm-verify` and executed by `sm-vm` to compile `S` to `C2.smc`.
3. `C2.smc` is admitted by `sm-verify` (it is not executed as part of the proof).
4. `Canonical(C1.smc)` and `Canonical(C2.smc)` are compared under the frozen rule.
5. On success with zero unexplained deltas, a Bootstrap Seal is recorded.

## 9. Migration state model

Each bootstrap unit has exactly one state:

```text
REFERENCE_ONLY
     ↓
MIRRORED
     ↓
DIFFERENTIAL
     ↓
QUALIFIED
     ↓
CANONICAL
```

- **REFERENCE_ONLY** — only the Rust reference is trusted.
- **MIRRORED** — a Semantic implementation exists; equivalence is not proven.
- **DIFFERENTIAL** — both run against a shared corpus; differences are recorded deterministically.
- **QUALIFIED** — the unit satisfies its qualification matrix in
  [QUALIFICATION.md](./QUALIFICATION.md).
- **CANONICAL** — ownership has been explicitly transferred.

## 10. Ownership transfer rule

`CANONICAL` is never reached automatically by tests. It requires an explicit, recorded
ownership-transfer decision by the repository owner. A unit may remain `QUALIFIED` indefinitely
while the Rust reference stays canonical. Rust removal is not a success criterion; proven
Semantic ownership is.

## 11. Non-goals

The first fixed point does not require: rewriting `sm-verify`, `sm-vm`, PROMETHEUS or a native
backend; UI, Workbench or Studio; Instant Pipeline or SRI work; `Semantic#1909` Native
Reasoning. These are listed in [FUTURE.md](./FUTURE.md).
