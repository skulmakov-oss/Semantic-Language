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
2. Git-tracked normative documents in this repository
3. Exact-SHA reference contracts from skulmakov-oss/Semantic
4. Accepted Git-tracked ADR / architecture decisions
5. Code and tests
6. Obsidian project brain
7. local-rag index
8. codebase-memory index
9. Session notes / historical logs
```

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

## 9. Working loop

1. Read current Git status and exact repository identity.
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
