#!/usr/bin/env python3
"""Differential qualification of the Semantic VM (vm/main.sm) against the reference VM.

Environment:
  SEMANTIC_SMC       required. smc-cli of the reference (oracle) build: compiles the corpus,
                     admits every artifact (sm-verify), runs it (sm-vm), and runs the
                     Semantic VM itself as an ordinary Semantic program. Its SHA-256 and
                     version are recorded; only an exact reference SHA is an oracle.
  SEMANTIC_C0_SMC    optional. smc-cli of the pinned C0; artifacts it admits are also run
                     on it and recorded (never judged: the corpus targets the oracle build).
  SEMANTIC_TEST_OUT  output directory, outside the source tree (default: $TMPDIR/semantic-vm-tests).
  SEMVM_SKIP_MUTATIONS=1  skip the VM mutation pass.

Per corpus case:
  1. compile with the oracle; record the artifact SHA-256; admit it with the oracle verifier;
  2. run it on the oracle VM (run-smc): exit code, printed lines, error message;
     and record the oracle's own guest counters (smc run --metrics json: steps, calls, frames);
  3. transport the artifact bytes as hex (bytes.hex(); no SemCode is interpreted here);
  4. run the Semantic VM twice; the two reports must be byte-identical;
  5. compare outcome, exact error message, every printed line and the guest counters.
Raw malformed artifacts (not admitted by the oracle verifier) are fed to the Semantic VM
directly to show its own decoder rejects them. VM mutations prove the comparison catches
deliberate semantic errors. Exit status is nonzero on any failure.
"""
import hashlib, json, os, pathlib, re, shutil, struct, subprocess, sys, tempfile

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent.parent
VM = REPO / "vm"
CORPUS = HERE / "corpus"


def need(name):
    v = os.environ.get(name)
    if not v or not pathlib.Path(v).is_file():
        sys.exit(f"run_vm_tests: {name} must name an existing smc-cli binary")
    return pathlib.Path(v)


SMC = need("SEMANTIC_SMC")
C0 = pathlib.Path(os.environ["SEMANTIC_C0_SMC"]) if os.environ.get("SEMANTIC_C0_SMC") else None

# The oracle is an exact reference revision, never "whatever smc-cli is on the path".
# smc-cli reports a fingerprint of the source it was built from; the binary SHA-256 is
# recorded as well (it also depends on the toolchain that built it).
ORACLE = {"git": "94c5c50d16c3f2efc7691456755461bedae83b18", "source_fingerprint": "1cd4707ec728c07a"}
C0_PIN = {"git": "89641da8", "source_fingerprint": "7f10f50c82216db2"}


def fingerprint(smc):
    p = subprocess.run([str(smc), "version", "--json"], capture_output=True, text=True, timeout=60)
    try:
        return json.loads(p.stdout).get("source_fingerprint")
    except ValueError:
        return None


if fingerprint(SMC) != ORACLE["source_fingerprint"]:
    sys.exit(f"run_vm_tests: SEMANTIC_SMC is not the pinned oracle "
             f"(source_fingerprint {fingerprint(SMC)!r}, expected {ORACLE['source_fingerprint']} = Semantic@{ORACLE['git']})")
if C0 and fingerprint(C0) != C0_PIN["source_fingerprint"]:
    sys.exit(f"run_vm_tests: SEMANTIC_C0_SMC is not the pinned C0 (source_fingerprint {fingerprint(C0)!r})")
OUT = pathlib.Path(os.environ.get("SEMANTIC_TEST_OUT", pathlib.Path(tempfile.gettempdir()) / "semantic-vm-tests")).resolve()
if REPO in OUT.parents or OUT == REPO:
    sys.exit("run_vm_tests: SEMANTIC_TEST_OUT must be outside the source tree")
for sub in ("art", "hex", "raw", "mut"):
    (OUT / sub).mkdir(parents=True, exist_ok=True)
sha = lambda b: hashlib.sha256(b).hexdigest()
HOST_MAX_STEPS = "200000000"


def run(cmd, cwd=None):
    p = subprocess.run([str(c) for c in cmd], cwd=cwd, capture_output=True, text=True, timeout=3600)
    return p.returncode, p.stdout, p.stderr


def metrics_of(stderr):
    for line in stderr.splitlines():
        if line.startswith('{"schema":"semantic.run.metrics'):
            return json.loads(line)
    return {}


