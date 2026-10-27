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
- A function with a non-`unit` return type must ensure all control flow paths return (`always_returns`). This matches modern candidate Semantic (S2043 CP04 / commit `30f395d3`), which accepts symmetric `if`/`else` blocks where both branches return (`p16`), while rejecting missing branches or paths that fall through (`n25`).
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

The VM meters cumulative **user-function calls**. Bootstrap Contract v2 authorizes the
trusted-compiler path with 20,000,000 Steps, 50,000 Calls and 256 Frames. This does not change
the defaults for ordinary applications or release runtime borrows. Builtins such as
`text_byte_at` and `map_get` are not counted as user calls. The compiler spends few calls per byte:
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

The exact reference is **`skulmakov-oss/Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2`**,
as recorded by `reference/semantic-reference.toml` and frozen Bootstrap Contract v2.
It supports the text inspection and u32 operations used by the compiler implementation.
The old v1.2.0/exploratory-reference observations are historical, not the current pin.

Semantic 1.3.0-beta.1 identifies a Windows x64 Developer Preview distribution; it is not a
replacement for the Git identity. Qualification requires a clean exact-SHA canonical checkout,
the frozen contract, matching build/anchor provenance, binary identity and a source fingerprint
computed using that checkout's build-identity implementation. A supplied binary path, hash or
`version --json` assertion alone is insufficient. Unsigned local build attestation is explicitly
distinguished from distribution provenance and is not cryptographic execution proof or a Seal.
Experimental WSL execution cannot inherit the Windows qualification status.

**`bootstrap/source-set.toml` stays empty and protected.** The compiler's working module graph
is not automatically canonical `S`; no C0(S) / C1(S) fixed-point claim follows from these tests.

### Encoding boundaries

All dynamic function names and string-table entries are checked before encoding any artifact
bytes. Although str16 uses a u16 wire length, C0's decoder limits these strings to **1024 UTF-8
bytes**, and each function's table to **256 entries**. Limits are byte-based, include generated
local-slot suffixes, and preserve the admitted boundary values. Oversized input returns a
deterministic `error unsupported` line, never a partially emitted `hex` artifact or a successful
VM trap. Encoding errors use the existing `@0` marker; precise token attribution is not claimed
for IR-level metadata failures.

## 5. Evidence (tests/compiler)

Run `tests/compiler/run_tests.py` with `SEMANTIC_SMC`, `SEMANTIC_C0_SMC`,
`SEMANTIC_C0_CHECKOUT`, `SEMANTIC_C0_PROVENANCE` and `SEMANTIC_C0_ANCHOR`.
The last two name the distribution build-provenance and upstream-anchor JSON records.
Set `SEMANTIC_TEST_OUT` outside the repository. Identity checks run before any differential
execution or evidence creation. Reusing one attested executable for both roles does not create
independent C0 evidence.
If the host role is an independently linked C0 build, provide its own
`SEMANTIC_SMC_PROVENANCE`; PE byte identity between separate builds is not required.

For every case in `cases.json` it:
1. compiles twice with the Semantic compiler and requires identical output;
2. decodes the result, verifies it with **both** verifiers and runs it on **both** VMs;
3. compiles the same source with the pinned executable roles at O0 and O1, then compares outcomes;
4. checks the expected outcome or diagnostic category and text;
5. checks eight negative artifact mutations and one unchanged positive admission control.

Compiler implementation artifacts are separately compiled and admitted at O0/O1. The pinned
`smc run` application path has fixed source compilation; `run-smc` does not expose application
capabilities or trusted-compiler quota flags. Therefore this runner does not claim execution
of an O1 compiler artifact through that CLI. O0/O1 test-program reference artifacts are executed.
No upstream CLI or admission change is implied.

`run_string_boundaries.py` covers admitted byte boundaries, oversized names, generated local
suffixes and multibyte UTF-8 entries. Accepted artifacts must pass canonical verification and
execution. `test_reference_identity.py` checks early rejection. Native command exits, output,
source/artifact hashes and compiler counters are retained outside the repository.

Historical result on 2026-10-14 (not fresh v2 qualification): **45/45 cases and eight mutation/control rows**:
- 15 positive programs, all executing correctly and matching the C0 compiler's outcome;
- 4 runtime traps (assertion in a callee, `DivisionByZero`, `ArithmeticOverflow`, frame-quota exhaustion from unbounded recursion), matching C0;
- 26 rejected sources with the expected category. C0 also rejects 21 of them; the other 5 are valid Semantic outside the subset.

### Resource measurements

The following table is historical, not a current bound or fresh qualification result. Current
runs use the approved `--envelope trusted-compiler --max-calls 50000 --max-steps 20000000`
with Frames 256. Actual per-case counters and effective quotas belong to each evidence record.

| Measure | Range |
|---|---|
| Source size | 26–484 bytes |
| Steps | 14,235–152,978 (≈ 14k + ≈ 280 per source byte) |
| Calls | 71–1,020 (≈ 2.1 per source byte) |
| Peak frames | 18 |
| Cases within verified-local (100,000 steps) | 41 of 45 |

## 6. What is not demonstrated

- **No C1/C2 fixed-point qualification or Bootstrap Seal.** Supported-subset differential
  evidence and successful compilation of the implementation by C0 do not prove self-hosting.
- **No self-compilation.** The compiler source (≈ 55 KB) uses records, tuples, `Sequence`, `Map`, `Option` match, text and `u32`. The subset doesn't cover these, and they need `MAKE_RECORD`, `TUPLE_*`, `SEQUENCE_*`, `ADT_*`, text and `u32` opcodes, plus non-empty `OWN0` ownership events (the reference emits about 3.8k of them for a compiler of this kind).
- **Self-compilation budgets are not established.** The old per-byte extrapolation is not a
  measurement under v2; no larger quota than the approved trusted-compiler envelope is admitted.
- **The equivalence evidence is behavioural**, on a hand-written corpus: assertion outcomes and traps. Byte identity with C0 is not a goal; register numbering and branch layout differ.
- **Platform:** historical tests were WSL2 experiments; current v2 qualification is explicitly
  platform-bound. C0's qualified platform is `x86_64-pc-windows-msvc`.
