# Semantic VM (`vm/**`)

Status: **MIRRORED / DIFFERENTIAL** for the subset below. Issue #29 (SEM-VM-001). No execution
authority: `sm-vm` stays the bootstrap host and reference execution authority, and admission
stays with `sm-verify` (see [ARCHITECTURE.md](./ARCHITECTURE.md) §7).

A virtual machine written in Semantic. It reads a real SemCode artifact, decodes it with its
own decoder, and executes the instructions with its own engine. During bootstrap the VM is
itself an ordinary Semantic program run by the reference toolchain:

```text
host OS -> sm-vm (reference, Rust) -> vm/main.sm (Semantic VM) -> guest SemCode
```

The reference VM runs the outer program only. It never executes a guest instruction on the
Semantic VM's behalf, and the host performs byte transport only (§3).

## 1. Status vocabulary

| Status | Meaning here |
|---|---|
| IMPLEMENTED | code exists in `vm/**` |
| VERIFIED | identical to the oracle on the corpus (status, exact message, every print, guest Steps/Calls/Frames) |
| PARTIALLY VERIFIED | identical on the observables the oracle CLI exposes; the rest is recorded, not compared |
| UNSUPPORTED | decodes correctly, then fails explicitly with `unsupported: ...` if executed; never skipped or approximated |
| NOT TESTED | implemented, but no admitted artifact in the corpus exercises it |

## 2. Reference identity

| Role | Identity |
|---|---|
| Oracle (compiler, verifier, VM) | `skulmakov-oss/Semantic@94c5c50d16c3f2efc7691456755461bedae83b18`, `smc-cli` SHA-256 `efac7456ce51feb24f5ac3ef6e2c5b881552b9227171031ca5f73fab50458485` (clean release build, byte-identical to the exploratory build used for the compiler) |
| C0 (recorded, not judged) | `89641da8`, `smc-cli` SHA-256 `cbb435d3…` |
| Host route for the Semantic VM | the oracle's `smc run vm/main.sm --profile cli-read-only --envelope trusted-compiler --max-steps 200000000` |

