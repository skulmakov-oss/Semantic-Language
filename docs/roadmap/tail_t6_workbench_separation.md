# M-Tail T6 — Workbench Branch/Backlog Separation

Status: closed  
Date: 2026-05-02

## Scope

Verify that workbench development history is cleanly separated from core
Semantic language development, and that no workbench branches remain in an
ambiguous state.

## Branch Disposition

All workbench development branches were handled in M-Tail T0
(branch disposition sweep, 2026-05-02):

| Branch pattern | Count | T0 Group | Disposition |
|---|---|---|---|
| `codex/wb-01-*` through `codex/wb-13-*` | 13 waves | A — merged | Deleted, content in `main` |
| `codex/wb-bh-*` | 1 hardening wave | A — merged | Deleted, content in `main` |
| `codex/workbench-beta-hardening-backlog` | 1 planning doc | B — superseded | Deleted, Workbench Beta milestone closed |

No workbench `codex/*` branches remain on `origin`.

## Backlog Separation

Workbench scope and backlog documentation lives exclusively in
`docs/workbench/`:

- `docs/workbench/architecture.md`
- `docs/workbench/scope.md`
- `docs/workbench/beta_packaging.md`
- `docs/workbench/beta_release_notes.md`

These docs are correctly separated from `docs/roadmap/` (core Semantic language
roadmap) and `docs/spec/` (language specification). No workbench scope
has leaked into the language roadmap or spec.

## Milestone Boundary

- **Workbench Beta milestone**: closed. All wb-wave branches merged and
  deleted.
- **POST-UI milestone**: not started. Tracked separately in
  `codex/ui-application-boundary-roadmap` (deleted as historical planning scope
  in T0). Future work will open new branches when the milestone is ready.

## Conclusion

The workbench branch/backlog is cleanly separated from the core language
development track. No ambiguous workbench branches remain.

T6 is closed. No further action required.