def oracle_run(smc, art):
    rc, so, se = run([smc, "run-smc", art])
    lines = so.splitlines()
    msg = se.strip().splitlines()[-1] if se.strip() else ""
    return {"rc": rc, "prints": lines, "error": msg if rc != 0 else ""}


def parse_report(line):
    """SEMVM1 <len>:<status><len>:<metrics><len>:<count>(<len>:<print>)*"""
    if not line.startswith("SEMVM1 "):
        return None
    # lengths are text_len values, i.e. UTF-8 byte counts
    s, i, fields = line[7:].encode("utf-8"), 0, []
    while i < len(s):
        j = s.index(b":", i)
        n = int(s[i:j])
        fields.append(s[j + 1:j + 1 + n].decode("utf-8"))
        i = j + 1 + n
    status, metrics, count = fields[0], fields[1], int(fields[2])
    prints = fields[3:]
    if len(prints) != count:
        return None
    m = dict(kv.split("=") for kv in metrics.split()) if metrics else {}
    return {"status": status, "metrics": {k: int(v) for k, v in m.items()}, "prints": prints}


def semvm(vm_dir, hex_dir, hex_name):
    rc, so, se = run([SMC, "run", "main.sm", "--profile", "cli-read-only", "--root", hex_dir, "--envelope",
                      "trusted-compiler", "--max-steps", HOST_MAX_STEPS, "--metrics", "json", "--", hex_name], cwd=vm_dir)
    host = metrics_of(se)
    return {"raw": so, "report": parse_report(so), "host": {"outcome": host.get("outcome"), "counters": host.get("counters"),
            "wall_ms": (host.get("nondeterministic") or {}).get("wall_ms"), "error": host.get("error")}}


def judge(name, spec, oracle, counters, vm):
    """Returns (class, checks) for one case."""
    checks = {}
    exp = spec["expect"]
    if exp == "ok":
        checks["oracle_matches_expectation"] = oracle["rc"] == 0
    else:
        checks["oracle_matches_expectation"] = oracle["rc"] != 0 and oracle["error"].startswith(exp[len("error:"):])
    rep = vm["report"]
    checks["semvm_report_parsed"] = rep is not None
    if rep is None:
        return "FAIL", checks
    if spec.get("semvm") == "unsupported":
        checks["semvm_fails_explicitly_unsupported"] = rep["status"].startswith("error unsupported:")
        return ("UNSUPPORTED" if all(checks.values()) else "FAIL"), checks
    want_status = "ok" if oracle["rc"] == 0 else "error " + oracle["error"]
    # an admitted artifact must never be rejected by the Semantic VM's loader
    checks["not_rejected_at_load"] = not rep["status"].startswith("reject ")
    checks["status_identical"] = rep["status"] == want_status
    # The oracle CLI delivers collected print observations only after a successful run, so
    # the observation stream is comparable on success only (recorded either way).
    if oracle["rc"] == 0:
        checks["prints_identical"] = rep["prints"] == oracle["prints"]
    else:
        vm["observations_not_exposed_by_oracle_cli"] = rep["prints"]
    if counters:
        m = rep["metrics"]
        checks["guest_steps_identical"] = m.get("guest_steps") == counters.get("steps")
        checks["guest_calls_identical"] = m.get("guest_calls") == counters.get("calls")
        checks["peak_frames_identical"] = m.get("peak_frames") == counters.get("peak_frames")
    return ("VERIFIED" if all(checks.values()) else "FAIL"), checks


