# B0-00 Reference Inventory

Reference commit: `979def10135e1a90d7333d5501405343d498579e`
(`skulmakov-oss/Semantic`, `origin/main`). File paths are relative to that
repository's root.

Classification legend: `LANGUAGE_SEMANTIC`, `WIRE_FORMAT`, `VERIFIER_RULE`,
`VM_IMPLEMENTATION`, `OPTIMIZATION`, `LEGACY`, `EXPERIMENTAL`, `UNKNOWN`.

## State type duplication

| Concept | File | Role | Canonical? | Notes |
|---|---|---|---|---|
| `QuadVal` enum | `crates/sm-front/src/types.rs:79` | Frontend AST/type representation | Identity yes, encoding no | No discriminants, no `#[repr]`. Pure nominal 4-way enum. |
| `QuadVal` enum | `crates/sm-vm/src/lib.rs:16` | VM value representation | Identity yes, encoding no | Independently defined (not `use`d from sm-front), structurally identical. No cross-crate type alias links the two — see "Unresolved" below. |
| `QuadState` enum | `crates/semantic-core-quad/src/lib.rs:60-65` | The actual algebra owner | **Yes — sole canonical owner** | `#[repr(u8)]` with explicit `N=0b00,F=0b01,T=0b10,S=0b11)`. Only place the numeric encoding is a Rust-level type invariant. Declared "canonical owner of the Quad Logic Frame implementation" in `docs/spec/quad_logic_frame_v1.md` §"Ownership Boundary". |
| `QuadVal`/`QuadState` bridge | `crates/sm-vm/src/semcode_vm.rs:3123-3136` (`quadval_to_quadstate`/`quadstate_to_quadval`) | Conversion | N/A | 1:1 identity-preserving match, no reordering, no encoding step (pure enum-to-enum). |
| `quadro` (legacy) | `crates/ton618-core/src/quadro.rs` | Retained-for-compat implementation | **No** — explicitly non-canonical | `docs/spec/quad_logic_frame_v1.md` §"Ownership Boundary": *"retained exclusively for backward compatibility purposes."* Not audited beyond confirming this explicit disclaimer; out of B0 scope. |

**Finding:** three Rust types name "the same concept" (`sm-front::QuadVal`,
`sm-vm::QuadVal`, `semantic-core-quad::QuadState`), plus a fourth legacy one
(`ton618-core::quadro`). This is **duplicated Rust representation, not
duplicated semantic ownership** — only `semantic-core-quad::QuadState`
carries the algebra (meet/join/inverse/map_*) and the frozen numeric
encoding; the other two `QuadVal` enums are pure identity carriers bridged
1:1 by hand-written converters, and `ton618-core` is explicitly disclaimed
as non-canonical by the normative spec. There is a single clean semantic
owner (`semantic-core-quad`); there is not a single clean *Rust type* owner,
which is exactly the kind of distinction issue #2 asked this audit to make.

## Operation ownership

