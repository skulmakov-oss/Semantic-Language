# Imports v0.2

This page is a companion guide to the current module/import surface.

The canonical public contract now lives in:

- `docs/spec/modules.md`
- `docs/spec/diagnostics.md`

This page should stay aligned with those source-contract documents and is
intended mainly as a compact guide with examples.

## Supported Forms

1. `Import "a/b/c"`  
Imports module namespace with default alias (derived from file stem).

2. `Import "a/b/c" as X`  
Imports module namespace with explicit alias.

3. `Import "a/b/c" { Foo, Bar as Baz }`  
Select import list. Each selected symbol must exist in imported module export set.

4. `Import "a/b/c" *`  
Wildcard import form is parsed and validated by policy rules.

5. `Import pub "a/b/c" { Foo }`  
Re-export selected symbols from dependency module.

Every current `Import` also creates one namespace alias:

- explicit alias from `as X`, or
- default alias from the imported file stem

## Resolve Behavior

1. Local symbols always win; conflicting import bindings are rejected with `E0241`.
2. Explicit select imports create direct local bindings.
3. Namespace-qualified access (`X.Foo`) stays available for every import alias.
4. Wildcards are fallback-only and are consulted by import declaration order.

Current clarification:

- `Import "dep.sm" { Foo }` binds both `Foo` and namespace alias `dep`
- `Import "dep.sm" *` still binds namespace alias `dep`
- explicit selected bindings outrank wildcard-provided names

## Validation Rules

1. Duplicate namespace alias in one module is rejected (`E0241`).
2. Missing selected symbol is rejected (`E0244`).
3. Duplicate selected alias in one import statement is rejected (`E0245`).
4. `*` cannot be combined with `{...}` in one import statement (`E0245`).

## Examples

Valid:

```exo
Import "dep.sm" { Sensor, LawA as CheckA }
Law "Root" [priority 1]:
    When true -> System.recovery()
```

Invalid (`E0244`):

```exo
Import "dep.sm" { MissingSymbol }
Law "Root" [priority 1]:
    When true -> System.recovery()
```

Invalid (`E0245`):

```exo
Import "dep.sm" { A as X, B as X }
Law "Root" [priority 1]:
    When true -> System.recovery()
```

## Related Errors

- `E0242`: see `docs/errors/E0242.md`
- `E0243`: see `docs/errors/E0243.md`
- `E0244`: see `docs/errors/E0244.md`
- `E0245`: see `docs/errors/E0245.md`

## Fixtures

Examples from this page are covered in `tests/fixtures/imports/`.
