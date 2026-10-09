# Semantic-Language — agent operating contract

This repository (`skulmakov-oss/Semantic-Language`) is the **bootstrap implementation repository**:
it moves the Semantic compiler from the Rust-hosted reference toward a compiler written
substantially in Semantic. Read `README.md`, `CONTRIBUTING.md` and `docs/` before any work.

## 1. Repositories and paths

| Role | Repository | Local path |
|---|---|---|
| Bootstrap (active workspace) | `skulmakov-oss/Semantic-Language` | `C:\Users\said3\Desktop\Semantic-Language` |
| Reference / oracle / C0 | `skulmakov-oss/Semantic` | `C:\Users\said3\Desktop\Semantic-Language-local\OLD_RUST_SEMANTIC_LANGUAGE` |
| Project brain (Obsidian) | — | `D:\Obsidian\Knowledge\Semantic-Brain` |
| Mutable local tool state | — | `C:\Users\said3\Desktop\Semantic-Language-local\` |

`OLD_RUST_SEMANTIC_LANGUAGE` = local convenience checkout / reference material.
It has no authority by directory identity.
Any qualification oracle must be identified by: `skulmakov-oss/Semantic` + exact Git SHA.
Before using it, verify its HEAD equals the required SHA. On mismatch: report; do not silently
qualify against it, do not pull/checkout/reset it; use an isolated worktree or an explicitly
authorized update.

Local tool state (lancedb, reference checkouts, scratch) never lives inside this source tree.

## 2. Authority hierarchy

```text
1. User / repository-owner explicit instruction
2. Active Harness task envelope (.harness/current.task.yaml)
3. Git-tracked normative documents in this repository
4. Exact-SHA reference contracts from skulmakov-oss/Semantic
5. Accepted Git-tracked ADR / architecture decisions
6. Code and tests
7. Obsidian project brain
8. local-rag index
9. codebase-memory index
10. Session notes / historical logs
```

The Harness envelope narrows a task; it never widens the architecture authority of items 3–5.
Owner GO may authorize a controlled Harness transition; agent convenience may not.

**Indexes are retrieval tools, not sources of truth.**
- local-rag or codebase-memory disagrees with Git-tracked content → Git wins.
- Obsidian disagrees with accepted Git architecture → Git wins.
- Local reference checkout disagrees with the pinned reference SHA → the pinned SHA wins.
- Floating upstream `main` is never a qualification oracle.

## 3. Self-hosting scope

Upstream authority: `skulmakov-oss/Semantic#1910`.

```text
S = compiler source written substantially in Semantic
C0(S) -> C1.smc
C1(S) -> C2.smc
Canonical(C1.smc) == Canonical(C2.smc)
```

First self-hosting does NOT require rewriting: sm-verify, sm-vm, PROMETHEUS, native backend,
UI, Workbench, Studio.

`skulmakov-oss/Semantic#1909` (Native Reasoning / Full Sigma + t¤) is a post-self-hosting track.
It is not a prerequisite for #1910; do not put its plan into the bootstrap work queue.

### Where a change belongs

Before implementing a missing capability (Text bytes, Bytes, u32 ops, Map iteration, generics,
binary I/O, …) ask: *does this change the Semantic language/runtime contract?*
- YES → belongs upstream in `skulmakov-oss/Semantic`, qualified there first.
- NO, ordinary compiler logic (lexer, parser, AST, analysis, IR, lowering, SemCode emitter,
  bootstrap harness, C1/C2 comparison) → belongs here.

## 4. Superpowers (process)

Use the smallest relevant process; do not invoke skills mechanically.

```text
new architecture / feature      -> superpowers:brainstorming
accepted direction              -> superpowers:writing-plans
implementation                  -> superpowers:test-driven-development
bug / unexpected failure        -> superpowers:systematic-debugging
before claiming completion      -> superpowers:verification-before-completion
before branch completion        -> superpowers:requesting-code-review
                                   superpowers:finishing-a-development-branch
```

Architecture tasks never jump directly from idea to implementation.

## 5. codebase-memory-mcp (code graph index)

Project identity: `C-Users-said3-Desktop-Semantic-Language` (this repo only).
The reference repo is NOT indexed into this graph; inspect it via Git/source, or under its own
separate project identity.

- Check `list_projects` / `index_status` before relying on results.
- After substantial code changes: `detect_changes` or `index_repository`.
- Use `search_graph`, `get_code_snippet`, `trace_path`, `get_architecture`, `query_graph`,
  `search_code` to understand structure — not to establish normative semantics.
- For exact current behavior, read the Git-tracked source.
- `manage_adr` only for accepted decisions, never for speculative brainstorming.
- Docs/config: plain Read/Grep/Glob/Git and local-rag are fine.

## 6. mcp-local-rag (document index)

Use the project-local server `local-rag-semantic-language` (Claude Code `local` scope for this
repo; `BASE_DIR = D:/Obsidian/Knowledge/Semantic-Brain`,
`DB_PATH = C:/Users/said3/Desktop/Semantic-Language-local/lancedb`,
`CACHE_DIR = C:/Users/said3/Desktop/Semantic-Language-local/models`). Do not ingest into the
global `local-rag` server; it belongs to other projects.