# ---------------------------------------------------------------- corpus
cases = json.loads((HERE / "corpus.json").read_text())
rows, failures = [], []
for name, spec in sorted(cases.items()):
    src = CORPUS / f"{name}.sm"
    art = OUT / "art" / f"{name}.smc"
    art.unlink(missing_ok=True)
    crc, cso, cse = run([SMC, "compile", src, "-o", art])
    row = {"case": name, "expect": spec["expect"], "semvm_expect": spec.get("semvm")}
    if crc != 0:
        row.update({"class": "FAIL", "checks": {"oracle_compiles": False}, "detail": (cso + cse)[-300:]})
        failures.append(name); rows.append(row)
        print(f"FAIL {name:<30} reference compile failed: {(cso + cse).strip()[-160:]}", flush=True)
        continue
    data = art.read_bytes()
    vrc, vso, vse = run([SMC, "verify", art])
    oracle = oracle_run(SMC, art)
    _, _, mse = run([SMC, "run", src, "--profile", "pure", "--root", CORPUS, "--metrics", "json"])
    om = metrics_of(mse)
    counters = om.get("counters") if om.get("outcome") in ("ok", "runtime_error", "quota_exceeded") else None
    c0 = None
    if C0:
        c0v, _, c0e = run([C0, "verify", art])
        c0 = {"admitted": c0v == 0, "detail": c0e.strip()[-160:]}
        if c0v == 0:
            c0["run"] = oracle_run(C0, art)
            c0["same_as_oracle"] = c0["run"] == oracle
    hexf = OUT / "hex" / f"{name}.hex"
    hexf.write_text(data.hex())
    first, second = semvm(VM, OUT / "hex", hexf.name), semvm(VM, OUT / "hex", hexf.name)
    cls, checks = judge(name, spec, oracle, counters, first)
    checks["oracle_admits"] = vrc == 0
    checks["semvm_deterministic"] = first["raw"] == second["raw"] and first["raw"] != ""
    checks["host_run_completed"] = first["host"]["outcome"] == "ok"
    if not all(checks.values()):
        cls = "FAIL"
    row.update({"class": cls, "checks": checks, "artifact_sha256": sha(data), "artifact_bytes": len(data),
                "oracle": oracle, "oracle_counters": counters, "c0": c0,
                "semvm": first["report"], "host": first["host"],
                "observations_not_exposed_by_oracle_cli": first.get("observations_not_exposed_by_oracle_cli")})
    rows.append(row)
    if cls == "FAIL":
        failures.append(name)
    m = (first["report"] or {}).get("metrics", {})
    hc = first["host"]["counters"] or {}
    bad = " ".join(k for k, v in checks.items() if not v)
    print(f"{cls:<11} {name:<30} guest_steps={m.get('guest_steps')} host_steps={hc.get('steps')} host_calls={hc.get('calls')} {bad}", flush=True)
    if bad:
        print(f"            oracle={oracle} counters={counters}\n            semvm={first['report']} host_error={first['host']['error']}", flush=True)


# ---------------------------------------------------------------- raw malformed artifacts
def instr_start(data, fname):
    """Absolute offset of the instruction stream of function fname (framing only)."""
    p = 14
    dc = struct.unpack_from("<H", data, 12)[0]
    for _ in range(dc):
        p += 2 + struct.unpack_from("<H", data, p)[0]
        vc = struct.unpack_from("<H", data, p)[0]; p += 2
        for _ in range(vc):
            p += 2 + struct.unpack_from("<H", data, p)[0] + 2
    while p < len(data):
        nl = struct.unpack_from("<H", data, p)[0]
        name = data[p + 2:p + 2 + nl].decode()
        p += 2 + nl
        cl = struct.unpack_from("<I", data, p)[0]; p += 4
        q, end = p, p + cl
        sc = struct.unpack_from("<H", data, q)[0]; q += 2
        for _ in range(sc):
            q += 2 + struct.unpack_from("<H", data, q)[0]
        if data[q:q + 4] == b"DBG0":
            q += 6 + 10 * struct.unpack_from("<H", data, q + 4)[0]
        assert data[q:q + 4] == b"OWN0"
        ec = struct.unpack_from("<H", data, q + 4)[0]
        q += 6
        for _ in range(ec):
            kind, mode = data[q], data[q + 1]
            q += 2 + (4 if (kind == 1 or mode == 1) else 0) + 4
            cc = struct.unpack_from("<H", data, q)[0]; q += 2
            for _ in range(cc):
                q += {0: 3, 1: 5, 2: 7, 3: 5}[data[q]]
        q += 6 + struct.unpack_from("<H", data, q + 4)[0]
        if name == fname:
            return q
        p = end
    raise KeyError(fname)


def disasm_offsets(art, fname, opname):
    _, so, _ = run([SMC, "disasm", art])
    cur, out = None, []
    for line in so.splitlines():
        m = re.match(r"fn (\S+):", line)
        if m:
            cur = m.group(1)
        m = re.match(r"\s+([0-9a-f]{4}): (\S+)", line)
        if m and cur == fname and m.group(2) == opname:
            out.append(int(m.group(1), 16))
    return out


def patch(data, at, new):
    b = bytearray(data); b[at:at + len(new)] = new; return bytes(b)


