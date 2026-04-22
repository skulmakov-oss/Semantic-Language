# Package Worked Example

Status: historical design note, not current baseline

## Purpose

This document gives one end-to-end example of a future package-manager design
direction beyond the landed first-wave package baseline.

Current-main truth:

- current `main` already has a narrower admitted package baseline centered on
  `Semantic.package` and deterministic local-path dependency loading
- this example does not describe the landed baseline; it is retained only as a
  future illustrative design note

It connects:

- `Semantic.toml`
- `Semantic.lock`
- dependency aliases
- source imports
- deterministic resolution

The point is to show one coherent user-facing flow instead of leaving the
package documents as separate abstractions.

## Example Layout

The example workspace contains three packages:

- `access-policy` as the executable root
- `policy-core` as a published dependency
- `mathx` as a local path dependency

Illustrative layout:

```text
workspace/
  access-policy/
    Semantic.toml
    Semantic.lock
    src/main.sm
  mathx/
    Semantic.toml
    src/lib.sm
```

## Root Manifest

`access-policy/Semantic.toml`

```toml
[package]
name = "access-policy"
version = "0.1.0"
edition = "v1"
entry = "src/main.sm"

[dependencies]
mathx = { path = "../mathx" }
policy_core = { version = "^0.2.0" }
```

Meaning:

- `mathx` is a local alias bound to a path dependency
- `policy_core` is a package alias bound to a versioned published dependency

## Lockfile

`access-policy/Semantic.lock`

```toml
version = 1
root = "access-policy"

[[package]]
name = "access-policy"
version = "0.1.0"
source = "path:."

[[package]]
name = "mathx"
version = "0.1.0"
source = "path:../mathx"

[[package]]
name = "policy-core"
version = "0.2.3"
source = "registry:semantic"
checksum = "sha256:abc123"
```

Meaning:

- the manifest requirement `^0.2.0` has been concretized to `0.2.3`
- the local path dependency is pinned explicitly as a path source
- the package graph is reproducible without reinterpretation

## Source Imports

`access-policy/src/main.sm`

```sm
Import "mathx/stats" as math
Import "policy_core/rules" as rules

fn main() -> quad {
    let score: f64 = math.normalize(0.61, 0.0, 1.0);
    return rules.allow_score(score);
}
```

Meaning:

- source imports still use module-path syntax
- package aliases expose import roots to the module resolver
- package metadata and source imports stay distinct but connected

## Resolution Walkthrough

The intended resolver flow is:

1. read `Semantic.toml`
2. validate dependency aliases and requirements
3. read `Semantic.lock`
4. resolve alias `mathx` to local source root `path:../mathx`
5. resolve alias `policy_core` to concrete published package `policy-core@0.2.3`
6. expose those roots to the existing module resolver
7. resolve `"mathx/stats"` and `"policy_core/rules"` within the selected roots

This preserves the current import model while giving it a deterministic package
context.

## Why This Example Matters

This single example proves the package layer is not a replacement for modules.

It shows:

- manifest intent
- lockfile pinning
- alias-to-root mapping
- ordinary source imports on top of the resolved graph

Without this worked example, the ecosystem documents risk sounding like
parallel plans rather than one executable design.

## Non-Goals

This example intentionally does not define:

- remote registry protocol
- workspace manifests
- dev-dependencies
- target-specific dependency tables

## Cross-References

This example depends on:

- `docs/roadmap/language_maturity/package_ecosystem.md`
- `docs/roadmap/language_maturity/package_manifest.md`
- `docs/roadmap/language_maturity/package_lockfile.md`
- `docs/roadmap/language_maturity/dependency_resolution.md`
