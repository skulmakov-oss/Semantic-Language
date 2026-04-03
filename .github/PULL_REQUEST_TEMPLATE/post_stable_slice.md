# <TRACK-ID> <Short Slice Name>

Closes part of: <issue link or track id>
Slice type: <docs-only | code | freeze>
One PR = one logical step.

## What This PR Does

- <change 1>
- <change 2>
- <change 3>

## What This PR Does Not Do

- <out-of-scope 1>
- <out-of-scope 2>
- <out-of-scope 3>

## Stable Boundary Statement

Published `v1.1.1`:

- <unchanged stable reading>

Current `main` after this PR:

- <new admitted reading>

This PR is:

- [ ] release-maintenance only
- [ ] forward-only widening on `main`
- [ ] not a retroactive widening of published stable

## Files / Ownership

Owner crates or docs touched:

- <crate/doc 1>
- <crate/doc 2>
- <crate/doc 3>

## Verification

- [ ] `cargo test --workspace`
- [ ] `cargo test --test public_api_contracts`
- [ ] extra focused tests:
- [ ] docs/spec updated if contract changed

Exact commands run:

```powershell
<command 1>
<command 2>
<command 3>
```

## Follow-Up

Next honest step after this PR:

- <next narrow slice>
