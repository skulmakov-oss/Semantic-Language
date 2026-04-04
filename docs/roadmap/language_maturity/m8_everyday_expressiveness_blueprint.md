# M8 Everyday Expressiveness Blueprint

Status: proposed future post-stable language-maturity blueprint

Related documents:

- `docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`
- `docs/roadmap/language_maturity/m8_everyday_expressiveness_phased_implementation_plan.md`

## Purpose

Define the architectural reading for the next Semantic language-maturity phase
after the stable `v1.1.1` line.

## Layer Model

### Layer A — Stable Semantic Core

Includes:

- syntax contract
- type contract
- records / ADT / match
- contracts
- schemas / validation / boundary core
- verifier / VM path

This layer remains canonical and must not be destabilized casually.

### Layer B — Everyday Language Expansion

Includes:

- text values and operations
- packages / manifests / dependency contract
- collections and iteration foundations
- first-class closure values

This layer is where immediate language-maturity work belongs.

### Layer C — General Abstraction Expansion

Includes:

- generics
- protocols / traits / interfaces
- broader abstraction carriers
- richer pattern semantics

This layer depends on Layer B becoming stable first.

### Layer D — Platform / Runtime Expansion

Includes:

- UI boundary
- broader runtime families
- concurrency
- extended orchestration semantics
- platform packaging and distribution forms

This layer is explicitly separate from language-maturity work.

## Dependency Order

Correct order:

1. preserve stable semantic core
2. strengthen everyday expressiveness
3. open general abstractions
4. open broader platform/runtime expansion

Discouraged anti-order:

- traits before generics are understood
- generics before text/data/package baseline exists
- UI before text/data/runtime ergonomics
- concurrency before package/data abstractions

## Design Doctrine

- Carrier before abstraction
- Narrow first-wave, then formal close-out
- Stable line honesty
- Optional means optional
- One active stream
- Platform boundary is not language syntax

## M8 Reading

The proposed M8 package should be read as one coherent language phase with four
ordered tracks:

1. text / strings
2. package ecosystem baseline
3. collections
4. first-class closures

Why package before collections:

- package/dependency contract should be explicit before broader language growth
  starts depending on file-only modularity

Why closures still belong in M8:

- they are part of ordinary language expressiveness, not only later
  abstraction machinery

## M9 Reading

M9 stays blocked until M8 core outputs are stable.

This is where it becomes reasonable to open:

- generics
- traits / protocols / interfaces
- iterable abstraction
- richer patterns

## M10 Reading

M10 remains a separate class of work.

If repository governance already carries a separately opened UI milestone, that
milestone stays the canonical platform-track entry point rather than being
silently absorbed by this language-maturity package.