The Semantic VM needs `text_len` / `text_byte_at` / `text_slice` and `u32` arithmetic (it runs
as a Semantic program), so it runs on the oracle build, not on C0. That is the same capability
gap that blocks a lexer on C0 (C0 transition proposal, PR #28). Results are therefore attributed to
`94c5c50d`, never to the frozen C0.

## 3. Pipeline and host boundary

```text
guest .smc --verify (sm-verify, oracle)--> admitted
           --bytes.hex() (transport)----> case.hex  (lowercase hex, optional trailing newline)
           --vm/loader.sm hex_decode----> bytes
           --vm/loader.sm decode--------> Prog (validated internal program)
           --vm/engine.sm run-----------> one report line on stdout
```

- **Admission is delegated, explicitly.** The harness admits every guest artifact with the oracle
  verifier before comparing. The Semantic VM's decoder is a structural check, not a verifier,
  and no claim of independent admission security is made.
- **Transport** is `bytes.hex()` in the harness and `hex_decode` in Semantic. The host never
  decodes an instruction, resolves a branch or interprets the guest.
- **Report** is one line with no newline (Semantic text literals have no escapes, so a VM written
  in Semantic cannot produce one):
  `SEMVM1 ` followed by length-prefixed fields `<len>:<status><len>:<metrics><len>:<n>` and
  `n` print fields. `len` is the UTF-8 byte count. The status is one of:
  - `ok`
  - `error <message>`: the reference message, character for character
  - `error unsupported: <what>`
  - `reject <load|transport>: <reason>`
- **No ambient authority.** The VM process holds `cli-read-only` capabilities (args, read the
  root, stdout). The guest receives none of them:
  - every application effect (`args_read`, `stdin_read_text`, `stdout_write`, `stderr_write`,
    `path_inspect`, `fs_read_text`, `fs_write_text`) fails exactly as on the reference's
    host-less route;
  - `print` is a VM-owned observation, never a host write.

## 4. Architecture

Five modules form a single import chain:

| Module | Role |
|---|---|
| `value.sm` | value and heap representation, aggregate equality |
| `program.sm` | executable program representation, builtin table |
| `loader.sm` | transport and SemCode decoding |
| `engine.sm` | execution |
| `main.sm` | entry point |

### 4.1 Program representation (decode once)

The wire format is decoded once. The engine never sees artifact bytes.

- **Instructions:** one global array of `Ins { op, a..f }` for all functions. `op` is a dense
  code grouped by operand shape (`program.sm` lists the encoding); opcode bytes are not reused.
- **Branches:** targets are pre-resolved to instruction indexes. A target that is not an
  instruction boundary is rejected at load.
- **Calls:** targets are pre-resolved to a function index or a builtin code. User functions shadow
  builtins, in the same order as the reference.
- **Locals:** `LOAD_VAR` / `STORE_VAR` name a frame slot, which is the first string-table entry
  with the same text. This is the reference's symbol identity, with no runtime string lookups.
- **Pools:** operand lists, texts (literals already unquoted) and `u32` immediates live in pools.

### 4.2 Values and memory

- **A value is one flat record** `V { k, i, u, t, h }`. `k` is the SIG0 family tag; `k = 0` marks
  an undefined register, which is a distinct error and never `Unit`. Scalars are inline.
- **Aggregates live in an append-only heap.** Sequences, tuples, records and ADTs are immutable
  `Obj`s in a `Sequence(Obj)`, so guest value semantics hold without reference counting; an
  update (`SEQUENCE_PUSH`, …) creates a new object.
- **Cost of the heap:** there is no reclamation. Heap growth is bounded by guest Steps (at most
  one object per instruction) and is reported as `heap_objects` (insertion sort of 60 numbers:
  1,891 objects).
- **Copying:** pushing onto an n-item sequence copies n items. That is natively fast but
  O(n²) memory for a loop of n pushes. Uniqueness-based in-place update, as the reference has
  since SHF-P3, is a measured next step (§8).
- **Registers and locals:** the current frame's registers and locals share one `Map(i32, V)`
  (register `r` → key `r`, slot `s` → key `-1 - s`). Semantic has no indexed sequence update,
  and `MAP_GET` / `MAP_SET` are single host instructions whatever the map size. A frame's map is
  copied natively on each write, which is cheap at frame sizes in the tens.
- **Frames are data,** a `Sequence(Fr)`, not host recursion. A guest call costs no host call, and
  guest depth is bounded only by the guest Frames quota.

### 4.3 Dispatch

One loop. Operation codes are grouped by range (`< 10` locals and branches, `< 30` integer
arithmetic and comparisons, `< 50` bool and quad, `< 60` calls, `< 99` aggregates), so dispatch
is a short range test followed by a small chain.

The engine is deliberately one loop rather than a function per opcode. The host's Calls quota
(16,384 per run, not configurable, frozen in the reference) would otherwise cap a guest program
at about 16,000 instructions. Measured: a whole run costs at most 108 host calls,
independent of how much guest work it does (§6).

This is a constraint of the bootstrap host, not of the design. Cold paths (aggregate equality,
error text, table construction, decoding helpers) are separate functions.

### 4.4 Quad as a native domain

The two-plane encoding is kept as data (`N 00, F 01, T 10, S 11`: truth plane in bit 1, falsity
plane in bit 0). Every quad operation is a 16-entry table, derived once from the plane formulas,
so executing one is a single lookup. The two families stay distinct:

| Family | Opcodes | AND | OR | NOT | IMPL |
|---|---|---|---|---|---|
| Lattice (`&&` `\|\|` `!` `->`) | `Q_AND/OR/NOT/IMPL` | meet: `t = at∧bt, f = af∧bf` | join: `t = at∨bt, f = af∨bf` | swap planes | `NOT(a) join b` |
| QTruth (`qtruth_*` intrinsics) | `QTRUTH_*` | `t = at∧bt, f = af∨bf` | `t = at∨bt, f = af∧bf` | swap planes | `NOT(a) join b` |

NOT and IMPL coincide between the families by definition; AND and OR differ. All 16
combinations of every binary operation, and all 4 of each NOT, are compared with the oracle
(`v07`, `v08`). A mutation that makes `QT_AND` use the lattice falsity rule is caught.

### 4.5 Guest resource accounting

These are the reference's verified-local quotas, enforced on the guest by the Semantic VM itself:

| Quota | Value | Rule |
|---|---|---|
| Steps | 100,000 | charged once per decoded instruction, before it runs (including back-jumps); a failed charge reports `used = limit + 1` and leaves the counter at the limit, as the reference does |
| Calls | 16,384 | each nested user-function call; builtins are free; the entry frame is exempt |
| Frames | 256 | checked before Calls; `used = depth + 1` |

The guest counters (`guest_steps`, `guest_calls`, `peak_frames`) are kept separate from the
host's counters (the outer `smc` run's Steps and Calls). The harness compares the guest counters
with the oracle's own counters for the same artifact (`smc run --metrics json`); they are
identical on every comparable case.

## 5. Supported matrix

