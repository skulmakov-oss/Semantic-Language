# The Semantic compiler (`compiler/`)

Status: **MIRRORED / DIFFERENTIAL for a function-level subset — not QUALIFIED, not CANONICAL.**

`compiler/` is a SemCode compiler written in Semantic. Every compiler-meaning stage runs as Semantic code on the existing verifier and VM:
- tokenizing;
- parsing;
- name resolution;
- type checking;
- lowering to an IR;
- register allocation;
- opcode selection;
- section layout;
- byte encoding.

The host only transports bytes (see "Host boundary").

## 1. Supported source subset

A program is one or more functions:

```
program   = { "fn" IDENT "(" [ param { "," param } ] ")" [ "->" type ] block }
param     = IDENT ":" type
type      = "i32" | "bool"
block     = "{" { stmt } "}"
stmt      = "let" ["mut"] IDENT [":" type] "=" expr ";"
          | IDENT "=" expr ";"
          | "if" expr block [ "else" ( if-stmt | block ) ]
          | "while" expr block
          | "break" ";" | "continue" ";"
          | "return" [ expr ] ";"
          | "assert" "(" expr ")" ";"
          | expr ";"
expr      = binary(1) over  || && == != < <= > >= + - * / %   (left-associative)
unary     = ( "-" | "!" ) unary | primary
primary   = INT | "true" | "false" | IDENT | IDENT "(" [ expr { "," expr } ] ")" | "(" expr ")"
```

`//` comments are accepted.

### Static rules

**Functions and calls:**
- Functions may be called before they are defined, and recursion and mutual recursion are allowed.
- Arguments must match the callee's parameter count and types.
- `main` must exist, take no parameters and return `unit`.

**Types and operators:**
- Types are `i32`, `bool` and `unit` (no return type). A `unit` value can't be an operand, an initializer, an argument or a condition.
- Operators follow Semantic's typing rules.

**Variables:**
- Variables are block-scoped and may be shadowed.
- Assignment must keep the variable's type. This includes parameters and `let` bindings without `mut`, which the reference compiler admits too.

**Returns and loops:**
- A function with a non-`unit` return type must **end with a top-level `return`**. This is the reference compiler's rule: an `if`/`else` whose branches both return is rejected (`n25`).
- `break` and `continue` are only allowed inside a loop.

### Runtime semantics

All runtime behaviour belongs to the VM:
- `i32` arithmetic wraps;
- `/` and `%` trap on zero and on `i32::MIN / -1`;
- `&&` and `||` evaluate both operands, exactly like the reference.

### Outside the subset

These forms are rejected with category `unsupported`:
- other types;
- records, enums and other top-level items;
- text and other literal families;
- `.` `[` `]` `|` `&` `?` syntax;
- `for`, `loop` and `const`;
- compound assignment;
- tuples;
- default parameters, contracts and expression-bodied functions;
- function values;
- bare blocks.

### Diagnostics

Each diagnostic has the form `error <syntax|name|type|context|unsupported> @<byte offset>: <message>`.

## 2. Architecture

| Module | Responsibility |
|---|---|
| `lexer.sm` | UTF-8 bytes to tokens. One loop with no user-function call per byte. |
| `ast.sm` | Node arena, `Func`, `Prog`. |
| `parser.sm` | Recursive descent for items and statements, precedence climbing for expressions. Tokens are read directly from the stream, not through helper calls. |
| `sema.sm` | Function table, scopes, types, arity, the return rule, loop context. The results are side tables (`Map` by node id): expression types, local slots, callees. |
| `lower.sm` | AST to a linear IR per function, with symbolic labels. Registers are allocated function-locally and linearly: parameters in `r0..r(n-1)`, then one fresh register per expression. Locals are VM local slots, one string-table symbol per declaration. |
| `semcode.sm` | IR to SemCode. A first pass maps labels to byte offsets; a second pass encodes instructions inline. |
| `bytes.sm` | Byte encoding through a 256-entry literal table, `i32` little-endian without bit operations, and UTF-8 text bytes. |
| `driver.sm` | `compile_source(text) -> "hex …" \| "error …"`. |
| `main.sm` | Entry point: `args_read(0)` then `fs_read_text`, then `stdout_write`. |

### Output format

The output header is **`SEMCOD22` (revision 23)**, the revision the pinned C0 itself emits. It contains:
- an `ADT0` section with the standard `Option`/`Result` prelude descriptors;
- per function: the string table, an empty `OWN0`, `SIG0` with the parameter families (`i32` = 7, `bool` = 2), and the instructions.

Because `SIG0` is present, the verifier runs its definite-register-assignment proof on every generated function.

### Instructions emitted

`LOAD_BOOL`, `LOAD_I32`, `LOAD_VAR`, `STORE_VAR`, `ADD/SUB/MUL/DIV/MOD_I32`, `CMP_EQ`, `CMP_NE`, `CMP_I32_LT`, `CMP_I32_LE`, `BOOL_AND`, `BOOL_OR`, `BOOL_NOT`, `JMP`, `JMP_IF`, `CALL`, `RET`, `ASSERT`.

