# B0-00 Upstream Blocker Audit

Reference commit: `979def10135e1a90d7333d5501405343d498579e`
(`skulmakov-oss/Semantic`). Method: searched all issues (open and closed)
whose title/body mentions "quad" in `skulmakov-oss/Semantic` via `gh issue
list`, plus every issue number cited by name inside the source comments and
roadmap docs read during this audit.

## Resolved history that established today's contract (all CLOSED)

These are not blockers — they are the record of *how* the current, frozen
split between the legacy lattice family and QTruth came to be, and are
cited here because `foundation_contract.md` depends on their outcome.

| Issue | Title | Classification |
|---|---|---|
| #1440 | vm/core-quad: audit QAnd/QOr semantic mismatch before opcode migration | Resolved — produced `vm_core_quad_semantic_mismatch_audit.md` |
| #1442 | core-quad/vm: define separate lattice vs truth-table opcode contract | Resolved — produced `lattice_truth_opcode_contract.md` |
| #1444 | core-quad: expose explicit lattice aliases for QuadroReg32 | Resolved — added `lattice_meet`/`lattice_join`/`lattice_inverse` |
| #1446 | vm: route scalar quad opcodes through core-quad lattice aliases | Resolved — `sm-vm`'s `quad_and`/`quad_or`/etc. now call the lattice aliases |
| #1448 | docs(vm): close quad lattice bridge migration | Resolved — migration closeout |
| #1404 | core-quad: Quad Logic Engine v1 upgrade roadmap | Resolved — parent roadmap issue |
| #1732 | FA-05-002: verifier has no minimum-header revision gate for opcodes, so QTruth executes under SEMCODE0 | Resolved — set QTruth's `minimum_semcode_revision=19` |

**Classification: DOES_NOT_BLOCK_B0.** All closed; their resolution is
exactly the contract this document freezes (legacy family = lattice,
permanently, by an explicit non-substitution rule).

## Open issues reviewed

| Issue | Title | Classification | Reasoning |
|---|---|---|---|
| #1228 | HARNESS-M1: specify deterministic quad-state backend equivalence harness | DOES_NOT_BLOCK_B0 | Explicitly "architecture/planning only... does not approve runtime changes, verifier changes, SemCode changes." Proposes a future differential harness across CPU/Pulsar/WGPU backends; complementary to, not in conflict with, this contract's differential corpus. |
| #1800 | SemCode: research minimum-header-revision for other post-baseline, no-capability opcodes | DOES_NOT_BLOCK_B0 | Umbrella #1617. Explicitly enumerates the opcodes it covers (`AddI32`/`SubI32`/`MulI32`/`DivI32`/`ModI32`/`LoadU32`, tuple/record/ADT ops, `CmpI32Lt`/`CmpI32Le`, `Assert`) and explicitly distinguishes them from QTruth by the absence of a roadmap/reservation doc. Quad opcodes are not in this list; `docs/roadmap/core_quad/*` is exactly the kind of roadmap doc this issue says the *other* opcodes lack. Legacy Q* is not implicated. |
| #1729 | FA-04-023: CrystalFold annihilator rules can erase runtime type-mismatch traps on admitted raw IR | DOES_NOT_BLOCK_B0 (noted in contract) | Names `QAnd`/`QOr`'s one-sided annihilator rewrites explicitly as an instance of the same class of bug as `BoolAnd`/`BoolOr`. This is a soundness gap on **ill-typed raw IR** admitted past the verifier's structural (not type-family) checks — it does not change any of the 36 truth-table cases frozen here for well-typed programs, which were independently re-verified against CrystalFold's actual output. Recorded as a "Known unresolved question" rather than silently dropped. |
| #1778 | FA-11-001: prom-abi publicly admits non-canonical Quad(u8) values with no ABI-level validation | DOES_NOT_BLOCK_B0 | About `prom-abi`'s `AbiValue::Quad(u8)` carrier lacking a checked constructor — a defense-in-depth gap in one host-ABI crate. The actual admission boundary this contract relies on (`sm-vm::quad_from_abi`) already exhaustively rejects out-of-domain bytes (test: `quad_from_abi_matches_canonical_domain_exhaustively`). Does not change the canonical domain or its encoding, only where else it could theoretically be re-validated. Related: #1775 (`FA-09-007`, not separately reviewed — same class). |
| #1674 | FA-03-005: Logos condition inference treats any '&', '\|' or '->' text as a valid Quad condition | DOES_NOT_BLOCK_B0 | `Logos` is a separate UI/source-adapter subsystem (`docs/spec/logos.md`, `docs/spec/ui/*`), not the Foundation VM/IR/format layer this contract covers. A parsing-leniency bug in that adapter, not a Quad semantics question. |
| #1671 | FA-03-002: Logos semantic adapter maps source quad fields to QVec(1) instead of Quad | DOES_NOT_BLOCK_B0 | Same Logos UI-adapter subsystem; a mapping bug between a UI field type and `Quad`/`QVec`, not a change to `Quad`'s own semantics. |
| #1677 | FA-03-008: dead-When analysis treats quad N as always false contrary to four-state contract | DOES_NOT_BLOCK_B0 | A `sm-sema` lint bug that **violates** the documented contract (`docs/spec/types.md`: "`N` is not collapsed into `F`") rather than proposing to change it. If anything this issue is corroborating evidence *for* the state-identity section of this contract, not a threat to it. |

No issue reviewed was classified `UNCERTAIN`. No `BLOCKS_B0` or `UNCERTAIN`
finding was produced, so no STOP condition (issue #2's "Stop conditions"
section, or the originating task brief's §15) was triggered.

## Explicitly not chased further

- `#1775` (`FA-09-007`) — cited only in passing in an `sm-vm` code comment as
  the reason `quad_from_abi` exists as a distinct function from
  `u8_to_quad`. It documents the *fix* for the exact trust boundary this
  contract already treats as correctly enforced (see `quad_from_abi`
  discussion above and in `foundation_contract.md`); not independently
  re-read in full since the boundary it fixed is already exhaustively
  tested and cited directly.
- The `#1617` "[Phase A COMPLETE] audit: platform-wide readiness
  self-deception / fail-open map (18 modules)" umbrella issue is the parent
  of #1729/#1778 and several unrelated modules; not separately reviewed
  beyond its two Quad-relevant children above, since re-auditing all 18
  modules is out of this slice's scope.
