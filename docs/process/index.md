# Semantic Process Index

Status: active project-governance baseline

This directory defines the operating discipline for repository changes.

Use these documents as the canonical process layer for new work:

- `project_discipline.md` - non-negotiable project rules
- `change_traceability.md` - required documentation trail for requests, milestones, and PRs
- `pr_and_merge_policy.md` - PR readiness, test expectations, and merge gate
- `semantic_core_capsule_pr_program.md` - canonical PR package set for the execution-core program
- `semantic_core_capsule_audit_matrix.md` - current execution-core status audit against the PR program

Process rule:

- if code, behavior, release claims, or milestone status changes, update the relevant process trail in the same step
- if a change is truly document-only, keep the scope narrow and do not claim behavioral impact
