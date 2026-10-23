# The Semantic compiler (`compiler/`)

Status: **MIRRORED / DIFFERENTIAL for the subset below — not QUALIFIED, not CANONICAL.**

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

```
program   = { record-decl | fn-decl }
record    = "record" NAME "{" field ":" type { "," field ":" type } [","] "}"
fn        = "fn" NAME "(" [ param { "," param } ] ")" [ "->" type ] block
type      = "i32" | "bool" | "u32" | "text" | NAME (record)
          | "Sequence" "(" type ")" | "Option" "(" type ")" | "(" type "," type { "," type } ")"
stmt      = "let" ["mut"] NAME [":" type] "=" expr ";" | "let" ["mut"] "(" NAME {"," NAME} ")" [":" type] "=" expr ";"
          | NAME "=" expr ";" | "(" NAME {"," NAME} ")" "=" expr ";"
          | "if" expr block [ "else" ( if-stmt | block ) ] | "while" expr block
          | "for" NAME "in" expr block | "break" ";" | "continue" ";"
          | "return" [ expr ] ";" | "assert" "(" expr ")" ";" | match-expr [";"] | expr ";"
expr      = binary over  || && == != < <= > >= + - * / %   (left-associative), unary - !
postfix   = primary { "." field | "[" expr "]" | "with" "{" field ":" expr {"," ...} "}" }
primary   = INT | INT"u32" | TEXT | "true" | "false" | NAME | NAME "(" args ")"
          | NAME "{" field ":" expr {"," ...} "}" | "(" expr ")" | "(" expr "," expr {"," expr} ")"
          | "[" [ expr {"," expr} ] "]" | match-expr | "if" expr block-expr "else" (block-expr | if-expr)
match     = "match" expr "{" { pattern "=>" ( expr | block-expr ) [","] } "}"
pattern   = "_" | INT | INT"u32" | "Option" "::" "Some" "(" NAME ")" | "Option" "::" "None"
block-expr= "{" { stmt } [ expr ] "}"
```

`//` comments are accepted.

### Builtins

| Builtin | Lowered as |
|---|---|
| `len(Sequence) -> i32` | `SEQUENCE_LEN` |
| `push(Sequence(T), T)` | `SEQUENCE_PUSH` |
| `pop(Sequence)` | `SEQUENCE_POP` |
| `to_text(i32\|u32\|bool\|text) -> text` | name-dispatched `CALL` |
| `text_len(text) -> u32` | name-dispatched `CALL` |
| `text_byte_at(text, u32) -> Option(u32)` | name-dispatched `CALL` |
| `text_slice(text, u32, u32) -> Option(text)` | name-dispatched `CALL` |

The name-dispatched `CALL`s are exactly how the reference compiler lowers these builtins.

### Static rules

These follow the reference compiler; each one in the second group was checked against it, and the first two were caught by differential tests.

**Rules taken from the reference compiler:**
- A function with a non-`unit` return type must ensure all control flow paths return (`always_returns`). This matches modern candidate Semantic (S2043 CP04 / commit `30f395d3`), which accepts symmetric `if`/`else` blocks where both branches return (`p18`), while rejecting missing branches or paths that fall through (`n25`).
- **`break`/`continue` are allowed only inside `while`**, not inside `for … in` (`n39`).
- Unsuffixed integer patterns match both `i32` and `u32` scrutinees (`a13`).

**Other rules:**
- **Values:**
  - an empty `[]` needs a contextual `Sequence(T)` type;
  - record literals give every field exactly once, and are evaluated in declaration-slot order;
  - equality is defined on `i32`, `u32`, `bool` and `text` only;
  - `u32` and `i32` never mix.
- **`match`:**
  - the scrutinee is `i32`, `u32` or `Option(T)`, and all arms have one type;
  - a `_` arm is required unless `Option::Some` and `Option::None` are both covered.

### Runtime semantics

All runtime behaviour belongs to the VM:
- `i32` wraps;
- `u32` traps on overflow;
- `/` and `%` trap on zero and on `i32::MIN / -1`;
- indexing traps out of bounds;
- `&&` and `||` evaluate both operands, exactly like the reference.

### Outside the subset

These forms are rejected with category `unsupported`:
- enums;
- `Map`, `Result`, `quad`, `f64`, `fx` and `Bytes`;
- `Import`;
- closures and function values;
- method calls;
- ranges and `loop`;
- `const`;
- compound assignment;
- match guards and other patterns;
- record or tuple equality;
- tuple field access;
- field punning;
- default parameters, contracts and expression-bodied functions;
- bare blocks.

## 2. Architecture

| Module | Responsibility |
|---|---|
| `lexer.sm` | UTF-8 bytes to tokens. One loop with no user-function call per byte. Exact `i32` and `u32` literal range checks; text literals stay raw (release limit R2). |
| `ast.sm` | Node arena, `Func`, `Rec`, `Prog`. |
| `parser.sm` | Recursive descent for items and statements, precedence climbing for expressions; postfix field, index and `with`; record literals (`Name { field: …`); `match`; value-producing `if` and blocks. |
| `sema.sm` | Interned type table; record and function tables; scopes; expected-type propagation (for `[]`); typing of builtins; `match` exhaustiveness; loop context; the return rule. The results are side tables (`Map` by node id): types, slots, callees, field indices. Hidden slots hold `if`/`match` results and `for` state. |
| `lower.sm` | AST to a linear IR per function, with symbolic labels. Registers are allocated function-locally and linearly. `match` on `Option` uses `ADT_TAG`/`ADT_GET`, followed by a guard trap (`LOAD_BOOL 0; ASSERT`), as in the reference. |
| `semcode.sm` | IR to SemCode. A first pass maps labels to offsets; a second pass encodes inline. The header is `SEMCOD22` (rev 23), or `SEMCOD23` (rev 24) when `u32` arithmetic or ordering is used, which is those opcodes' minimum revision. It writes the `ADT0` prelude (`Option`, `Result`), the string table (text literals keep their quotes, as the reference stores them), `OWN0`, and `SIG0` families. |
| `bytes.sm`, `driver.sm`, `main.sm` | Byte encoding; `compile_source`; the entry point (`args_read`, `fs_read_text`, `stdout_write`). |