| Family | Opcodes | Status |
|---|---|---|
| Literals | `LOAD_Q`, `LOAD_BOOL`, `LOAD_I32`, `LOAD_U32`, `LOAD_TEXT` | VERIFIED |
| Locals | `LOAD_VAR`, `STORE_VAR` | VERIFIED |
| `i32` arithmetic | `ADD/SUB/MUL` (wrapping, as the reference); `DIV/MOD` (`DivisionByZero`, `MIN / -1` `ArithmeticOverflow`) | VERIFIED |
| `u32` arithmetic | `ADD/SUB/MUL` (checked, `ArithmeticOverflow`), `DIV/MOD`, `LT/LE` | VERIFIED |
| Comparisons | `CMP_EQ/NE` (scalars and structural aggregates), `CMP_I32_LT/LE` | VERIFIED |
| Bool | `BOOL_AND/OR/NOT` | VERIFIED |
| Quad, lattice | `Q_AND/OR/NOT/IMPL` | VERIFIED (all combinations) |
| Quad, QTruth | `QTRUTH_AND/OR/NOT/IMPL` | VERIFIED (all combinations) |
| Control | `JMP`, `JMP_IF`, `ASSERT`, `RET` | VERIFIED |
| Calls | `CALL` to user functions: SIG0 arity/family check, recursion, mutual recursion | VERIFIED (family-mismatch messages NOT TESTED: the verifier rejects such artifacts) |
| Text | `CONCAT_TEXT`; builtins `to_text`, `text_len`, `text_byte_at`, `text_slice`, `text_is_empty`, `text_starts_with`, `text_ends_with`, `print` (including its host-output-marker rule) | VERIFIED |
| Tuples | `MAKE_TUPLE`, `TUPLE_GET` | VERIFIED |
| Records | `MAKE_RECORD`, `RECORD_GET` | VERIFIED |
| ADTs | `MAKE_ADT`, `ADT_TAG`, `ADT_GET`; builtin `Option` from text builtins | VERIFIED |
| Sequences | `MAKE_SEQUENCE`, `SEQUENCE_GET/LEN/IS_EMPTY/CONTAINS/PUSH/PREPEND/POP` | VERIFIED |
| Application effects | 7 builtins; the guest gets no authority | VERIFIED (3 compared: `FsRead`, `StdoutWrite`, `ArgsRead`) |
| Steps / Frames quotas | | VERIFIED |
| Calls quota | | NOT TESTED against the oracle: under verified-local quotas, 16,385 calls need more than 100,000 Steps, so Steps always trips first |
| Map | `MAP_EMPTY/CONTAINS/GET/SET` | UNSUPPORTED (`u01`) |
| f64, fx | `LOAD_F64`, `LOAD_FX`, arithmetic | UNSUPPORTED (`u02`) |
| Closures | `MAKE_CLOSURE`, `CLOSURE_CALL` | UNSUPPORTED |
| PRNG | `RNG_SEED`, `RNG_NEXT_I32` | UNSUPPORTED |
| Bytes, endian, math builtins | | UNSUPPORTED |
| PROMETHEUS effects | `GATE_*`, `PULSE_EMIT`, `STATE_*`, `EVENT_POST`, `CLOCK_READ` | UNSUPPORTED |
| Ownership | `OWN0` Write-only events (executed exactly: with no Borrow, no write can conflict) | VERIFIED |
| Ownership | `OWN0` Borrow events (frame-local borrow tracking, `BorrowWriteConflict`) | UNSUPPORTED: calling such a function fails explicitly; an empty or Write-only `OWN0` is never taken as proof of borrow safety |
| Headers | `SEMCOD22` (rev 23), `SEMCOD23` (rev 24) | VERIFIED; every other header is rejected explicitly |
| Text values | ASCII 32..126 except `"` | IMPLEMENTED; other bytes give an explicit unsupported failure (a byte-to-text conversion is not available in Semantic) |

## 6. Evidence

`tests/vm/make_corpus.py` writes 48 programs to `tests/vm/corpus/`. `tests/vm/run_vm_tests.py`
runs the qualification (§3), writes `results.json` outside the tree, and exits nonzero on any
failure.

```bash
python3 tests/vm/make_corpus.py
SEMANTIC_SMC=<oracle smc-cli> SEMANTIC_C0_SMC=<C0 smc-cli> SEMANTIC_TEST_OUT=<dir> python3 tests/vm/run_vm_tests.py
```

Results at the head of this branch (Linux x86_64):

**Corpus: 46 VERIFIED, 2 UNSUPPORTED (as expected), 0 failures.**
- **Positive cases:** empty main, integer and bool logic, branching, loops with
  `break` / `continue`, assert, parameters of four families, nested calls, recursion, mutual
  recursion, fibonacci, unit functions, text, sequences, tuples, records, Option, user enums,
  both quad families, quad `match`, `u32`, records in sequences, a text builder, and four
  scaling workloads.
