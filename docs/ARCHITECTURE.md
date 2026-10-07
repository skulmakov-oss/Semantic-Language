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

## 8. C0 / C1 / C2 path

1. C0 (reference compiler at the pinned SHA) compiles `S` to `C1.smc`.
2. `C1.smc` is admitted by `sm-verify` and executed by `sm-vm` to compile `S` to `C2.smc`.
3. `C2.smc` is admitted by `sm-verify`.
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