| Operation | File / owner | Role | Canonical? | Notes |
|---|---|---|---|---|
| `QNot`/`QAnd`/`QOr`/`QImpl` opcode bytes | `crates/sm-format/src/local_format.rs:412-415` | Wire format | Yes | `0x10-0x13`, `minimum_semcode_revision=1` (baseline). |
| `QNot`/`QAnd`/`QOr`/`QImpl` IR variants | `crates/sm-ir/src/legacy_lowering.rs:199-216` | IR representation | Yes | Lowered from `&&`/`\|\|`/`!`/`->` on `Type::Quad` operands (lines 4199,4315,4333,4357). |
| `quad_not`/`quad_and`/`quad_or`/`quad_implies` | `crates/sm-vm/src/semcode_vm.rs:3153-3168` | VM_IMPLEMENTATION | Delegates to canonical algebra | Each is a one-line call into `QuadroReg32::lattice_inverse/lattice_meet/lattice_join`. |
| `lattice_meet`/`lattice_join`/`lattice_inverse` | `crates/semantic-core-quad/src/lib.rs:451-461` | **LANGUAGE_SEMANTIC** | **Canonical definition** | "Explicit Lattice Aliases" — deliberately named apart from `map_*` to prevent silent substitution (see `docs/roadmap/core_quad/lattice_truth_opcode_contract.md` §5). |
| `QAnd`/`QOr`/`QNot`/`QImpl` fold rules | `crates/sm-ir/src/passes/crystalfold.rs:302-445` (`quad_not_const`/`quad_and_const`/`quad_or_const`, lines 835-866) | OPTIMIZATION | Mirrors canonical semantics | Reimplements the lattice formulas directly on the `u8` encoding (bitwise AND/OR, bit-swap) rather than calling `semantic-core-quad`; verified to produce identical results to `lattice_meet`/`lattice_join`/`lattice_inverse` for all 16+16+4 cases (see foundation_contract.md truth tables) but is a **second, independent implementation of the same formulas**, not a shared call site. Also applies one-sided annihilator rewrites (`N` absorbs `AND`, `S` absorbs `OR`) — see issue #1729, DOES_NOT_BLOCK_B0. |
| `CoreOpcode::QNot`/`QAnd`/`QOr`/`QImpl` | `crates/semantic-core-exec/src/lib.rs:141-144,264,278,1055-1092` | VM_IMPLEMENTATION (separate execution engine) | Delegates to canonical algebra | Own opcode numbering (9,12,...) and own `Instr`/`CoreProgram` format, entirely distinct from `sm-format`'s. Execution calls `QuadState::inverse()/.meet()/.join()` directly (lines 1059,1066,1078,1085-1092) — same canonical primitive, not a re-derivation. Purpose beyond "a second core execution engine" not audited further; out of B0 scope but confirmed not to be a competing semantic owner. |
| `QTruthAnd`/`QTruthOr`/`QTruthNot`/`QTruthImpl` opcodes | `crates/sm-format/src/local_format.rs:419-422` | WIRE_FORMAT | Yes, but separate contract | `0x17-0x1A`, `minimum_semcode_revision=19`. Excluded from B0 (see foundation_contract.md). |
| `quad_truth_not`/`quad_truth_and`/`quad_truth_or`/`quad_truth_implies` | `crates/sm-vm/src/semcode_vm.rs:3169-3184` | VM_IMPLEMENTATION | Delegates to canonical algebra | Calls `QuadroReg32::map_not/map_and/map_or/map_implies`. Excluded from B0. |
| `NOT_LUT`/`AND_LUT`/`OR_LUT`/`IMPLIES_LUT` | `crates/semantic-core-quad/src/logic_frame.rs:10-124` | **LANGUAGE_SEMANTIC** (QTruth family) | Canonical definition of the *excluded* family | Human-readable const LUTs, exhaustively self-tested (`test_and_or_plane_formulas`, `test_implies_table`, etc.). Not part of B0's frozen corpus, but is the authoritative source if/when a future slice covers QTruth. |
| QTruth CrystalFold handling | `crates/sm-ir/src/passes/crystalfold.rs:447-461` | OPTIMIZATION (absence of one) | N/A | Explicitly passes QTruth instructions through unfolded — no constant-folding support exists yet for this family, unlike the legacy family. |
| verifier operand decode for both families | `crates/sm-verify/src/lib.rs:1365-1379` | VERIFIER_RULE | Yes | Both families share identical wire operand shape (2-register unary, 3-register binary); this is the only place they are handled by one shared code path, and it's purely structural (register-count decoding), not semantic. |

## Duplicate-owner audit summary

Single canonical semantic owner: **`crates/semantic-core-quad`**
(`QuadState` + `logic_frame` module). Every execution surface audited
(`sm-vm`'s SemCode VM, `semantic-core-exec`'s separate Core VM,
`sm-ir`'s CrystalFold optimizer) either calls into this crate directly or
(CrystalFold only) reimplements its formulas on the raw encoding in a way
verified to match exactly. No contradiction was found between these
surfaces for the legacy lattice family within this audit's scope. The one
genuine architectural fork (legacy lattice vs. QTruth Belnap) is not a
duplication bug — it is a deliberately designed, spec-documented, and
issue-resolved (#1440/#1442/#1444/#1446) separation between two named
families, resolved in favor of **not** merging them (§5,
"Non-Substitution Rule", `lattice_truth_opcode_contract.md`).
