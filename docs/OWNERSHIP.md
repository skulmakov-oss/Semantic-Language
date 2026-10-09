# Ownership

Status: canonical ownership map (SHF reset)

| Surface | Current authority | Bootstrap target |
|---|---|---|
| Language semantics | `Semantic` | remains upstream until explicit transfer |
| Rust C0 compiler | `Semantic` (exact SHA) | reproducible historical executable reference |
| Compiler source `S` | `Semantic-Language` | Semantic |
| Lexer / parser / sema / IR / emitter | `Semantic-Language` implementation | Semantic |
| SemCode contract | `Semantic` (`sm-format` / `sm-emit`) | initially remains upstream |
| Verifier | `Semantic` (`sm-verify`) | remains Rust initially |
| VM | `Semantic` (`sm-vm`) | remains Rust initially |
| PROMETHEUS host boundary | `Semantic` | remains host/runtime |
| Bootstrap qualification | `Semantic-Language` | `Semantic-Language` |

## Rules

1. **One public concept has one owner.** This repository never becomes a competing
   specification of a surface owned upstream.
2. **Capabilities go upstream first.** If `S` needs a language or runtime capability that
   Semantic does not provide (examples: compiler-grade text, `Bytes`, integer/index/bit
   completeness, deterministic collections, executable generics, module semantics, binary I/O),
   the enabler is implemented and qualified in `skulmakov-oss/Semantic` before `S` relies on it.
3. **Compiler logic stays here.** Lexer, parser, AST, binding, typechecking, IR, lowering,
   SemCode construction, compiler diagnostics, differential harnesses, reference manifests and
   C1/C2 evidence belong in this repository.
4. **Transfer is explicit.** A surface moves from "current authority" to "bootstrap target"
   only through a recorded ownership-transfer decision; tests alone never transfer ownership
   (see the `CANONICAL` state in [ARCHITECTURE.md](./ARCHITECTURE.md)).
5. **TypeScript is never a language authority.** Any future tooling client is a presentation
   consumer (see [FUTURE.md](./FUTURE.md)).
6. **Technical dependency != administrative dependency.** The bootstrap implementation depends on
   frozen technical contracts and exact Git SHAs in `skulmakov-oss/Semantic`. It does not depend on
   upstream GitHub issue or PR administrative lifecycles.

## Classifying a change

Ask: *does this change the Semantic language or runtime contract?*

- Yes → it belongs in `skulmakov-oss/Semantic`.
- No, it is ordinary compiler logic → it belongs in `Semantic-Language`.

A single task does not modify both repositories unless explicitly authorized.