raw_rows = []
base = (OUT / "art" / "b05_loops.smc").read_bytes() if (OUT / "art" / "b05_loops.smc").exists() else None
b02 = (OUT / "art" / "b02_i32_arithmetic.smc").read_bytes() if (OUT / "art" / "b02_i32_arithmetic.smc").exists() else None
b03 = (OUT / "art" / "b03_bool_logic.smc").read_bytes() if (OUT / "art" / "b03_bool_logic.smc").exists() else None
raw = []
if base and b02 and b03:
    ms = instr_start(base, "main")
    jmp = disasm_offsets(OUT / "art" / "b05_loops.smc", "main", "JMP")[0]
    tgt = struct.unpack_from("<I", base, ms + jmp + 1)[0]
    lv = disasm_offsets(OUT / "art" / "b05_loops.smc", "main", "LOAD_VAR")[0]
    m2 = instr_start(b02, "main")
    add = disasm_offsets(OUT / "art" / "b02_i32_arithmetic.smc", "main", "ADD_I32")[0]
    m3 = instr_start(b03, "main")
    lb = disasm_offsets(OUT / "art" / "b03_bool_logic.smc", "main", "LOAD_BOOL")[0]
    raw = [
        ("r01_truncated", base[:-1], "reject load: function code out of bounds"),
        ("r02_bad_magic", patch(base, 0, b"SEMCOD99"), "reject load: unsupported SemCode header"),
        ("r03_unknown_opcode", patch(base, ms, b"\x0f"), "reject load: unknown opcode 15"),
        ("r04_jump_into_instruction", patch(base, ms + jmp + 1, struct.pack("<I", tgt + 1)), "reject load: invalid jump address"),
        ("r05_string_id_out_of_range", patch(base, ms + lv + 3, b"\xff\xff"), "reject load: invalid string id"),
        ("r06_noncanonical_bool", patch(b03, m3 + lb + 3, b"\x02"), "reject load: non-canonical literal"),
        ("r07_undefined_register_read", patch(b02, m2 + add + 3, struct.pack("<H", 4000)), "error undefined register r4000"),
        ("r08_jump_past_end", patch(base, ms + jmp + 1, struct.pack("<I", 0x7fffff)), "reject load: invalid jump address"),
    ]
for rname, data, want in raw:
    f = OUT / "raw" / f"{rname}.smc"
    f.write_bytes(data)
    vrc, vso, vse = run([SMC, "verify", f])
    (OUT / "hex" / f"{rname}.hex").write_text(data.hex())
    vm = semvm(VM, OUT / "hex", f"{rname}.hex")
    st = (vm["report"] or {}).get("status", "")
    ok = vrc != 0 and st.startswith(want)
    raw_rows.append({"case": rname, "oracle_verifier_rejects": vrc != 0, "oracle_detail": (vso + vse).strip()[-200:],
                     "semvm_status": st, "expected_prefix": want, "pass": ok})
    if not ok:
        failures.append(f"raw:{rname}")
    print(f"{'PASS' if ok else 'FAIL':<11} raw:{rname:<26} oracle_verify_rc={vrc} semvm={st[:90]}", flush=True)

# transport boundary: invalid hex never reaches the decoder
(OUT / "hex" / "t01_bad_hex.hex").write_text("53454d434f4432zz")
t = semvm(VM, OUT / "hex", "t01_bad_hex.hex")
tst = (t["report"] or {}).get("status", "")
t_ok = tst.startswith("reject transport: invalid hex digit")
raw_rows.append({"case": "t01_bad_hex", "semvm_status": tst, "pass": t_ok})
if not t_ok:
    failures.append("transport:t01_bad_hex")
print(f"{'PASS' if t_ok else 'FAIL':<11} transport:t01_bad_hex{'':<14} semvm={tst}", flush=True)

