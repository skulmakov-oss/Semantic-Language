# Imports v0.2

This page documents the import behavior currently implemented in EXOcode semantics.

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

## Resolve Behavior

1. Local symbols
2. Explicit select imports
3. Namespace-qualified access (`X.Foo`)
4. Wildcards by import declaration order

## Validation Rules

1. Duplicate namespace alias in one module is rejected (`E0241`).
2. Missing selected symbol is rejected (`E0244`).
3. Duplicate selected alias in one import statement is rejected (`E0245`).
4. `*` cannot be combined with `{...}` in one import statement (`E0245`).

## Examples

Valid:

```exo
Import "dep.exo" { Sensor, LawA as CheckA }
Law "Root" [priority 1]:
    When true -> System.recovery()
```

Invalid (`E0244`):

```exo
Import "dep.exo" { MissingSymbol }
Law "Root" [priority 1]:
    When true -> System.recovery()
```

Invalid (`E0245`):

```exo
Import "dep.exo" { A as X, B as X }
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
