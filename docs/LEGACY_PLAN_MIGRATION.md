# Legacy Plan Migration

Status: record of the SHF architecture reset. This is the **only** active document allowed to
use the retired milestone vocabularies (B0–B8, SH-xx, IP-xx, SRI-xx) — as history.

Reset base: `Semantic-Language@d43ba530b1ec70b7a35e12c0b8d15cf90d2ca5ca`.
Reference planning pin: `Semantic@5f3e230223a35384cb505e418fdac0b07102d2e6` (no drift from the
previously observed value at reset time).

None of the GitHub objects below is closed or edited by the reset. Their final disposition is a
separate owner decision.

## 1. Classification vocabulary

| Class | Meaning |
|---|---|
| KEEP-AS-CANON | Survives unchanged in the active architecture |
| SALVAGE-AND-REWRITE | Idea survives, rewritten under SHF ownership |
| MOVE-POST-BOOTSTRAP | Valid, but moved to [FUTURE.md](./FUTURE.md) |
| HISTORICAL-EVIDENCE | Preserved as provenance; not active |
| SUPERSEDE | Replaced by an SHF equivalent |
| REJECT | Not adopted |

## 2. Salvage matrix

| Concept | Source | Class | Destination |
|---|---|---|---|
| B0–B8 roadmap | README, ROADMAP, #1, #4 | SUPERSEDE | SHF-0 … SHF-17 ([ROADMAP.md](./ROADMAP.md)) |
| SH-00 … SH-27 numbering | #4, PR #6 | SUPERSEDE | SHF stages (same content, upstream `#1910` numbering) |
| IP-00 … IP-10 | #4 | MOVE-POST-BOOTSTRAP | [FUTURE.md](./FUTURE.md) Instant Pipeline |
| SRI-00 … | #5 | MOVE-POST-BOOTSTRAP | [FUTURE.md](./FUTURE.md) SRI |
| REFERENCE_ONLY → … → CANONICAL | ARCHITECTURE, #1, #4 | KEEP-AS-CANON | [ARCHITECTURE.md](./ARCHITECTURE.md) §9–10 |
| Quad qualification (legacy lattice) | #2, PR #3 | HISTORICAL-EVIDENCE | future requalification under SHF only if `S` needs it |
| Reference vectors | PR #3 | SALVAGE-AND-REWRITE | [QUALIFICATION.md](./QUALIFICATION.md) §4 |
| Mutation testing | PR #3 | SALVAGE-AND-REWRITE | [QUALIFICATION.md](./QUALIFICATION.md) §4 |
| Exact-SHA pinning | PR #3, #4 | KEEP-AS-CANON | [BOOTSTRAP_CONTRACT.md](./BOOTSTRAP_CONTRACT.md) §2, `reference/semantic-reference.toml` |
| Fail-closed differential gates | PR #3 | SALVAGE-AND-REWRITE | [QUALIFICATION.md](./QUALIFICATION.md) §4 |
| Legacy lattice vs QTruth distinction | PR #3 | KEEP-AS-CANON | [QUALIFICATION.md](./QUALIFICATION.md) §5 |
| BOOTSTRAP_SUBSET | PR #6 | SALVAGE-AND-REWRITE | [BOOTSTRAP_SUBSET.md](./BOOTSTRAP_SUBSET.md) (no B1–B5 / SH-00 / BS-000) |
| HOST_CAPABILITY_ABI | PR #6 | SALVAGE-AND-REWRITE | [HOST_BOUNDARY.md](./HOST_BOUNDARY.md) (policy only) |
| Canonical SemCode ordering / forbidden hidden inputs | PR #6 | SALVAGE-AND-REWRITE | [BOOTSTRAP_CONTRACT.md](./BOOTSTRAP_CONTRACT.md) §6 |
| Host-side compiler-logic prohibition | PR #6, #4 | KEEP-AS-CANON | [HOST_BOUNDARY.md](./HOST_BOUNDARY.md) §3 |
| C0 → C1 → C2 | PR #6, #4, `#1910` | KEEP-AS-CANON | [BOOTSTRAP_CONTRACT.md](./BOOTSTRAP_CONTRACT.md) |
| Bootstrap Seal | PR #6, #4 | KEEP-AS-CANON | [BOOTSTRAP_CONTRACT.md](./BOOTSTRAP_CONTRACT.md) §7 |
| Verifier rewrite (B6) | ROADMAP, #1 | MOVE-POST-BOOTSTRAP | [FUTURE.md](./FUTURE.md) |
| VM rewrite (B7) | ROADMAP, #1 | MOVE-POST-BOOTSTRAP | [FUTURE.md](./FUTURE.md) |
| Instant Pipeline | #4 | MOVE-POST-BOOTSTRAP | [FUTURE.md](./FUTURE.md) |
| Persistent compiler | #4 | MOVE-POST-BOOTSTRAP | [FUTURE.md](./FUTURE.md) |
| Incremental compiler | #4 | MOVE-POST-BOOTSTRAP | [FUTURE.md](./FUTURE.md) |
| SEMIMG | #4 | MOVE-POST-BOOTSTRAP | [FUTURE.md](./FUTURE.md) |
| SRI | #5 | MOVE-POST-BOOTSTRAP | [FUTURE.md](./FUTURE.md) |
| TypeScript Workbench | #4, #5, PR #6 | MOVE-POST-BOOTSTRAP | [FUTURE.md](./FUTURE.md) (presentation only) |
| "Semantic-Language owns language semantics" (SRI §3) | #5 | REJECT | language semantics stay upstream ([OWNERSHIP.md](./OWNERSHIP.md)) |
| Full Sigma, t¤ | upstream `#1909` | MOVE-POST-BOOTSTRAP | [FUTURE.md](./FUTURE.md) |