### Design constraint: Calls

The VM meters **user-function calls** with a fixed, cumulative per-run quota of **16,384** in every CLI envelope (measured: `quota_exceeded: Calls`, `used 16385`). Builtins are free. See section 6.

## 3. Host boundary

The host is `tests/compiler/hex2bin.py` plus the test runner. It does three things only:
1. grants read access to one root and passes a file name;
2. decodes the compiler's lowercase hex output into bytes;
3. runs the existing verifier and VM.

The hex step exists because the C0 host boundary has no binary output (`HOST_BOUNDARY.md` §9, SHF-2 gap). `hex2bin.py` parses, chooses, repairs and adds nothing.

## 4. Reference compatibility

**The compiler source cannot be compiled by the pinned C0 (`Semantic@89641da8`, v1.2.0).** At C0:
- `text_len`, `text_byte_at` and `text_slice` do not exist;
- `u32` arithmetic is rejected.

These are registry gaps BSF-101 and BSF-103. No C0 operation decomposes a `text`, so no lexer can exist at C0.

**What the compiler runs on:** it is developed and run on the exploratory reference `Semantic@94c5c50d16c3f2efc7691456755461bedae83b18`. A separate C0 reference-transition proposal covers re-pinning (`docs/proposals/`, its own PR). Nothing here is C0 qualification.

**What C0 judges:** programs compiled by this compiler that use nothing beyond C0 are verified and executed with **C0's own verifier and VM**, and compared with C0's own compilation. The cases that use post-C0 capabilities (text inspection, `u32` arithmetic) are tagged `c0_comparable: false` **by hand** in the corpus. They are judged against the exploratory toolchain only, and C0's outcome is recorded:
- C0's verifier rejects them (`UnknownCallTarget 'text_len'` or `UnsupportedVersion 'SEMCOD23'`);
- C0's compiler rejects their source.

## 5. Evidence (tests/compiler)

Run `tests/compiler/run_tests.py` with `SEMANTIC_SMC` and `SEMANTIC_C0_SMC`. The runner is fail-closed and writes its output outside the tree.

For every case it:
1. compiles twice and requires identical output;
2. decodes the result, verifies it and runs it;
3. compares with the reference compilers;
4. checks the expected category and text;
5. finally checks that 8 mutations of a generated artifact are rejected by both verifiers.

**Result: 74/74 cases and 8/8 mutations.** The cases include:
- 30 positive programs: recursion, mutual recursion, records, nested records and `with`, tuples and destructuring, sequences, `for`, text, `u32`, `match` on `i32`/`u32`/`Option`, value `if`, sequences of records, and a mini-lexer workload;
- 6 runtime traps;
- 38 rejected sources. The reference compiler also rejects 34 of them; the other 4 are valid Semantic outside the subset.

5 cases use post-C0 capabilities.

### Two findings

1. **`OWN0`:** the reference emits ownership *write events* for record construction and for `STORE_VAR` of aggregates. This compiler emits an empty `OWN0`. **Both verifiers (C0 and exploratory) admit the artifacts**, and their behaviour matches the reference on every case. The events support runtime write-path enforcement for borrows, and this subset has no borrows. Generating them is required work before borrows (`ref` patterns) are admitted.
2. **Admission without a capability:** a C0 verifier rejects post-C0 artifacts (unknown builtin call target, newer header). That is the expected and correct fail-closed behaviour.

## 6. Scaling and the Calls budget (measured)

**Small cases (≤ 1.6 KB):** steps grow linearly at about 280 per source byte; calls at about 2 per byte.

**Self-compilation probe:** the compiler's own module `compiler/lexer.sm`, plus a test `main` (10,930 bytes, 2,110 tokens, 1,154 AST nodes):

| Stage | Steps | Calls |
|---|---|---|
| Lexing | 868,967 | 1 |
| Parsing | +342,261 | +6,810 (≈ 5.9 per node) |
| Analysis | +206,329 | +3,330 (≈ 2.9 per node) |
| Lowering + encoding | — | `quota_exceeded: Calls 16385` at 1.71M total steps |

**Calls, not steps, are the binding limit.** Calls scale with AST nodes (≈ 14 per node across stages), because recursive descent and the recursive tree walks pay one call per node.

### Plan, with no runtime change

1. **Per-function compilation units.** The call-free lexer splits a module into items. Each run compiles one function body against the parsed signatures of all items. A Semantic linker run then assembles the artifact. The per-run limit then bounds the largest *function*, not the program.
2. **Iterative traversal** (explicit stacks) in the parser and the tree walks, to cut calls per node.

A quota change is proposed to the owner only if measurements after 1 and 2 still exceed the limit.

## 7. What is not demonstrated

- **C0 qualification of the compiler itself** (section 4).
- **Self-compilation.** The constructs the compiler uses are now nearly all inside the subset; `Map` and `Import` are missing. But the Calls budget stops even one module (section 6).
- **Ownership metadata** (`OWN0` events) and borrows.
- **Byte identity with C0.** Not a goal; equivalence is behavioural, on a hand-written corpus.
- **Other platforms.** Only Linux (WSL2) was used.
