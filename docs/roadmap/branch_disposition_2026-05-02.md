# Branch Disposition Report — 2026-05-02

Status: closed  
Scope: all `codex/*` remote branches  
Author: M-Tail T0

## Summary

| Category | Count |
|----------|-------|
| Branches before sweep | 215 |
| Deleted — merged into main (commit ancestry) | 183 |
| Deleted — squash-merged / superseded | 32 |
| Remaining `codex/*` branches | **0** |
| Non-`codex` branches (retained, see below) | 4 |

All `codex/*` remote branches have been deleted.

## Deletion Details

### Group A — Merged (183 branches)

All branches where `git branch -r --merged origin/main` returned a match.
Content is fully present in main commit history. Deleted without review.

Prefix patterns covered:
- `codex/a*`, `codex/b*`, `codex/c*`, `codex/d*`, `codex/e*`, `codex/f*`
- `codex/d20-*`, `codex/d20p-*`
- `codex/doc-*`
- `codex/enable-*`, `codex/executable-*`, `codex/fr-00-*`, `codex/fr-01-*`, `codex/fr-02-*`
- `codex/gap-i32-*`, `codex/gap-match-*` (merged variants)
- `codex/ir-*`, `codex/iterable-*`
- `codex/local-*` (merged)
- `codex/lower-*`
- `codex/m7-*`, `codex/m8-*` (merged)
- `codex/m9-10-*`
- `codex/narrow-*`, `codex/node24-*`
- `codex/option-result-*`
- `codex/pr10-*` through `codex/pr19-*`, `codex/pr2-*` through `codex/pr9-*`,
  `codex/pr23-*` through `codex/pr41-*` (merged PRs)
- `codex/preserve-*`
- `codex/record-*`, `codex/release-*`, `codex/runtime-*` (merged)
- `codex/schema-*`, `codex/semcode-*`, `codex/source-*`
- `codex/trait-*`
- `codex/ui-milestone-*`, `codex/units-*`, `codex/update-*`
- `codex/v03-01-*`
- `codex/verify-record-*`, `codex/vm-*`
- `codex/wb-01-*` through `codex/wb-13-*`, `codex/wb-bh-*` (merged workbench waves)
- `codex/wiki-*`, `codex/workbench-*`, `codex/write-*`

### Group B — Squash-merged / superseded (32 branches)

These branches showed as "not merged" by commit ancestry but their content
was confirmed present in main via squash-merge PRs or was superseded:

| Branch | Disposition | Evidence |
|--------|-------------|----------|
| `codex/core-capsule-import` | Superseded | PR #385 landed equivalent content from main repo |
| `codex/core-capsule-workspace-green` | Superseded | PR #385 |
| `codex/fr-03-stdlib-v0` | Historical docs | Planning scope, milestone not started |
| `codex/fr-04-project-model-v0` | Historical docs | Planning scope |
| `codex/fr-05-06-execution-runtime-closure` | Historical docs | Planning scope |
| `codex/fr-07-09-examples-onboarding-release` | Historical docs | Planning scope |
| `codex/gap-canonical-weather-station-example` | Historical docs | Gap scope doc |
| `codex/gap-comment-lexing-robustness` | Historical docs | Gap scope doc |
| `codex/gap-diagnostic-span-accuracy` | Historical docs | Gap scope doc |
| `codex/gap-f64-relational-surface` | Historical docs | Gap scope doc |
| `codex/gap-line-ending-normalization` | Historical docs | Gap scope doc |
| `codex/gap-match-terminal-cfg` | Historical docs | Gap scope doc |
| `codex/gap-record-executable-readiness` | Historical docs | Gap scope doc |
| `codex/local-unstaged-tail-snapshot` | Historical snapshot | Old unstaged tail |
| `codex/m7-wave3` | Historical | M7 milestone closed |
| `codex/m7-wave4` | Historical | M7 milestone closed |
| `codex/m8-planning-sync` | Historical | M8 milestone closed |
| `codex/m9-1-generics-wave3` | Squash-merged | Content in PR #262 |
| `codex/m9-2-traits-wave0` | Superseded | Superseded by wave1 |
| `codex/m9-2-traits-wave1` | Squash-merged | Content in PRs #260, #303 |
| `codex/m9-4-patterns-wave0` | Superseded | Superseded by wave1 |
| `codex/m9-4-patterns-wave1` | Squash-merged | Content in PR #262 |
| `codex/pr-build-stdlib-surface` | Historical docs | Planning scope |
| `codex/pr-freeze-source-language-contract` | Historical docs | Planning scope |
| `codex/pr-package-ecosystem-story` | Historical docs | Planning scope |
| `codex/pr24-schema-scope-baseline` | Superseded | See T1 in M-Tail |
| `codex/pr41-selected-import-wave` | Squash-merged | Content in PR #341 |
| `codex/runtime-ownership-contract-v0` | Superseded | Replaced by later ownership work |
| `codex/ui-application-boundary-roadmap` | Historical docs | POST-UI milestone not yet started |
| `codex/v03-02-ir-lowering-text-sequence-closure` | Squash-merged | PR #387 |
| `codex/verify-ownership-runtime-core-base` | Squash-merged | Content in PRs #267, #270 |
| `codex/workbench-beta-hardening-backlog` | Historical docs | Workbench Beta milestone closed |

## Retained Non-`codex` Branches

These branches are outside the T0 scope and are retained:

| Branch | Reason |
|--------|--------|
| `origin/main` | Active default branch |
| `origin/release/v0` | Historical v0 release tag anchor |
| `origin/add-license` | Historical license addition; harmless |
| `origin/dev/v1-math` | Historical early dev branch; harmless |

## Verification

```
git branch -r | grep "origin/codex" | wc -l
# → 0
```

## Notes

- Local worktree cleanup was performed separately on 2026-05-02:
  117 Desktop/EXOcode/EXOcode-* and EXOcode_* worktrees removed via
  `git worktree remove --force` + `git worktree prune`.
- `codex/pr24-schema-scope-baseline` is addressed under M-Tail T1.
- No code behavior was changed by this sweep.
