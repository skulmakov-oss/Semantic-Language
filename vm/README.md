# Quad VM v2 kernel

`main.sm` is the clean-slate execution kernel for the experimental Quad VM.
It intentionally starts above the legacy SemCode decoder: the machine owns a
typed `Quad` value represented by independent truth and falsity planes, an
instruction record, and an immutable state transition per step.

The four states are:

| State | truth | falsity |
|---|---|---|
| N | false | false |
| F | false | true |
| T | true | false |
| S | true | true |

`quad_branch4` returns a distinct branch tag for every state. `quad_assert`
returns success, false, unknown, and conflict outcomes separately. Unknown and
conflict never become Boolean success. The executable `main` function covers
all four states, joins/meets, and a deterministic three-instruction engine run.

This checkpoint does not claim SemCode compatibility. The upstream opcode and
revision contract must be qualified before replacing the local instruction
record with a SemCode loader.