# ---------------------------------------------------------------- VM mutations
MUTATIONS = [
    ("m01_add_becomes_sub", "engine.sm", "ri = l + r;", "ri = l - r;", ["b02_i32_arithmetic"]),
    ("m02_branch_polarity", "engine.sm", "} else if c.i == 1 {\n                        next = it.b;", "} else if c.i == 0 {\n                        next = it.b;", ["b04_branching"]),
    ("m03_qtruth_and_as_lattice", "engine.sm", "t = at * bt;\n                    f = af + bf - af * bf;", "t = at * bt;\n                    f = af * bf;", ["v08_quad_truth"]),
    ("m04_steps_off_by_one", "engine.sm", "steps > maxsteps", "steps >= maxsteps", ["x09_steps_quota"]),
    ("m05_frames_limit", "engine.sm", "let maxframes: i32 = 256;", "let maxframes: i32 = 257;", ["x10_frames_quota"]),
    # (a record-name-only mutation is equivalent: == is typed, so compared records share a name)
    ("m06_aggregate_eq_ignores_fields", "value.sm",
     "                for v in ox.items {\n                    xs = push(xs, v);\n                }\n                for v in oy.items {\n                    ys = push(ys, v);\n                }\n",
     "", ["v04_records", "v03_tuples"]),
    ("m07_guest_gets_host_effects", "engine.sm", "err = \"ABI \" + effect_name(tgt)", "res = unit; err = \"\" + \"\"", ["x15_capability_stdout_write"]),
    ("m08_u32_overflow_wraps", "engine.sm", "if l > 4294967295u32 - r {\n                                err = \"runtime trap: ArithmeticOverflow\";\n                            } else {\n                                ru = l + r;\n                            }",
     "if l > 4294967295u32 - r {\n                                ru = l - (4294967295u32 - r) - 1u32;\n                            } else {\n                                ru = l + r;\n                            }", ["x05_u32_add_overflow"]),
    ("m09_pop_drops_nothing", "engine.sm", "items: pop(o.items) });", "items: o.items });", ["v02_sequences"]),
    ("m10_steps_not_charged_for_jmp", "engine.sm", "} else if op == 5 {\n                    next = it.a;", "} else if op == 5 {\n                    next = it.a;\n                    steps = steps - 1;", ["b05_loops"]),
]
mut_rows = []
if os.environ.get("SEMVM_SKIP_MUTATIONS") != "1":
    for mname, fname, old, new, targets in MUTATIONS:
        d = OUT / "mut" / mname
        if d.exists():
            shutil.rmtree(d)
        shutil.copytree(VM, d)
        f = d / fname
        s = f.read_text()
        applied = s.count(old) == 1
        if applied:
            f.write_text(s.replace(old, new))
        killed_by, invalid = [], []
        for tname in targets:
            r = next(r for r in rows if r["case"] == tname)
            vm = semvm(d, OUT / "hex", f"{tname}.hex")
            if vm["report"] is None or vm["host"]["outcome"] != "ok":
                invalid.append(tname)          # the mutant did not run: not a semantic kill
                continue
            cls, _ = judge(tname, cases[tname], r["oracle"], r["oracle_counters"], vm)
            if cls == "FAIL":
                killed_by.append(tname)
        ok = applied and bool(killed_by) and not invalid
        mut_rows.append({"mutation": mname, "applied": applied, "killed_by": killed_by,
                         "mutant_did_not_run": invalid, "pass": ok})
        if not ok:
            failures.append(f"mutation:{mname}")
        print(f"{'KILLED' if ok else 'SURVIVED':<11} mutation:{mname:<30} by {killed_by}", flush=True)

# ---------------------------------------------------------------- summary
classes = {}
for r in rows:
    classes[r["class"]] = classes.get(r["class"], 0) + 1
summary = {
    "runner": "tests/vm/run_vm_tests.py",
    "vm_source_sha256": {p.name: sha(p.read_bytes()) for p in sorted(VM.glob("*.sm"))},
    "oracle_smc": {"pin": ORACLE, "sha256": sha(SMC.read_bytes()), "version": run([SMC, "version", "--json"])[1]},
    "c0_smc": {"sha256": sha(C0.read_bytes()), "version": run([C0, "version", "--json"])[1]} if C0 else None,
    "host_envelope": {"envelope": "trusted-compiler", "max_steps": int(HOST_MAX_STEPS)},
    "classes": classes, "failures": failures, "rows": rows, "raw": raw_rows, "mutations": mut_rows,
}
(OUT / "results.json").write_text(json.dumps(summary, indent=1))
print(f"\ncorpus: {classes}; raw/transport: {sum(r['pass'] for r in raw_rows)}/{len(raw_rows)}; "
      f"mutations killed: {sum(r['pass'] for r in mut_rows)}/{len(mut_rows)}; failures: {failures}")
print(f"evidence: {OUT / 'results.json'}")
sys.exit(1 if failures else 0)
