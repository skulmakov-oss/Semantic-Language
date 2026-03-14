# Semantic Core Spec Bundle

Status: draft v0

This directory is the canonical specification bundle for the current core
execution contract.

Current documents in this PR:

- `semcode.md` - SemCode binary contract and compatibility rules
- `profile.md` - `ParserProfile` policy contract
- `verifier.md` - SemCode admission verification contract
- `vm.md` - Semantic VM public execution contract
- `quotas.md` - runtime quota taxonomy and enforcement contract

Later PRs may extend this bundle with source-surface, IR, CLI, versioning, and
PROMETHEUS-boundary specifications.

Contract precedence:

1. `docs/spec/*` defines the public contract.
2. Code must implement that contract.
3. Architecture and roadmap documents constrain ownership and sequencing around
   that contract.

Blocking rule:

- any public change to SemCode admission, VM execution, quota semantics, or
  `ParserProfile` policy must update the relevant file in this directory in the
  same change series.
