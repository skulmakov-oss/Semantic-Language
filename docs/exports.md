# Exports v0.2

This page is a companion guide to the current export and re-export surface.

The canonical public contract now lives in:

- `docs/spec/modules.md`
- `docs/spec/diagnostics.md`

This page should stay aligned with those source-contract documents and is
intended mainly as a compact guide with examples.

## Exportable Items

The current export set includes top-level Logos declarations:

1. `System`
2. `Entity`
3. `Law`

## Re-export

Re-export is supported through `Import pub ...`:

1. `Import pub "dep.sm"`
2. `Import pub "dep.sm" { Foo, Bar as Baz }`
3. `Import pub "dep.sm" *`

Each exported item stores provenance:

1. `Local { module }`
2. `Imported { module, symbol }`
3. `ReExport { chain }`

## Deterministic Export Surface

Export ordering is deterministic by declaration order (`decl_order` ascending).

## Collision Policy

If two exports in one module publish the same public name, compilation fails with `E0242`.

## Symbol-level Cycle Policy

Re-export symbol cycles are detected and rejected with `E0243`, with a chain trace.

## Examples

Collision (`E0242`):

```exo
Import pub "a.sm"
Import pub "b.sm"
```

Cycle (`E0243`):

```exo
// a.sm
Import pub "b.sm"
// b.sm
Import pub "a.sm"
```

## Related Errors

- `E0242`: see `docs/errors/E0242.md`
- `E0243`: see `docs/errors/E0243.md`
- `E0244`: see `docs/errors/E0244.md`
- `E0245`: see `docs/errors/E0245.md`