Tools: `query_documents`, `ingest_file`, `ingest_data`, `status`, `list_files`, `delete_file`.

- Before architecture work: `query_documents` for existing specs, decisions, terminology.
- local-rag is an index, not authority: verify important results against the Git/Obsidian source.
- Re-ingest of the **identical path string** REPLACES the document (verified 2026-10-07).
  A differently spelled path to the same file (`\` vs `/`) creates a DUPLICATE.
  Always use the canonical form `D:/Obsidian/Knowledge/Semantic-Brain/<folder>/<name>.md`.
- Very short documents or chunks may be ignored by the current local-rag implementation.
  Do not depend on an exact minimum length unless verified from the implementation/configuration.

### Ingestion policy (Obsidian)

```text
Decisions/   -> YES
Specs/       -> YES
Knowledge/   -> YES
Sessions/    -> only curated, important conclusions
Inbox/       -> NO (until triaged)
Templates/   -> NO
attachments  -> NO unless specifically useful
```

Never ingest the whole machine, and never ingest `D:\ClaudeMemory\log.md` without authorization.

## 7. Obsidian project brain

Vault: `D:\Obsidian\Knowledge\Semantic-Brain` (plain .md; write directly). One vault only.

Obsidian = thinking + project memory + decision context + research + session continuity.
It is NOT the normative specification. Canonical architecture, bootstrap contract,
qualification protocol, ownership, normative compiler contracts and release-affecting decisions
live in Git.

Lifecycle: `Inbox → research → Specs/Decisions → owner acceptance → Git-tracked doc / ADR`.
Never leave a load-bearing compiler rule only in Obsidian.

| Folder | Content | Statuses |
|---|---|---|
| `Inbox/` | raw thoughts; triage or delete | — |
| `Sessions/YYYY-MM-DD.md` | done, key evidence (exact SHAs), decisions, blockers, next authorized step | — |
| `Decisions/` | project/architecture decisions (`Templates/Decision.md`) | proposed / accepted / superseded / rejected |
| `Specs/` | design exploration, drafts (`Templates/Spec.md`) | draft / review / accepted / superseded |
| `Knowledge/` | terminology, gotchas, tool behavior, qualification lessons — nothing speculative | — |

Global log `D:\ClaudeMemory\log.md` = compact cross-project memory (date, repo, PR/SHA,
decision, next state, critical gotcha). Detailed reasoning stays in Sessions/Decisions.
Do not duplicate.

### Memory write discipline

Write persistent memory only when it is architecturally important, hard to rediscover,
likely to affect future decisions, a proven gotcha, or a durable workflow constraint.
At the end of meaningful work:
1. Update the project Session note.
2. Stable lessons → `Knowledge/`; accepted choices → `Decisions/`; normative ones → Git docs.
3. Re-ingest only changed curated notes into local-rag.
4. Re-index codebase-memory only if code structure changed.
5. Global log entry only for meaningful milestones.

No memory writes after trivial reads or formatting changes.

## 8. Git / PR discipline

- Never work on `main`; use a narrow branch. Before branching: `git fetch origin`,
  `git status`, `git rev-parse origin/main`; require a clean tree, never overwrite unknown changes.
- Small, auditable changes. Before claiming ready:
  `git status`, `git diff --check`, `git diff --stat`, `git diff`.
- No merge without explicit owner GO. Never automatically merge, close issues, delete branches,
  rebase unrelated work or rewrite history. Force push only `--force-with-lease`, only after an
  authorized rebase.

### Harness (per-task authorization envelope)

`.harness/current.task.yaml` names the single active task, its allowed/forbidden paths and its
capability flags. `scripts/harness-check.ps1` enforces it locally (staged, unstaged and untracked
files) and in CI against the exact PR base (`-BaseRef`). Forbidden always wins over allowed;
a missing or malformed envelope fails closed.

An agent may NOT broaden or replace the active Harness merely because its task is blocked.
When a task needs different paths or capabilities:

1. STOP;
2. report the blocker;
3. obtain explicit owner authorization for the named task;
4. change `.harness/current.task.yaml` in its own envelope-only PR, reviewed and merged by
   the owner;
5. only then make the newly authorized changes, in a later PR.

CI enforces this: in `-BaseRef` mode a change to the envelope's scope (task id, allowed/forbidden
paths, authorization flags) relative to the base envelope fails unless the PR touches nothing
else, so a PR can never authorize its own payload. Envelope transition PRs record their exact base in
`constraints.base_sha`; for ordinary tasks operating under an unchanged envelope, `constraints.base_sha`
records historical provenance and does not require per-PR bumping when `main` advances.

Trust root: `.github/workflows/harness-trusted.yml` (`pull_request_target`, job
`scope-enforcement`) runs as it exists on the base branch, executes only the base revision's
`scripts/harness-check.ps1`, and inspects the PR head as data only (read-only token, no secrets,
no PR code executed). A PR may propose changes to it, but they are judged by the previous trusted
version. Ordinary task envelopes must explicitly list `scripts/harness-check.ps1`, `.github/**`,
and `.harness/current.task.yaml` under `forbidden_paths`; only an owner-authorized governance task may allow them.

The Harness is never a mechanism for self-authorization. Bootstrap fact: the PR that introduced
the Harness (issue #14) added envelope, checker, CI enforcement and the trusted workflow together
under explicit owner GO. Because the trusted workflow did not yet exist on its base, that one PR was
qualified by exact-head owner review, CI and independent review instead. That exception ended at
its merge; no later PR may claim it. There is no bypass.

Under Strategy v1.1 (Issue #20: Implementation Independence, Specification Unity, Deterministic Proof),
ordinary SHF tasks are authorized for full-stage delivery (compiler code, tests, and documentation in a
single PR cycle within the active envelope) rather than fragmented micro-PRs. However, governance invariants
remains strictly fail-closed: ordinary tasks cannot touch `.github/**` or `scripts/harness-check.ps1`,
and C0 remains a reproducible historical executable reference evaluated by deterministic fixed-point proof.

### Stable ordinary SHF delivery profile & Surface Classification

Post-migration operations are organized into four distinct surfaces:

1. **Stable ordinary engineering surfaces (A):**
   `compiler/**`, `tests/**`, `docs/**`, `README.md`, `CONTRIBUTING.md`.
   Ordinary SHF delivery occurs within these surfaces. No envelope transition PR is needed between
   normal SHF stages; the active task/stage is authorized by owner instruction and tracked in the
   relevant Issue and PR.
2. **Protected governance/trust surfaces (B):**
   `.github/**`, `scripts/harness-check.ps1`, `.harness/current.task.yaml`.
   Permanently forbidden to ordinary tasks. An ordinary task envelope must explicitly list these in
   `forbidden_paths` and cannot include them in `allowed_paths`. Any change to these surfaces requires
   an owner-authorized governance task (`task.type: governance` or `governance_migration`) via an
   envelope transition PR.
3. **Reference / C0 identity changes (C):**
   `reference/**`.
   Changes to reference pins or manifests require explicit owner authority and drift analysis.
4. **Release and Bootstrap Seal surfaces (D):**
   `bootstrap/**`.
   Changes to the frozen bootstrap contract protocol or seal generation require dedicated authorization.

### Envelope transition criteria

An envelope transition PR (`envelope-only PR -> payload PR`) is NOT a routine per-SHF requirement.
It is required exclusively for:
- Alterations to protected governance/trust surfaces (`.github/**`, `scripts/harness-check.ps1`);
- Re-scoping task types or modifying permanently protected paths;
- Reference/C0 identity changes (`reference/**`) or bootstrap contract changes (`bootstrap/**`).

Normal SHF implementation, tests, bugfixes, and documentation inside the stable engineering boundary
proceed directly without recurring governance transition PRs.

### One-Time G2 Closeout Plan

Issue #20 migration is structured in distinct stages:
- **Phase G0 (Complete):** Landed envelope `GOVERNANCE-MIGRATION-ISSUE-20` on `main` (#21).
- **Phase G1 (Active PR #22):** Implements invariant checker protection, regression test suite, CI wiring, and normative document reconciliation.
- **Phase G2 (One-Time Closeout):** Following merge of PR #22 under owner GO, a final envelope-only PR transitions `.harness/current.task.yaml` into the stable ordinary SHF delivery profile (`task.type: implementation`). G2 is a one-time migration closeout, not a recurring per-SHF workflow. Issue #20 remains open until G2 is merged.

## 9. Working loop

1. Read current Git status, exact repository identity and `.harness/current.task.yaml`.
2. Read relevant recent project Session/Decisions.
3. Search curated docs with local-rag.
4. Inspect current code with codebase-memory/source.
5. Verify the exact Semantic reference SHA for any oracle claim.
6. Use Superpowers for design/plan/TDD/debug/verification as appropriate.
7. Make the smallest authorized change.
8. Run exact validation.
9. Update only durable project memory.
10. Stop at the owner-review boundary; do not merge without GO.

## 10. Forbidden

- Do not treat floating upstream `main` as a qualification oracle.
- Do not treat `OLD_RUST_SEMANTIC_LANGUAGE` as canonical by directory name.
- Do not use Obsidian as a substitute for normative Git documentation.
- Do not treat local-rag or codebase-memory output as current truth without verifying source.
- Do not silently invent Semantic semantics to make bootstrap easier.
- Do not hide missing compiler capability in Rust bootstrap glue.
- Do not mix #1909 Native Reasoning into #1910 self-hosting.
- Do not begin Instant Pipeline, SRI, UI, VM rewrite or verifier rewrite as a prerequisite for
  first self-hosting.
- Do not modify both repositories in one task unless explicitly authorized.
- Do not merge without owner GO.
