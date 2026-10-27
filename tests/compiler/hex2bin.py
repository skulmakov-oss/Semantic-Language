#!/usr/bin/env python3
"""Transport-only adapter: decode the exact HEX produced by the Semantic emitter into bytes.

usage: hex2bin.py <hex-file> <out.smc>
It performs representation decoding only: no parsing of Semantic source, no opcode,
header, offset or metadata decisions, no repair. Any non-hex content is rejected.
"""
import sys

hex_text = open(sys.argv[1], encoding="ascii").read().strip()
if len(hex_text) % 2 or any(c not in "0123456789abcdef" for c in hex_text):
    sys.exit("hex2bin: input is not an even-length lowercase hex string")
open(sys.argv[2], "wb").write(bytes.fromhex(hex_text))
