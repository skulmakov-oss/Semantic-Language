# Module Selected-Import Settlement

Status: draft text artifact for `PR-B0.1`

## Why This Example Exists

This program models a small settlement helper split where two helper modules
both export a function called `status_text`.

The natural source shape wants symbol-level import plus aliasing:

- one helper provides policy-facing classification
- another helper provides presentation-facing rendering

This example is intentionally outside the current admitted executable contour.
It exists only as decision input for the next module-authoring widening wave.
