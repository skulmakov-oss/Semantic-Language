# Instruction Set

The public instruction set covers:

- load operations for `unit`, `quad`, `bool`, `i32`, `u32`, and `fx`
- quad operations for negation, join, meet, implication, equality, and state predicates
- boolean logic and comparisons
- signed integer arithmetic and comparisons
- unsigned integer arithmetic and comparisons
- fixed-point arithmetic and comparisons
- register move
- jump, conditional jump, call, and return
- assert and explicit trap

The typed `Instr` format is allocation-free per instruction, size-frozen at 12 bytes by a compile-time assertion, and uses stable opcode names.
