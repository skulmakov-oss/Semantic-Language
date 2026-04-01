# Root Legacy Cleanup To FULL

Status: proposed post-stable closure track

## Goal

Bring root-level legacy compatibility down to one explicit perimeter:

- root shim library
- root process entrypoints
- legacy `ton618_core` compatibility bin

Everything outside that perimeter should live under canonical crate owners.

## Why This Is A Post-Stable Track

The repository already shipped stable `v1.1.1` with the current root layout.

The remaining work here is not new language surface. It is a repository hygiene
and ownership closure track intended to:

- make root inventory explicit
- prevent backend logic from drifting back into `root/src`
- keep compatibility shims narrow and reviewable
- strengthen CI guards around that policy

## In Scope

`NEXT-3` may include only:

- final inventory of `root/src`
- exact allowlist policy for root shims and bins
- tests/CI guards that reject new root legacy creep
- documentation sync for canonical owners versus compatibility perimeter
- removal or migration of root legacy Rust sources that are outside the final
  allowlist

## Out Of Scope

This track must not silently expand into:

- new CLI redesign
- package/runtime/`prom-*` widening
- new public language features
- broad crate renaming
- compatibility-bin removal without an explicit deprecation decision

## Intended Slice Order

1. docs/governance checkpoint
2. exact root inventory guard tightening
3. migration or removal of any remaining non-allowlisted root legacy sources
4. CI/checklist freeze for root cleanliness policy

## Acceptance Reading

`NEXT-3` is done only when:

- `root/src` contains only the explicit shim/binary allowlist
- compatibility-named root code is visibly narrow
- CI guards fail on any new root legacy expansion
- repository docs describe the root perimeter honestly

## Non-Goal Reminder

This is a closure pass over repository ownership boundaries, not a new product
surface wave.