### Design constraint: Calls

The VM meters **user-function calls** with a fixed, cumulative per-run quota of 16,384 in every CLI envelope. Builtins such as `text_byte_at` and `map_get` are not counted. The compiler is therefore written to spend few calls per source byte:
- the lexer makes no calls per byte;
- the parser accesses tokens directly;
- the encoder writes bytes inline;
- analysis results go into `Map`s.

## 3. Host boundary

The host is `tests/compiler/hex2bin.py` plus the test runner. It only does three things:
1. passes a file name and grants read access to one root;
2. decodes the compiler's lowercase hex output into bytes;
3. runs the existing verifier and VM.

The hex step exists because the C0 host boundary has no binary output (`HOST_BOUNDARY.md` §9, SHF-2 gap). `hex2bin.py` rejects anything that is not even-length lowercase hex, and it doesn't parse, choose, repair or add any byte.

## 4. Reference compatibility (important)

**The compiler source cannot be compiled by the pinned C0 (`Semantic@89641da8`, v1.2.0).** A lexer needs byte access to source text. At C0:
- `text_len`, `text_byte_at` and `text_slice` do not exist (`unknown function`);
- `u32` arithmetic and comparison are not admitted.

These are exactly the registry gaps BSF-101 (text inspection) and BSF-103 (`u32`) in `BOOTSTRAP_SUBSET.md`. No C0 operation decomposes a `text` value, so any lexer, and therefore any self-hosted compiler, requires a C0 that includes SHF-1 and SHF-3. This was verified with a C0 build (`smc-cli` SHA-256 `cbb435d3…`, `source_fingerprint` `7f10f50c82216db2`) against minimal probes.

Consequences, with the manifest and the bootstrap contract left unchanged:
- **The compiler is developed and run on an exploratory reference**, `Semantic@94c5c50d16c3f2efc7691456755461bedae83b18` (`source_fingerprint` `1cd4707ec728c07a`). Results from it are **not C0 qualification**.
- **The programs it compiles** use nothing beyond C0. Their artifacts are verified and executed with **the pinned C0's own verifier and VM**, and compared with C0's compiler output. That part is real C0 differential evidence.
- **`bootstrap/source-set.toml` stays empty.** Registering `compiler/` there is a protected (`bootstrap/**`) contract change.
- **The required owner decision:** re-pin C0 to a qualified reference that includes SHF-1B (text inspection) and SHF-3A (`u32`), with the drift analysis the contract requires, and admit BSF-101/BSF-103.

## 5. Evidence (tests/compiler)

Run `tests/compiler/run_tests.py` with `SEMANTIC_SMC` (the exploratory `smc-cli`) and `SEMANTIC_C0_SMC` (the pinned C0 `smc-cli`). The runner is fail-closed.

For every case in `cases.json` it:
1. compiles twice with the Semantic compiler and requires identical output;
2. decodes the result, verifies it with **both** verifiers and runs it on **both** VMs;
3. compiles the same source with C0 and with the exploratory compiler, then compares outcomes;
4. checks the expected outcome or diagnostic category and text;
5. finally checks that 8 mutations of a generated artifact are rejected by both verifiers.

Result on 2026-10-14: **45/45 cases and 8/8 mutations**:
- 15 positive programs, all executing correctly and matching the C0 compiler's outcome;
- 4 runtime traps (assertion in a callee, `DivisionByZero`, `ArithmeticOverflow`, frame-quota exhaustion from unbounded recursion), matching C0;
- 26 rejected sources with the expected category. C0 also rejects 21 of them; the other 5 are valid Semantic outside the subset.

### Resource measurements

The compiler runs under `--envelope trusted-compiler`. Calls (16,384) and Frames (256) are unchanged; only the Steps budget is raised.

| Measure | Range |
|---|---|
| Source size | 26–484 bytes |
| Steps | 14,235–152,978 (≈ 14k + ≈ 280 per source byte) |
| Calls | 71–1,020 (≈ 2.1 per source byte) |
| Peak frames | 18 |
| Cases within verified-local (100,000 steps) | 41 of 45 |

## 6. What is not demonstrated

- **No C0 qualification of the compiler itself** (section 4).
- **No self-compilation.** The compiler source (≈ 55 KB) uses records, tuples, `Sequence`, `Map`, `Option` match, text and `u32`. The subset doesn't cover these, and they need `MAKE_RECORD`, `TUPLE_*`, `SEQUENCE_*`, `ADT_*`, text and `u32` opcodes, plus non-empty `OWN0` ownership events (the reference emits about 3.8k of them for a compiler of this kind).
- **Self-compilation would also exceed the Calls quota** at the measured ≈ 2.1 calls per byte: about 115k calls for 55 KB against 16,384. Either the per-byte call rate must fall by about 7×, or the quota model needs an owner decision.
- **The equivalence evidence is behavioural**, on a hand-written corpus: assertion outcomes and traps. Byte identity with C0 is not a goal; register numbering and branch layout differ.
- **Platform:** Linux (WSL2) only. C0's qualified platform is `x86_64-pc-windows-msvc`.
