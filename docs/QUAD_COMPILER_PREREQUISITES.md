# Native Quad compiler prerequisites

The clean-slate VM kernel in `vm/main.sm` is executable today because it uses a
nominal `Quad` record with two Boolean planes.  It does not claim that the
compiler's primitive `quad` type is available.

The minimum compiler work required before this VM can consume native Quad
values is:

1. Add canonical `N`, `F`, `T`, and `S` literal tokens and AST representation.
2. Add a primitive Quad type identity to `compiler/sema.sm` instead of treating
   `quad` as an unsupported annotation.
3. Add Quad expression typing and explicit truth-plane/falsity-plane
   operations; do not coerce Quad values to `bool`.
4. Extend `compiler/lower.sm` with a Quad value family and typed control
   lowering.
5. Extend `compiler/semcode.sm` only after the upstream SemCode opcode and
   revision contract is approved.
6. Add compiler fixtures for all four states and negative cases for implicit
   Boolean coercion.

Until those steps are qualified, the record-based kernel is the only honest
Semantic-native four-valued execution path.  Encoding Quad as an integer would
hide the state semantics and is intentionally not used.
