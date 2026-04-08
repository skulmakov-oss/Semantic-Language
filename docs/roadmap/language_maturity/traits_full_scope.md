# Traits Full Scope

Status: completed M9.2 post-stable subtrack
Related roadmap package:
`docs/roadmap/language_maturity/generics_full_scope.md`

## Goal

Introduce the first admitted static trait surface for Semantic without
opening runtime dispatch, trait objects, or advanced method resolution
ahead of schedule.

This is a forward-only language-maturity subtrack for current `main`. It is not
a claim that trait-based polymorphism was part of the published `v1.1.1` line.

## Decision Check

- [x] This is a new explicit post-stable track with its own scope decision
- [x] This does not silently widen published `v1.1.1`
- [x] This is one stream, not a mixture of multiple tracks
- [x] This can be closed with a clear done-boundary

## Done-Boundary

`M9.2 closes at static trait admission + coherence/conformance + bound satisfaction`

## Included In This Track

### Wave 1 — Owner Layer
- `TraitDecl`, `ImplDecl`, `TraitBound`, `TraitMethodSig` AST nodes
- `KwTrait` and `KwImpl` token kinds and lexer keyword map
- `trait_bounds: Vec<TraitBound>` field on `Function` and `FnSig`
- `TraitTable` and `ImplTable` public type aliases

### Wave 2 — Parser Admission
- `trait TraitName { fn method(params) -> ret; }` admitted at top level
- `impl TraitName for TypeName { fn method(...) { ... } }` admitted at top level
- `<T: TraitName>` bound syntax on function type parameters
- `Program.traits` and `Program.impls` fields
- `build_trait_table()` public API function

### Wave 3 — Typecheck
- `validate_trait_coherence`: rejects duplicate `(trait, for_type)` impl pairs
- `validate_impl_conformance`: rejects impls missing required methods or with wrong return types
- Bound satisfaction check at generic call sites: after type-var substitution,
  verifies `impl TraitName for ConcreteType` exists in the program's impl list
- All checks run centrally in `type_check_program`

## Explicit Non-Goals (out of scope)

- runtime dispatch / vtable-based dynamic dispatch
- trait objects (`dyn TraitName`)
- specialization (overlapping impls with precedence rules)
- advanced method resolution (inherent vs trait method precedence)
- blanket impls (`impl<T: Foo> Bar for T`)
- associated types or type families
- default method implementations in trait bodies
- negative impls
- silent widening of published `v1.1.1`

## Test Coverage

- coherence: duplicate `(trait, for_type)` pair is rejected
- conformance: impl missing a required method is rejected
- conformance: impl method with wrong return type is rejected
- bound satisfaction: generic call with satisfying impl typechecks
- bound satisfaction: generic call with no impl is rejected

## Wave Order

```
W0 governance   → scope decision recorded
W1 owner layer  → AST nodes, TokenKind, type aliases
W2 parser       → trait/impl admitted at top level, bound syntax
W3 typecheck    → coherence, conformance, bound satisfaction
W4 freeze       → changelog, milestone, scope doc marked complete
```
