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
          | "[" [ expr {"," expr} ] "]" | block-expr | match-expr | "if" expr block-expr "else" (block-expr | if-expr)
match     = "match" expr "{" { pattern "=>" ( expr | block-expr ) [","] } "}"
pattern   = "_" | INT | INT"u32" | "Option" "::" "Some" "(" NAME ")" | "Option" "::" "None"
block-expr= "{" { stmt } [ expr ] "}"
```

`//` comments are accepted.

Value-position blocks support nested blocks and value-producing conditional tails.
Tuple assignment remains statement-level: it is not admitted inside a value-producing
block by the frozen C0 or its v0 block-body contract. Recovery does not widen that contract.
The ordinary statement form retains its arity, target and type checks.

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
- A function with a non-`unit` return type must ensure all control flow paths return (`always_returns`). The frozen C0 accepts symmetric `if`/`else` blocks where both branches return (`p18`), while rejecting missing branches or paths that fall through (`n25`).
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
- statement-only bare blocks (value-position blocks are supported).

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

The VM meters cumulative **user-function calls**. Bootstrap Contract v2 authorizes the
trusted-compiler path with 20,000,000 Steps, 50,000 Calls and 256 Frames. This does not change
the defaults for ordinary applications or release runtime borrows. Builtins such as
`text_byte_at` and `map_get` are not counted as user calls. The compiler spends few calls per byte:
- the lexer makes no calls per byte;
- the parser accesses tokens directly;
- the encoder writes bytes inline;
- analysis results go into `Map`s.

## 3. Host boundary

The host is `tests/compiler/hex2bin.py` plus the test runner. It does three things only:
1. grants read access to one root and passes a file name;
2. decodes the compiler's lowercase hex output into bytes;
3. runs the existing verifier and VM.

The hex step exists because the C0 host boundary has no binary output (`HOST_BOUNDARY.md` §9, SHF-2 gap). `hex2bin.py` parses, chooses, repairs and adds nothing.

## 4. Reference compatibility

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

## 6. Qualification and remaining limits

Historical aggregate results and Calls=16384 measurements do not qualify the current v2 candidate. Fresh per-case counters are retained externally. The approved trusted-compiler budget is 20M Steps / 50K Calls / 256 Frames.

The subset emits empty OWN0. It admits no ref patterns and does not claim general borrow support; canonical Borrow/Write metadata is required before such support can be enabled. Verifier-first execution remains mandatory.

Map and Import remain compiler self-coverage gaps, not missing upstream C0 capabilities. No canonical source set, C1/C2 fixed point, self-hosting or Seal is established. Platform qualification is Windows MSVC; experimental WSL cannot inherit it.