## 3. Per-item mapping

### Issue #1 — Bootstrap Program: Semantic → Semantic

- **Old role:** umbrella program with phases B0–B8.
- **Valuable:** no-silent-divergence invariants; migration state model; "bootstrap is not a
  rewrite"; UI outside the critical path; success defined as reproducing the toolchain.
- **Obsolete:** B0–B8 phases; B6 verifier and B7 VM as phases of the self-hosting program.
- **New destination:** README, [ARCHITECTURE.md](./ARCHITECTURE.md), [ROADMAP.md](./ROADMAP.md).
- **Recommended disposition:** close as superseded by the SHF self-hosting program.

### Issue #2 — B0-00: freeze the first Foundation contract

- **Old role:** first slice, Quad Foundation contract.
- **Valuable:** contract-before-port discipline; ownership questions for `QuadVal` across layers;
  exhaustive finite-domain vectors; stop conditions.
- **Obsolete:** B0-00 numbering; Quad as automatic first self-hosting milestone.
- **New destination:** principles in [QUALIFICATION.md](./QUALIFICATION.md); Quad work is not on
  the SHF critical path unless `S` requires it.
- **Recommended disposition:** close as historical Foundation/Quad contract work.

### PR #3 — B0-00: freeze Foundation contract for legacy-lattice Quad

- **Head:** `04e2ffe497220509e5831a9b5b7bddfd5308b38c`; historical reference pin
  `Semantic@979def10135e1a90d7333d5501405343d498579e` (not the current planning reference).
- **Valuable:** the evidence discipline listed in [QUALIFICATION.md](./QUALIFICATION.md) §4 —
  exact pinning with clean-checkout check, mechanical extraction, explicit schema, positive /
  negative / boundary vectors, exhaustive coverage checks, mutation proofs (17 recorded),
  independent normative oracle, fail-closed structural validation, runtime-path and host-ABI
  verification, ordering independence, target-directory isolation, exit-code checking, limitation
  disclosure; and the legacy lattice vs QTruth distinction.
- **Obsolete:** B0-00 numbering; old pin as oracle; treating its gate as qualified for SHF.
- **Unresolved finding (recorded, not fixed):** Codex P2 on `04e2ffe` — the `not` / `and` / `or` /
  `implies` tables call `QuadroReg32` register methods directly rather than going through the
  public source → lowering → SemCode → verifier → VM path (only `vm_eq` does), so a divergence in
  lowering, opcode dispatch or `sm-vm::quad_*` handlers could pass under
  `--allow-reference-drift`.
- **New destination:** HISTORICAL-EVIDENCE / future-requalification material.
- **Recommended disposition:** close unmerged as historical evidence; any Quad requalification
  happens fresh under SHF.

### Issue #4 — Semantic Self-Hosting + Instant Pipeline (C0 → C1 → C2)

- **Old role:** combined self-hosting and performance milestone.
- **Valuable:** C0 → C1 → C2 definition; correctness-era / performance-era split; authority
  reconciliation (stale `sm-ir` vs `sm-format` ownership wording upstream); Bootstrap Seal
  contents; fail-closed admission reuse; clean-build oracle.
- **Obsolete:** B0–B8 lane; SH-xx numbering; acting as combined authority for both tracks;
  B6/B7 inside the self-hosting lane.
- **New destination:** self-hosting content → SHF ([ROADMAP.md](./ROADMAP.md),
  [BOOTSTRAP_CONTRACT.md](./BOOTSTRAP_CONTRACT.md)); Instant Pipeline → [FUTURE.md](./FUTURE.md).
- **Recommended disposition:** split — close the combined issue; open a post-Bootstrap Instant
  Pipeline issue when that track is authorized.

### Issue #5 — Semantic Reactive Interface (SRI)

- **Old role:** TypeScript reactive tooling boundary milestone.
- **Valuable:** TypeScript as presentation-only; no `quad` → `boolean` collapse; revision /
  cancellation / supersession model; generated bindings.
- **Obsolete:** claim that Semantic-Language owns language semantics (rejected: upstream owns them).
- **New destination:** [FUTURE.md](./FUTURE.md).
- **Recommended disposition:** keep as the post-Bootstrap SRI successor, re-scoped to depend on
  the Bootstrap Seal.

### PR #6 — freeze subset and host-boundary qualification rules

- **Head:** `3714ce27d6ea6381c0272084c49f695799bd2fda`.
- **Valuable:** BOOTSTRAP_SUBSET registry and state model; minimality rule; host capability
  policy; forbidden hidden inputs; canonical SemCode ordering; fixed-point definition;
  Bootstrap Seal; fail-closed capability behavior.
- **Obsolete:** B1–B5 / B0 / B7 references; SH-00 as authority; `BS-000` / `HC-000` placeholder
  rows; the name `HOST_CAPABILITY_ABI` (no ABI is frozen).
- **New destination:** [BOOTSTRAP_SUBSET.md](./BOOTSTRAP_SUBSET.md),
  [HOST_BOUNDARY.md](./HOST_BOUNDARY.md), [BOOTSTRAP_CONTRACT.md](./BOOTSTRAP_CONTRACT.md).
- **Recommended disposition:** close unmerged; content migrated; PR remains auditable provenance.

## 4. Documents changed by the reset

- `docs/BOOTSTRAP.md` → replaced by [QUALIFICATION.md](./QUALIFICATION.md).
- `README.md`, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md` rewritten around SHF.
- New: `BOOTSTRAP_CONTRACT.md`, `BOOTSTRAP_SUBSET.md`, `HOST_BOUNDARY.md`, `OWNERSHIP.md`,
  `FUTURE.md`, this file, `reference/semantic-reference.toml`.