- **Expected failures:** assert, i32 and u32 division by zero, i32 `MIN / -1`, u32
  add/sub/mul overflow, the Steps and Frames quotas, sequence index out of range and negative,
  pop of an empty sequence, three denied capabilities, a trap after output, and a rejected
  print marker.
- **Per case:**
  - the artifact SHA-256 is recorded;
  - the oracle verifier admits it;
  - the oracle VM's outcome matches the case's stated expectation;
  - the Semantic VM's status, message, prints (on success) and guest Steps/Calls/Frames
    equal the oracle's.

**Determinism:** every case ran twice on the Semantic VM, and the two report lines were
byte-identical.

**Raw artifacts and transport, 9/9.** The oracle verifier rejects each malformed artifact, and
the Semantic VM's own decoder rejects it for the stated reason:
- truncated function;
- bad magic;
- unknown opcode;
- jump into the middle of an instruction;
- jump past the end;
- string id out of range;
- non-canonical bool literal;
- a read of an undefined register, caught at run time (`undefined register r4000`);
- invalid hex in transport.

**Mutations, 10/10 caught.** Each one deliberately breaks the VM and is detected by the
comparison:
- `ADD` computes `SUB`;
- `JMP_IF` polarity;
- QTruth AND uses the lattice rule;
- Steps limit off by one;
- Frames limit 257;
- aggregate equality ignores fields;
- the guest receives host effects;
- `u32` add wraps;
- `POP` drops nothing;
- `JMP` uncharged.

A record-name-only equality mutation is equivalent (`==` is typed, so compared records always
share a name) and was replaced by an observable one.

**C0 (recorded):** C0 admits 41 of the 48 artifacts and behaves identically to the oracle on
all 41. The other 7 need post-C0 capabilities (`text_len` and related builtins, `SEMCOD23`).

### Observation delivery

Both reference CLI routes (`run-smc` and `run`) deliver collected `print` observations only
after a successful run. On a failure the oracle shows no output. The Semantic VM reports the
observations up to the failure point (`x17` prints seven lines before `DivisionByZero`). Prints
are therefore compared on successful runs only. This is PARTIALLY VERIFIED for failing runs.

## 7. Measured cost of nested execution

Host Steps are counted by the outer reference run. A least-squares fit over the 48 cases gives:

```text
host_steps ≈ 170 · artifact_bytes + 101 · guest_steps + 23,600
```

| Case | Artifact bytes | Guest steps | Host steps | Host calls | Wall (ms) |
|---|---|---|---|---|---|
| `b01_empty_main` | 89 | 1 | 32,926 | 15 | 151 |
| `v07_quad_lattice` | 2,022 | 287 | 342,463 | 108 | 550 |
| `f05_fibonacci` | 316 | 23,676 | 2,939,542 | 24 | 757 |
| `w01_counting_loop` | 308 | 75,013 | 7,772,571 | 22 | 2,297 |
| `w03_call_heavy` (2,000 calls) | 449 | 50,013 | 5,803,366 | 27 | 2,656 |
| `w04_quad_batch` | 1,187 | 39,746 | 4,366,694 | 36 | 3,911 |

The reference VM runs the same artifacts natively in 0.01–0.04 s. **No performance claim** is made
for the Semantic VM. These numbers measure the bootstrap nesting, not the architecture.

Measured optimization of the loader (A/B on the same artifacts):

| Change | Effect |
|---|---|
| Opcode operand sizes and hex digits from `Map` lookups instead of compare chains | v07 decode 524k → 455k host Steps |
| Two-digit hex pairs and byte→character from tables | hex transport 117 → ~53 host Steps per byte; v07 total 455k → 342k; fit 272 → 170 Steps per byte; fixed table cost +16k Steps |

The execution core (101 host Steps per guest instruction) has not been optimized yet.

## 8. Gaps and next steps

1. **UNSUPPORTED families:** Map, f64/fx, closures, PRNG, Bytes and endian, PROMETHEUS effects,
   OWN0 Borrow tracking.
2. **Calls quota:** not reachable against the oracle under verified-local quotas (see §5).
3. **Memory:** no heap reclamation; O(n) copy per push. Next: a decode-time uniqueness analysis
   (as SHF-P3 does for the reference) to update an aggregate in place when the source register
   is dead.
4. **Dispatch cost:** 101 host Steps per guest instruction, with no breakdown yet. Next: a
   per-group cost measurement before restructuring.
5. **Text values** are limited to transportable ASCII.
6. **Admission:** the Semantic VM is not a verifier. Independent admission requires a separately
   qualified verifier written in Semantic.
7. **Platforms:** results are on Linux x86_64 only.
8. **Execution authority:** none. Any use of the Semantic VM in a proof requires a separately
   authorized contract revision ([ARCHITECTURE.md](./ARCHITECTURE.md) §7).
