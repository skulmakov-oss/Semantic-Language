# Changelog — Semantic-Language

Dates are UTC. Identities are SHA-256 unless stated otherwise. Entries summarize merged pull requests; the pull requests are authoritative.

## 2026-11-25 — Self-Hosting Milestone (Bootstrap Contract v4)

- **#66: C0 code-generation parity and the self-host workflow.**
  - The compiler emits SemCode byte-identical to the pinned C0 (`1d1e76c7`).
  - `tests/selfhost/` adds a reproducible workflow (`selfhost.py`) and an independent fail-closed checker (`check_results.py`).
  - Review fixes:
    - unit-valued `if` lowering;
    - return-path proof for `match` with `_`;
    - statement-position `match` proofs;
    - `--generations` choices.
- **#67, #68: governance envelope for the v4 payload.**
  - #67 (G1) opened the envelope for the v4 payload.
  - #68 (G1b) added the C0 reference-identity check.
- **#69: Bootstrap Contract v4** (`shf0-bootstrap-contract-v4`).
  - Canonical source set `bootstrap/source-set.toml`.
  - Stored-artifact executor: upstream `smc run-smc --expect-sha256` at `skulmakov-oss/Semantic@825deca4` (skulmakov-oss/Semantic#2086).
  - Trusted-compiler envelope: 150M steps / 2M calls / 256 frames.
  - v4 evidence fields.
  - Seal states `validated -> reproduced -> accepted` (`check_seal`).
- **#70: ordinary engineering envelope restored.**
- **Normative qualification** on a fresh checkout of `8d71cc3`:
  - `S` = `c9e0cb33…148f`;
  - C1 = C2 = C3 = `2bfd56e1…e939` (339,746 B);
  - 101,176,981 steps / 1,092,148 calls / 36 frames;
  - 140/140 fixtures;
  - evidence record valid; Seal candidate **validated**.
- **Not done:** independent reproduction, human review, Seal acceptance, cross-platform qualification (SHF-17).

## 2026-11-14 — Quad-native main compatibility (#63) and C2 = C3 = C4 engineering checkpoint

- Historical lineage (C2 = C3 = C4 = `120ed3c1…`, 301,087 B, with C1 != C2). Superseded by the 2026-11-25 milestone.

## Earlier

- Bootstrap Contract v1 to v3, compiler foundation and SHF-1 to SHF-14 implementation work: see `docs/ROADMAP.md`, `docs/BOOTSTRAP_CONTRACT.md` §10 and the merged pull requests.
