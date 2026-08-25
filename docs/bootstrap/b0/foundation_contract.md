# B0-00 Foundation Contract: Quad (legacy lattice family)

## Status

**Contract frozen for the legacy lattice family only.** Not yet `QUALIFIED` per
[`docs/BOOTSTRAP.md`](../../BOOTSTRAP.md) — no Semantic mirror has been
implemented against this contract yet. This document freezes what evidence
supports; implementation is a separate, later PR.

## Reference commit SHA

`979def10135e1a90d7333d5501405343d498579e` (`skulmakov-oss/Semantic`,
`origin/main`, 2026-08-25, "test(sm-vm): prove invocation rejection and
determinism contracts (#1839)").

All file/line evidence below refers to this commit unless stated otherwise.

## Scope

This contract covers exactly:

- the four-valued `Quad` state carrier (`N`/`F`/`T`/`S`) and its canonical
  identity/encoding;
- the **legacy lattice** operation family: `NOT` (inverse), `AND` (meet),
  `OR` (join), `IMPLIES` (derived: `NOT(a) OR b`);
- structural equality (`==`/`!=`) on `Quad`.

It explicitly does **not** cover parser syntax, AST types, VM
frames/registers, SemCode verifier policy beyond the one gating fact stated
under "Serialized representation", SIMD/packed optimization internals, UI,
host ABI trust boundaries, or unrelated collection/runtime values, per the
scope boundary in
[Semantic-Language#2](https://github.com/skulmakov-oss/Semantic-Language/issues/2).

## Canonical states

Four states, each a distinct, non-orderable identity — there is no
`PartialOrd`/`Ord` on the type at any audited layer:

| State | Meaning (normative) | Encoding |
|---|---|---|
| `N` | Null / unknown | `0b00` |
| `F` | Strict False | `0b01` |
| `T` | Strict True | `0b10` |
| `S` | Conflict / Super | `0b11` |

Source: [`docs/spec/quad_logic_frame_v1.md`](https://github.com/skulmakov-oss/Semantic/blob/979def10135e1a90d7333d5501405343d498579e/docs/spec/quad_logic_frame_v1.md)
("State Encoding" section, status "Frozen spec draft") and
[`docs/spec/types.md`](https://github.com/skulmakov-oss/Semantic/blob/979def10135e1a90d7333d5501405343d498579e/docs/spec/types.md#quad-lexical-model)
("Quad lexical model": *"`N` means unknown... `S` is not silently normalized
away, and `N` is not collapsed into `F`"*).

The encoding is verified frozen by an exhaustive test:
`crates/semantic-core-quad/src/lib.rs::quad_state_encoding_is_frozen`
(asserts `N.bits()==0b00`, `F.bits()==0b01`, `T.bits()==0b10`,
`S.bits()==0b11`) and independently by
`crates/sm-vm/src/semcode_vm.rs::quad_from_abi_matches_canonical_domain_exhaustively`
(asserts the same 0/1/2/3 mapping at the host-ABI boundary, and that every
other byte value `0x04..=0xFF` is rejected, not truncated).

### Identity is stable, but numeric commitment is not uniform across layers

- `crates/sm-front/src/types.rs::QuadVal` (frontend) — a plain
  `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] enum QuadVal { N, F, T, S }`
  with **no explicit discriminants and no `#[repr]`**. The frontend commits to
  four distinct nominal identities and nothing about their numeric encoding.
- `crates/sm-vm/src/lib.rs::QuadVal` (VM) — structurally identical enum,
  independently defined (not a re-export of the frontend type), also with no
  explicit discriminants at the Rust-enum level.
- `crates/semantic-core-quad/src/lib.rs::QuadState` — `#[repr(u8)] enum
  QuadState { N = 0b00, F = 0b01, T = 0b10, S = 0b11 }`. This is the layer
  where the numeric encoding is actually committed and frozen (see test
  above).
- The host-ABI boundary (`crates/sm-vm/src/semcode_vm.rs::quad_to_u8` /
  `quad_from_abi`) re-commits to the same `0/1/2/3` mapping and is the one
  place a **malformed** byte can arrive from outside the trusted VM (an
  untrusted host implementing `PrometheusHostAbi`); it is exhaustively
  rejected there, not masked. `u8_to_quad` (only compiled under `#[cfg(test)]`)
  masks with `& 0b11` instead, but is documented (semcode_vm.rs:3093-3096) as
  applying only to already-valid `QuadVal`s recombined via bitwise ops, never
  to untrusted input.

Conclusion: **the four-way nominal identity (N/F/T/S) is language semantics,
stable at every audited layer. The `0/1/2/3` numeric encoding is real and
frozen, but it is a `semantic-core-quad` / host-ABI wire-level commitment,
not a frontend-language-level one** — the frontend type carries no numeric
assumption at all. See "Serialized representation" below for exactly which
numeric facts are load-bearing for B0.

## Semantic operations (legacy lattice family)

Opcodes: `QNot`, `QAnd`, `QOr`, `QImpl`
(`crates/sm-format/src/local_format.rs:412-415`, byte values `0x10-0x13`).

Classification: **LANGUAGE_SEMANTIC.** This is the family the source-language
"evidence operators" (`&&`, `||`, `!`, `->` on `quad` operands) actually
lower to — see
[`docs/spec/types.md`](https://github.com/skulmakov-oss/Semantic/blob/979def10135e1a90d7333d5501405343d498579e/docs/spec/types.md#quad-operation-families)
("Evidence algebra | `a && b`, `a \|\| b`, `!a`, `a -> b` | `quad` | T/F
evidence-plane operation") and
`crates/sm-ir/src/legacy_lowering.rs:4199,4315,4333,4357` (`Type::Quad =>
IrInstr::QNot/QAnd/QOr/QImpl`). The spec gives a worked example that pins
this family, not the truth-table family, as the one `&&`/`||` reach:
`S || T -> S : quad` (types.md, "Evidence algebra" section) — `S` is the
**lattice join** result; the truth-table `map_or(S, T)` would be `T` (see
Explicit exclusions).

Definitions, all reducing to `QuadroReg32`'s explicitly-named
`lattice_inverse`/`lattice_meet`/`lattice_join` aliases
(`crates/semantic-core-quad/src/lib.rs:451-461`, "Explicit Lattice Aliases"),
which `sm-vm`'s `quad_not`/`quad_and`/`quad_or`/`quad_implies`
(`crates/sm-vm/src/semcode_vm.rs:3153-3168`) call by wrapping each `QuadVal`
into lane 0 of a `QuadroReg32` (`quad_lane0`/`quad_lane0_value`). Note this
is a *different* Rust implementation from `QuadState`'s own (separately
defined, single-value) `inverse`/`meet`/`join` methods
(`crates/semantic-core-quad/src/lib.rs:113-127`) — they are verified
mathematically equivalent (both reduce to the same bit formulas; lane 0 of
an otherwise-zeroed register behaves identically to a bare value), but the
VM runtime exercises the `QuadroReg32` path specifically, and this
contract's reference corpus is extracted through that exact path (see
Appendix A), not the simpler `QuadState`-level one:

- `NOT(a) = lattice_inverse(a)` = swap the truth/falsity bit planes
- `AND(a, b) = lattice_meet(a, b)` = bitwise AND of the 2-bit codes
- `OR(a, b) = lattice_join(a, b)` = bitwise OR of the 2-bit codes
- `IMPLIES(a, b) = lattice_join(lattice_inverse(a), b)` = `NOT(a) OR b`

`IMPLIES`'s exact formula is a **documented, frozen policy decision**, not
free choice:
[`docs/spec/quad_logic_frame_v1.md`](https://github.com/skulmakov-oss/Semantic/blob/979def10135e1a90d7333d5501405343d498579e/docs/spec/quad_logic_frame_v1.md)
§"IMPLIES Policy": *"The existing `IMPLIES` execution semantics (`A -> B =
NOT(A) join B`) must be retained to preserve backward compatibility."*

Independent second execution surface: `crates/semantic-core-exec/src/lib.rs`
(a separate bytecode/VM, `CoreOpcode`/`Instr`, its own opcode numbering)
implements `QNot`/`QAnd`/`QOr`/`QImpl` by calling `.inverse()`/`.meet()`/
`.join()` directly on `QuadState`
(`crates/semantic-core-exec/src/lib.rs:1059,1066,1078,1085-1092`) — i.e. the
*same* canonical algebra, not a re-derivation. This cross-validates
`semantic-core-quad` as the single semantic owner (see
[reference_inventory.md](reference_inventory.md)).

## Truth tables

Machine-readable, mechanically extracted (not hand-typed) at
[`reference/b0/reference_vectors.json`](../../../reference/b0/reference_vectors.json)
— `not`/`and`/`or`/`implies` (52 cases total) plus a 16-case `eq` table (see
"Equality / identity rules" below). Extraction method and reproduction
instructions: Appendix A below and
[`qualification/b0/check_reference_vectors.py`](../../../qualification/b0/check_reference_vectors.py).

### NOT (4 cases)

| a | NOT(a) |
|---|---|
| N | N |
| F | T |
| T | F |
| S | S |

### AND (16 cases)

| AND | N | F | T | S |
|---|---|---|---|---|
| **N** | N | N | N | N |
| **F** | N | F | N | F |
| **T** | N | N | T | T |
| **S** | N | F | T | S |

### OR (16 cases)

| OR | N | F | T | S |
|---|---|---|---|---|
| **N** | N | F | T | S |
| **F** | F | F | S | S |
| **T** | T | S | T | S |
| **S** | S | S | S | S |

### IMPLIES (16 cases)

| IMPLIES | N | F | T | S |
|---|---|---|---|---|
| **N** | N | F | T | S |
| **F** | T | S | T | S |
| **T** | F | F | S | S |
| **S** | S | S | S | S |

All four tables above were cross-derived two independent ways during this
audit (by hand from the documented bit-plane formulas, and mechanically via
the reference crate's own methods) and matched exactly; they additionally
match
[`docs/roadmap/core_quad/vm_core_quad_semantic_mismatch_audit.md`](https://github.com/skulmakov-oss/Semantic/blob/979def10135e1a90d7333d5501405343d498579e/docs/roadmap/core_quad/vm_core_quad_semantic_mismatch_audit.md)'s
"Legacy QAnd"/"Legacy QOr" tables, which is the resolved audit that
established this family's semantics (see blocker_audit.md, issues #1440/#1442/#1444/#1446).

## Equality / identity rules

`Quad == Quad` is structural identity, returns `bool`, and performs no
evidence merging or conflict resolution:
[`docs/spec/types.md`](https://github.com/skulmakov-oss/Semantic/blob/979def10135e1a90d7333d5501405343d498579e/docs/spec/types.md#quad-equality)
§"Quad equality" (`N == N -> true`, `N == S -> false`, etc. — all four states
mutually distinct, no coercion). All three Rust `QuadVal`/`QuadState`
definitions audited derive plain `PartialEq, Eq` (structural, not custom)
with no `PartialOrd`/`Ord` anywhere in the audited files. All 16 cases are
mechanically extracted through **two independent oracle surfaces**, both
recorded in `reference/b0/reference_vectors.json`:

- `eq` — `QuadState`'s own derived `PartialEq`.
- `vm_eq` — the *source-language* `==` operator, compiled from real `.sm`
  source and run through the public `sm_emit::compile_program_to_semcode`
  -> `sm_verify::verify_semcode_token` ->
  `sm_vm::run_verified_function_semcode_with_args` pipeline. This exercises
  `Opcode::CmpEq`'s runtime handler
  (`crates/sm-vm/src/semcode_vm.rs:2439-2447`), which calls the VM's
  private `value_eq` function directly — a genuinely different code path
  from `QuadState::PartialEq`, closing the gap where `eq` staying correct
  could no longer guarantee `value_eq` also did.

## Serialized representation

The **`0/1/2/3` numeric encoding for `N/F/T/S` is a frozen, contractually
required fact**, but only at these two specific points, both mechanically
verified:

1. `semantic-core-quad::QuadState` bit encoding — mechanically extracted
   via `QuadState::bits()` into `reference/b0/reference_vectors.json`'s
   `state_encoding` field (also backed by the reference's own
   `quad_state_encoding_is_frozen` test).
2. The SemCode host-ABI boundary (`quad_to_u8`/`quad_from_abi`) — this is
   the one place external, untrusted bytes are converted to `Quad` and is
   explicitly documented as a closed 4-value domain, not a raw byte. Both
   functions are private with no lightweight public entry point (reaching
   them requires a full host-call round trip through
   `run_verified_semcode_with_host_and_capabilities*` and a
   `PrometheusHostAbi` implementation), so this claim is proven not by
   extraction but by `qualification/b0/check_reference_vectors.py`
   requiring the reference's own exhaustive tests for it
   (`quad_from_abi_matches_canonical_domain_exhaustively`,
   `gate_read_admits_every_canonical_quad_byte`,
   `gate_read_rejects_every_out_of_domain_quad_byte`) to actually run and
   pass at the pinned commit — not merely exit 0, which a test filter
   matching nothing would also report.

The `QAnd`/`QOr`/`QNot`/`QImpl` **opcode bytes** (`0x10`/`0x11`/`0x12`/`0x13`,
`crates/sm-format/src/local_format.rs:412-415`) and their
**`minimum_semcode_revision` of `1`** (baseline, no header/capability gate —
`crates/sm-format/src/local_format.rs:634-639`, "present since the original
enum") are also frozen wire facts, needed to know this family requires no
special SemCode header revision, unlike QTruth (see Explicit exclusions).
Both are mechanically extracted via `sm_format::Opcode`'s own public
`.byte()` and `.minimum_semcode_revision()` methods into the
`opcode_encoding`/`minimum_semcode_revision` fields of
`reference/b0/reference_vectors.json`, not hand-copied from source reading
alone - a real, demonstrated risk: during this same review round, a
paraphrase of this exact fact (in chat, not in this file) transposed the
QNot/QAnd/QOr/QImpl byte order relative to the actual enum, which
generating the table from `Opcode::byte()` directly would have caught
immediately had it been the source of truth.

Nothing else about representation (register allocation, instruction operand
byte layout beyond the two register/three register shape, `QuadroReg32`
packing, `QuadTile128` GPU layout) is claimed as part of the B0 language
contract — those are VM/GPU-transport implementation detail
(`crates/semantic-core-quad/src/lib.rs:684-692`: *"this is not itself the
GPU upload ABI... the visual adapter owns any GPU transport
representation"*).

## Explicit exclusions

- **The QTruth family** (`QTruthAnd`/`QTruthOr`/`QTruthNot`/`QTruthImpl`,
  opcodes `0x17-0x1A`) is a **separate, later-added, genuinely distinct**
  Belnap truth-table algebra, not a duplicate name for the same thing:
  - `docs/spec/quad_logic_frame_v1.md` §"Operation Families": *"Mixing
    operations between truth-table families and knowledge-lattice families
    by name is strictly forbidden."*
  - `docs/roadmap/core_quad/lattice_truth_opcode_contract.md` §5
    ("Non-Substitution Rule"): *"Under no circumstances may a
    `semantic-core-quad` truth-table mapping be silently substituted as a
    backend for a lattice opcode."* — resolved via issues #1440/#1442/#1444/#1446
    (all closed; see blocker_audit.md).
  - It requires SemCode `minimum_semcode_revision = 19` (`SEMCOD18`), added
    by #1732 (closed) specifically because no earlier header ever claimed it
    — i.e. it is **not** part of the SemCode baseline the legacy family sits
    in.
  - `AND`/`OR` genuinely differ in value from the legacy family (e.g.
    `F legacy-AND T = N` vs. `F QTruth-AND T = F`); only `NOT` and `IMPLIES`
    happen to compute the identical function today (see "Known unresolved
    questions").
  - `crates/sm-ir/src/passes/crystalfold.rs` constant-folds the legacy
    family (`QAnd`/`QOr`/`QNot`/`QImpl`, lines 302-445) but explicitly does
    **not** fold QTruth instructions (lines 447-461 just pass them through)
    — an additional maturity/coverage asymmetry, not a semantic one.
  - Reference vectors for QTruth are **not** included in this contract's
    corpus. A future B-slice may cover it separately.
- **`crates/ton618-core`** — explicitly documented
  (`docs/spec/quad_logic_frame_v1.md` §"Ownership Boundary") as *"retained
  exclusively for backward compatibility purposes"*; not the canonical
  owner. Not audited further in this pass (see reference_inventory.md).
- **Host-ABI trust-boundary hardening** (issue #1778, open — `prom-abi`'s
  `AbiValue::Quad(u8)` carrier has no constructor-level validation) is a
  robustness gap in one consumer crate, not a contract ambiguity: the
  canonical domain and its ABI-boundary validation (`quad_from_abi`) are
  already exhaustively specified and tested at the `sm-vm` layer. Excluded
  from B0 scope; tracked in blocker_audit.md as DOES_NOT_BLOCK_B0.
- Parser/lexer surface syntax, AST shape, VM register/frame layout, SIMD
  packing (`QuadroReg32`/`QuadTile128`), UI/GPU adapters, and
  `semantic-core-exec`'s independent bytecode format are all out of scope
  per the issue's own scope boundary; `semantic-core-exec` is noted only to
  establish that it is not a duplicate semantic owner.

## Known unresolved questions

1. **`NOT` and `IMPLIES` are bit-identical between the legacy lattice family
   and the QTruth family today** (`lattice_inverse` and `map_not` both
   reduce to the same bit-swap; `IMPLIES`'s policy is explicitly shared by
   design per the spec quoted above). `AND`/`OR` are not. This is evidenced,
   not a contradiction, but it means "are Q* and QTruth* the same algebra"
   does not have one uniform yes/no answer per operation — recorded here so
   a future slice does not accidentally assume `QNot`/`QTruthNot` diverging
   is impossible by construction, only that they coincide under the current
   frozen definitions of both families.
2. `crates/semantic-core-quad/src/lib.rs`'s `QuadTile128::map_not` carries a
   doc comment explicitly disclaiming that it is "not an alias for the
   knowledge-lattice `inverse`" even though, for the 32-lane register form
   actually used by the VM, they currently compute identically (see #1
   above) — i.e. the two operations are conceptually kept separate by the
   API design even where they coincide in value. Not a blocker; noted for
   anyone tempted to collapse the two families based on today's `NOT`
   behavior alone.
3. Issue #1729 (open) documents that `CrystalFold` applies one-sided
   annihilator rules for `QAnd`/`QOr` (`N` as AND-annihilator, `S` as
   OR-annihilator) without proving the other operand's runtime type on
   raw/adversarial IR, which can erase a would-be runtime type-mismatch
   trap. This does not change the truth tables above for well-typed
   programs (verified: the constant-fold results match this contract's
   truth tables exactly for all 16+16+4 cases) and is therefore
   DOES_NOT_BLOCK_B0, but a future VM/optimizer bootstrap slice should not
   assume CrystalFold's current annihilator behavior is soundness-clean on
   ill-typed raw IR.

## Qualification boundary

This document and its corpus establish `REFERENCE_ONLY` evidence for the
legacy-lattice `Quad` contract, per
[`docs/BOOTSTRAP.md`](../../BOOTSTRAP.md)'s slice lifecycle (steps 1-3 of 8:
select contract, capture reference behavior, define canonical comparison
form). It does **not** by itself qualify anything, because no Semantic
mirror exists yet (steps 4-8 remain). The comparison form fixed here is
**exact structured identity** (tier 2 of `BOOTSTRAP.md`'s comparison
hierarchy) over `{state_encoding, opcode_encoding,
minimum_semcode_revision, not, and, or, implies, eq, vm_eq}` as recorded in
`reference/b0/reference_vectors.json`, plus a pass/fail requirement (not a
value comparison) that the reference's own host-ABI boundary tests run and
pass at the pinned commit. `--reference-checkout` must itself be a clean
checkout with git HEAD exactly at the frozen commit by default
(`--allow-reference-drift` opts out explicitly, and visibly labels the
result `PASS (DRIFT MODE)` rather than a qualification pass).

Mutation-proof evidence (required by issue #2's qualification checklist) is
recorded in this PR's description: numerous deliberate mutations - value
mutations in the committed corpus (e.g. `T AND T`: `T` -> `S`, `S eq S`:
`true` -> `false`, `opcode_encoding.QAnd`, `minimum_semcode_revision.QNot`,
`vm_eq`'s `T,T` cell), structural/domain/coverage corruption, frozen-
constant substitution, reference-checkout identity mismatches, and a
mutation of the reference implementation itself (`QuadroReg32::lattice_meet`,
and separately `quad_from_abi`) - each caused
`qualification/b0/check_reference_vectors.py` to fail with an exact
reported cause, then were reverted and re-verified passing.

## Appendix A: extraction probe source

Reproduced at [`reference/b0/dump_b0_vectors.rs`](../../../reference/b0/dump_b0_vectors.rs)
for readability; it is not compiled as part of this repository.
`qualification/b0/check_reference_vectors.py` builds and runs it as the
`src/main.rs` of a throwaway crate in its own temp directory, with `path`
dependencies on the reference checkout's `semantic-core-quad`, `sm-format`,
`sm-emit`, `sm-verify`, and `sm-vm` crates - the checkout itself is only
ever read, never written to (no file is ever installed into or removed
from it), so two qualification runs against the same checkout cannot race
on shared mutable state there. Row/column order in the emitted tables is
an explicit `N/F/T/S` literal in the probe, not `QuadState::ALL`'s own
iteration order, so a pure reorder of that array upstream cannot appear as
a spurious corpus divergence.

Four independent oracle surfaces, each through its own real entry point:

- `not`/`and`/`or`/`implies` route through lane 0 of a `QuadroReg32` and
  its `lattice_inverse`/`lattice_meet`/`lattice_join` methods — the exact
  call chain `sm-vm`'s own opcode handlers use — rather than `QuadState`'s
  separately-implemented single-value methods.
- `eq` reads `QuadState`'s own derived `PartialEq` directly.
- `vm_eq` compiles and runs `{a} == {b}` as real `.sm` source through the
  public `sm_emit::compile_program_to_semcode` ->
  `sm_verify::verify_semcode_token` ->
  `sm_vm::run_verified_function_semcode_with_args` pipeline, exercising
  `Opcode::CmpEq`'s runtime handler (which calls the VM's private
  `value_eq` function) — a genuinely different path from `eq`.
- `opcode_encoding`/`minimum_semcode_revision` read `sm_format::Opcode`'s
  own public `.byte()`/`.minimum_semcode_revision()` methods.

The host-ABI boundary (`quad_to_u8`/`quad_from_abi`) is a private function
pair with no lightweight public entry point reachable without a full
host-call round trip; that specific claim is proven not by extraction but
by the qualification script separately requiring the reference's own
exhaustive tests for it to run and pass (see "Serialized representation"
above). No reference-repo source was copied or reimplemented by hand into
any corpus value — every value is the live return of a reference-crate
call, or the pass/fail result of the reference's own test suite.
