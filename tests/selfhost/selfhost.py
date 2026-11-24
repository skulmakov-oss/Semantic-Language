#!/usr/bin/env python3
"""Semantic self-host workflow (SHF-15/16 engineering integration).

One entry point, one fresh attempt directory per run:

  python -I tests/selfhost/selfhost.py all          --out <new dir> [options]
  python -I tests/selfhost/selfhost.py preflight    --out <new dir> [options]
  python -I tests/selfhost/selfhost.py build        --out <attempt> [options]
  python -I tests/selfhost/selfhost.py verify       --out <attempt> [--fixed-point c1c2|c2c3]
  python -I tests/selfhost/selfhost.py differential --out <attempt> [options]
  python -I tests/selfhost/selfhost.py report       --out <attempt>

Declared inputs (flags win over environment variables; nothing is guessed):
  --smc          SEMANTIC_C0_SMC        smc-cli built from the pinned C0 (sha256 pinned in pins.json)
  --driver       SEMANTIC_REV5_DRIVER   rev5 stored-exec-driver (sha256 pinned in pins.json)
  --c0-checkout  SEMANTIC_C0_CHECKOUT   clean checkout of skulmakov-oss/Semantic at the pinned SHA
  --s2           SEMANTIC_S2_HARNESS    optional: run every generation step under the S2 harness
  --source       compiler source root (default: <repo>/compiler); files are listed in pins.json

The host only moves bytes: it copies declared files, runs smc/driver/s2-harness, decodes the
compiler's "hex ..." stdout line and compares bytes. Lexing, parsing, analysis, lowering and
SemCode emission are performed by the Semantic compiler. Every phase writes <phase>.json; every
command is logged in commands.jsonl with its stdout/stderr under logs/. Files are created with
exclusive mode: nothing is overwritten or deleted. Acceptance is decided by check_results.py.
"""
import argparse
import concurrent.futures as cf
import datetime
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import subprocess
import sys
import threading
import time

sys.dont_write_bytecode = True  # never leave __pycache__ in the repository (bootstrap/** is protected)
HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent.parent
PINS = json.loads((HERE / "pins.json").read_text(encoding="utf-8"))
PHASES = ("preflight", "build", "verify", "differential", "report")
# Expected outcomes of the import fixtures (tests/compiler/imports/<name>/main.sm).
IMPORT_EXPECT = {
    "positive": "ok",
    "cycle": "reject:unsupported:cyclic executable helper import detected",
    "duplicate": "reject:name:function 'score' is defined more than once",
    "escape": "reject:unsupported:import path escapes the declared source root",
    "unsupported": "reject:unsupported:unsupported import form",
    "missing": "fault:ABI FsRead failure",
}
# Compiler-source mutations (applied to a copy of the snapshot, never to the repository).
SOURCE_MUTATIONS = {
    "comment_only": ("lower.sm", None, "\n// selfhost mutation probe: comment only\n", "same_artifact"),
    "diagnostic_text": ("parser.sm", "expected '{'", "expected '{' [mutated]", "different_artifact"),
}


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="milliseconds")


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def sha256_file(path):
    return sha256(pathlib.Path(path).read_bytes())


def load_validator():
    """bootstrap/validate_contract.py is read, never modified: its shf0-source-set-v1 rules are reused."""
    spec = importlib.util.spec_from_file_location("bootstrap_validate_contract", REPO / "bootstrap" / "validate_contract.py")
    module = importlib.util.module_from_spec(spec)
    old = sys.dont_write_bytecode
    sys.dont_write_bytecode = True
    try:
        spec.loader.exec_module(module)
    finally:
        sys.dont_write_bytecode = old
    return module


def classify(out):
    """Compiler-level reading of the compiler's stdout bytes."""
    if out.startswith(b"hex "):
        body = out[4:].strip()
        if len(body) % 2 or not re.fullmatch(rb"[0-9a-f]*", body):
            return {"kind": "malformed_hex"}
        return {"kind": "semcode", "semcode": bytes.fromhex(body.decode("ascii"))}
    m = re.fullmatch(rb"error (\w+) @(\d+): (.*?)\s*", out, re.S)
    if m:
        return {"kind": "diagnostic", "category": m.group(1).decode(), "message": m.group(3).decode("utf-8", "replace")}
    return {"kind": "no_output" if not out.strip() else "other"}


def driver_provenance(stderr):
    for line in reversed(stderr.decode("utf-8", "replace").splitlines()):
        if line.startswith("{") and "stored_exec_provenance" in line:
            try:
                j = json.loads(line)
            except ValueError:
                continue
            return {k: j.get(k) for k in ("outcome", "phase_reached", "error", "counters", "executed_artifact_sha256", "effective_quotas")}
    return None


class Attempt:
    def __init__(self, out, create=False):
        self.out = pathlib.Path(out).resolve()
        if create:
            if self.out.exists():
                sys.exit(f"selfhost: attempt directory already exists: {self.out}")
            if REPO in self.out.parents or self.out == REPO:
                sys.exit("selfhost: the attempt directory must be outside the repository")
            if not self.out.parent.is_dir():
                sys.exit(f"selfhost: parent directory does not exist: {self.out.parent}")
            self.out.mkdir()
            (self.out / "logs").mkdir()
        elif not (self.out / "commands.jsonl").exists() and not (self.out / "logs").is_dir():
            sys.exit(f"selfhost: not an attempt directory: {self.out}")
        self.lock = threading.Lock()

    def path(self, rel):
        p = (self.out / rel).resolve()
        if self.out not in p.parents:
            raise SystemExit(f"selfhost: refusing a path outside the attempt: {rel}")
        return p

    def create(self, rel, data):
        p = self.path(rel)
        p.parent.mkdir(parents=True, exist_ok=True)
        with open(p, "xb") as f:
            f.write(data)
        return p

    def write_json(self, rel, obj):
        return self.create(rel, (json.dumps(obj, indent=1, sort_keys=True) + "\n").encode("utf-8"))

    def read_json(self, rel):
        p = self.path(rel)
        if not p.exists():
            sys.exit(f"selfhost: missing {rel}; run the earlier phase first")
        return json.loads(p.read_text(encoding="utf-8"))

    def run(self, tag, argv, cwd=None, timeout=3600):
        """Runs one command; stdout/stderr are kept byte-exact under logs/<tag>.*; no retries."""
        if not re.fullmatch(r"[A-Za-z0-9_.-]+", tag):
            raise SystemExit(f"selfhost: bad tag {tag}")
        start, t0 = now(), time.perf_counter()
        try:
            p = subprocess.run([str(a) for a in argv], cwd=str(cwd) if cwd else None, capture_output=True, timeout=timeout)
            exit_code, so, se = p.returncode, p.stdout, p.stderr
        except subprocess.TimeoutExpired as e:
            exit_code, so, se = "timeout", e.stdout or b"", e.stderr or b""
        wall = round(time.perf_counter() - t0, 3)
        self.create(f"logs/{tag}.stdout", so)
        self.create(f"logs/{tag}.stderr", se)
        entry = {"tag": tag, "argv": [str(a) for a in argv], "cwd": str(cwd) if cwd else None, "start_utc": start, "wall_s": wall,
                 "exit": exit_code, "stdout_sha256": sha256(so), "stderr_sha256": sha256(se)}
        with self.lock:
            with open(self.out / "commands.jsonl", "a", encoding="utf-8") as f:
                f.write(json.dumps(entry, sort_keys=True) + "\n")
        return exit_code, so, se


class Tools:
    def __init__(self, args):
        def pick(flag, env):
            v = getattr(args, flag, None) or os.environ.get(env)
            return pathlib.Path(v).resolve() if v else None
        self.smc = pick("smc", "SEMANTIC_C0_SMC")
        self.driver = pick("driver", "SEMANTIC_REV5_DRIVER")
        self.s2 = pick("s2", "SEMANTIC_S2_HARNESS")
        self.c0_checkout = pick("c0_checkout", "SEMANTIC_C0_CHECKOUT")
        self.source = pathlib.Path(args.source).resolve() if getattr(args, "source", None) else REPO / PINS["source_set"]["root"]

    def need(self, name):
        """Hash-check a pinned tool immediately before use (fail closed)."""
        p = getattr(self, name)
        if p is None or not p.is_file():
            sys.exit(f"selfhost: required tool '{name}' is not supplied or does not exist")
        want = PINS["tools"][name]["sha256"]
        got = sha256_file(p)
        if got != want:
            sys.exit(f"selfhost: tool identity mismatch for {name}: {got} != pinned {want}")
        return p


def git(repo, *args):
    p = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True)
    return p.returncode, p.stdout.strip()


def snapshot_entries(directory):
    files = PINS["source_set"]["files"]
    root = PINS["source_set"]["root"] + "/"
    return [(f, (pathlib.Path(directory) / f[len(root):]).read_bytes()) for f in files]


def preflight(att, tools, args):
    v = load_validator()
    res = {"schema": "semantic.selfhost.preflight/1", "utc": now(), "attempt_root": str(att.out)}
    ok = True
    tool_rows = {}
    for name in ("smc", "driver"):
        p = getattr(tools, name)
        got = sha256_file(p) if p and p.is_file() else None
        tool_rows[name] = {"path": str(p) if p else None, "sha256": got, "pinned": PINS["tools"][name]["sha256"], "match": got == PINS["tools"][name]["sha256"]}
        ok &= tool_rows[name]["match"]
    if tools.s2:
        if not tools.s2.is_file():
            sys.exit("selfhost: --s2 does not exist")
        ec, so, _ = att.run("preflight.s2_identity", [tools.s2, "--print-identity"])
        got = sha256_file(tools.s2)
        tool_rows["s2"] = {"path": str(tools.s2), "sha256": got, "print_identity_exit": ec, "pinned": PINS["s2_harness"]["sha256_allowed"],
                           "match": got in PINS["s2_harness"]["sha256_allowed"]}
        ok &= ec == 0 and tool_rows["s2"]["match"]
    res["tools"] = tool_rows
    if tool_rows["smc"]["match"]:
        att.run("preflight.smc_version", [tools.smc, "version", "--json"])
    c0 = {"path": str(tools.c0_checkout) if tools.c0_checkout else None, "pinned_sha": PINS["c0"]["sha"]}
    if tools.c0_checkout and tools.c0_checkout.is_dir():
        errors = v.verify_reference_checkout(tools.c0_checkout, PINS["c0"]["sha"])
        # Logged so the checker can recompute the attestation from raw output.
        he, hso, _ = att.run("preflight.c0_head", ["git", "-C", tools.c0_checkout, "rev-parse", "HEAD"])
        se_, sso, _ = att.run("preflight.c0_status", ["git", "-C", tools.c0_checkout, "status", "--porcelain", "--untracked-files=all"])
        c0["head"] = hso.decode("utf-8", "replace").strip() if he == 0 else None
        c0["clean"] = se_ == 0 and sso.strip() == b""
        c0["origin"] = git(tools.c0_checkout, "remote", "get-url", "origin")[1]
        c0["verify_reference_checkout_errors"] = errors
        c0["ok"] = not errors and c0["head"] == PINS["c0"]["sha"] and c0["clean"]
    else:
        c0["ok"] = False
        c0["error"] = "C0 checkout not supplied (--c0-checkout / SEMANTIC_C0_CHECKOUT)"
    ok &= c0["ok"]
    res["c0_checkout"] = c0
    files = PINS["source_set"]["files"]
    src = {"source_root": str(tools.source), "protocol": PINS["source_set"]["protocol"], "files": []}
    errors = list(v.check_paths(files, PINS["source_set"]["root"]))
    entries = []
    for f in files:
        p = tools.source / f.split("/", 1)[1]
        data = p.read_bytes() if p.is_file() else None
        if data is None:
            errors.append(f"missing source file {f}")
            continue
        errors += v.check_content(f, data)
        entries.append((f, data))
        src["files"].append({"path": f, "sha256": sha256(data), "bytes": len(data)})
    src["errors"] = errors
    if not errors:
        src["identity"] = v.source_set_identity(entries)
        for f, data in entries:
            att.create("snapshot/" + f.split("/", 1)[1], data)
        src["snapshot_identity"] = v.source_set_identity(snapshot_entries(att.out / "snapshot"))
        src["ok"] = src["identity"] == src["snapshot_identity"]
    else:
        src["ok"] = False
    ok &= src["ok"]
    res["source_set"] = src
    head = git(REPO, "rev-parse", "HEAD")
    status = git(REPO, "status", "--porcelain", "--untracked-files=all")
    res["repository"] = {"head": head[1], "dirty": bool(status[1]), "status": status[1].splitlines()}
    if args.require_clean and res["repository"]["dirty"]:
        ok = False
    res["status"] = "pass" if ok else "fail"
    att.write_json("preflight.json", res)
    return ok


def execute_compiler(att, tools, tag, artifact, root, entry, envelope, source_files=None, source_id=None, accepted_name=None):
    """Executes an admitted stored compiler artifact on <root>/<entry>. Returns (kind, stdout_bytes, record)."""
    art_sha = sha256_file(artifact)
    env = PINS["envelopes"][envelope]
    rec = {"executor": "s2" if (tools.s2 and source_files) else "driver", "artifact": str(artifact), "artifact_sha256": art_sha, "envelope": envelope}
    driver = tools.need("driver")
    if rec["executor"] == "s2":
        out = att.path(f"s2/{tag}")
        out.parent.mkdir(parents=True, exist_ok=True)
        argv = [tools.s2, "--driver", driver, "--artifact", artifact, "--artifact-sha256", art_sha, "--root", root, "--profile", "cli-read-only",
                "--envelope", "trusted-compiler", "--steps", env["max_steps"], "--calls", env["max_calls"], "--frames", env["max_frames"],
                "--job-memory-bytes", env["job_memory_bytes"], "--timeout-secs", env["timeout_secs"], "--active-process-limit", "1",
                "--source-set-id", source_id, "--source-snapshot", root, "--source-files", ",".join(source_files),
                "--stdout-hex-decode", "--accepted-name", accepted_name, "--out-dir", out, "--", entry]
        ec, so, se = att.run(tag, argv, timeout=env["timeout_secs"] + 300)
        vec, vso, _ = att.run(tag + ".verify_attempt", [tools.s2, "--verify-attempt", out])
        rec.update(harness_exit=ec, verify_attempt_exit=vec, attempt_dir=str(out))
        record_path = out / "s2_record.json"
        s2rec = json.loads(record_path.read_text(encoding="utf-8")) if record_path.exists() else {}
        rec["s2"] = {"accepted": s2rec.get("accepted"), "reject_reasons": s2rec.get("reject_reasons"), "containment": s2rec.get("containment"),
                     "qualification": s2rec.get("qualification"), "published": s2rec.get("published"), "provenance": s2rec.get("provenance"),
                     "harness": s2rec.get("harness")}
        stdout = (out / "capture" / "stdout.bin").read_bytes() if (out / "capture" / "stdout.bin").exists() else b""
        rec["driver"] = driver_provenance((out / "capture" / "stderr.bin").read_bytes()) if (out / "capture" / "stderr.bin").exists() else None
        if vec == 0 and s2rec.get("accepted") is True:
            accepted = (out / "accepted" / accepted_name).read_bytes()
            rec["accepted_sha256"] = sha256(accepted)
            return "semcode", accepted, rec
        return "rejected_by_s2", stdout, rec
    argv = [driver, artifact, "--expect-sha256", art_sha, "--profile", "cli-read-only", "--root", root, "--envelope", "trusted-compiler",
            "--max-steps", env["max_steps"], "--max-calls", env["max_calls"], "--max-frames", env["max_frames"], "--metrics", "json", "--", entry]
    ec, so, se = att.run(tag, argv, timeout=env["timeout_secs"] + 300)
    rec.update(exit=ec, driver=driver_provenance(se))
    return None, so, rec


def build(att, tools, args):
    pre = att.read_json("preflight.json")
    if pre["status"] != "pass":
        sys.exit("selfhost: preflight did not pass")
    v = load_validator()
    snap = att.out / "snapshot"
    sid = pre["source_set"]["identity"]
    res = {"schema": "semantic.selfhost.build/1", "utc": now(), "source_set_identity": sid, "generations": []}
    ok = v.source_set_identity(snapshot_entries(snap)) == sid
    smc = tools.need("smc")
    c1 = att.path("gen/C1.smc")
    c1.parent.mkdir(parents=True, exist_ok=True)
    ec, so, se = att.run("build.C1.compile", [smc, "compile", snap / PINS["source_set"]["entry"], "-o", c1])
    row = {"name": "C1", "producer": "C0 smc compile", "compile_exit": ec, "artifact": "gen/C1.smc"}
    if ec == 0 and c1.exists():
        row.update(sha256=sha256_file(c1), bytes=c1.stat().st_size)
        row["verify_exit"] = att.run("build.C1.verify", [tools.need("smc"), "verify", c1])[0]
    ok &= row.get("verify_exit") == 0
    res["generations"].append(row)
    prev = row
    names = [f.split("/", 1)[1] for f in PINS["source_set"]["files"]]
    for n in range(2, args.generations + 1):
        if not ok:
            break
        name = f"C{n}"
        kind, out, rec = execute_compiler(att, tools, f"build.{name}.execute", att.path(prev["artifact"]), snap, PINS["source_set"]["entry"],
                                          "compiler", source_files=names, source_id=sid, accepted_name=f"{name}.smc")
        row = {"name": name, "producer": prev["name"], "execution": rec}
        if kind is None:
            c = classify(out)
            kind = c["kind"]
            out = c.get("semcode", b"")
        row["kind"] = kind
        if kind == "semcode" and out:
            p = att.create(f"gen/{name}.smc", out)
            row.update(artifact=f"gen/{name}.smc", sha256=sha256(out), bytes=len(out))
            row["verify_exit"] = att.run(f"build.{name}.verify", [tools.need("smc"), "verify", p])[0]
        ok &= row.get("verify_exit") == 0 and (rec.get("driver") or {}).get("executed_artifact_sha256") == prev["sha256"]
        res["generations"].append(row)
        prev = row
    res["snapshot_identity_after"] = v.source_set_identity(snapshot_entries(snap))
    ok &= res["snapshot_identity_after"] == sid and len(res["generations"]) == args.generations
    res["status"] = "pass" if ok else "fail"
    att.write_json("build.json", res)
    return ok


def first_difference(a, b):
    i = next((k for k in range(min(len(a), len(b))) if a[k] != b[k]), None)
    return i if i is not None else (None if len(a) == len(b) else min(len(a), len(b)))


def verify(att, tools, args):
    b = att.read_json("build.json")
    gens = {g["name"]: g for g in b["generations"] if g.get("artifact")}
    res = {"schema": "semantic.selfhost.verify/1", "utc": now(), "rule": "byte-equality-v1", "comparisons": [], "expect": args.fixed_point}
    data = {}
    for name, g in gens.items():
        data[name] = att.path(g["artifact"]).read_bytes()
    ok = all(sha256(data[n]) == gens[n]["sha256"] for n in data)
    for a, c in (("C1", "C2"), ("C2", "C3")):
        if a in data and c in data:
            res["comparisons"].append({"a": a, "b": c, "equal": data[a] == data[c], "a_sha256": sha256(data[a]), "b_sha256": sha256(data[c]),
                                       "a_bytes": len(data[a]), "b_bytes": len(data[c]), "first_difference": first_difference(data[a], data[c])})
    want = {"c1c2": [("C1", "C2"), ("C2", "C3")], "c2c3": [("C2", "C3")]}[args.fixed_point]
    for a, c in want:
        row = next((r for r in res["comparisons"] if r["a"] == a and r["b"] == c), None)
        ok &= bool(row and row["equal"])
    res["c1_equals_c2"] = next((r["equal"] for r in res["comparisons"] if r["a"] == "C1"), None)
    res["c2_equals_c3"] = next((r["equal"] for r in res["comparisons"] if r["a"] == "C2"), None)
    res["status"] = "pass" if ok and b["status"] == "pass" else "fail"
    att.write_json("verify.json", res)
    return res["status"] == "pass"


def corpus():
    cases = json.loads((REPO / "tests/compiler/cases.json").read_text(encoding="utf-8"))
    rows = [{"id": f"case.{n}", "dir": "tests/compiler/cases", "files": [f"{n}.sm"], "entry": f"{n}.sm", "expect": e} for n, e in sorted(cases.items())]
    for n, e in sorted(IMPORT_EXPECT.items()):
        d = REPO / "tests/compiler/imports" / n
        rows.append({"id": f"import.{n}", "dir": f"tests/compiler/imports/{n}", "files": sorted(p.name for p in d.iterdir() if p.suffix == ".sm"),
                     "entry": "main.sm", "expect": e})
    return rows


def differential(att, tools, args):
    b = att.read_json("build.json")
    if b["status"] != "pass":
        sys.exit("selfhost: build did not pass")
    gens = [g for g in b["generations"] if g["name"] in args.gens]
    if len(gens) != len(args.gens):
        sys.exit("selfhost: a requested generation was not built")
    rows = corpus()
    att.create("corpus/cases.json", (REPO / "tests/compiler/cases.json").read_bytes())
    smc = tools.need("smc")
    tools.need("driver")
    for r in rows:
        for f in r["files"]:
            att.create(f"fixtures/{r['id']}/root/{f}", (REPO / r["dir"] / f).read_bytes())
        r["file_sha256"] = {f: sha256_file(REPO / r["dir"] / f) for f in r["files"]}

    def one(r):
        root = att.path(f"fixtures/{r['id']}/root")
        base = f"fixtures/{r['id']}"
        ref = att.path(f"{base}/c0.smc")
        ec, so, se = att.run(f"diff.{r['id']}.c0", [smc, "compile", root / r["entry"], "-o", ref])
        r["c0"] = {"exit": ec, "artifact": f"{base}/c0.smc" if ref.exists() else None, "sha256": sha256_file(ref) if ref.exists() else None}
        r["gens"] = {}
        runs = [(g["name"], "") for g in gens] + [(args.gens[-1], ".repeat")]
        for name, suffix in runs:
            g = next(x for x in gens if x["name"] == name)
            kind, out, rec = execute_compiler(att, tools, f"diff.{r['id']}.{name}{suffix}", att.path(g["artifact"]), root, r["entry"], "fixture")
            c = classify(out)
            row = {"exit": rec["exit"], "stdout_sha256": sha256(out), "kind": c["kind"], "driver": rec["driver"]}
            if c["kind"] == "diagnostic":
                row.update(category=c["category"], message=c["message"])
            if c["kind"] == "semcode":
                p = att.create(f"{base}/{name}{suffix}.smc", c["semcode"])
                row.update(artifact=f"{base}/{name}{suffix}.smc", sha256=sha256(c["semcode"]))
            r["gens"][name + suffix] = row
        return r

    with cf.ThreadPoolExecutor(args.jobs) as ex:
        rows = list(ex.map(one, rows))
    # Admission and execution of every distinct SemCode artifact on both C0-lineage VMs.
    distinct = {}
    for r in rows:
        for src in [r["c0"]] + list(r["gens"].values()):
            if src.get("artifact"):
                distinct.setdefault(src["sha256"], src["artifact"])
    execs = {}

    def run_artifact(item):
        h, rel = item
        p = att.path(rel)
        ve = att.run(f"exec.{h[:16]}.verify", [smc, "verify", p])[0]
        e = {"artifact": rel, "verify_exit": ve}
        if ve == 0:
            re_, so, se = att.run(f"exec.{h[:16]}.run_smc", [smc, "run-smc", p], timeout=300)
            e.update(run_exit=re_, run_output=(so + se).decode("utf-8", "replace")[-600:])
            env = PINS["envelopes"]["program"]
            pe, pso, pse = att.run(f"exec.{h[:16]}.driver_pure", [tools.need("driver"), p, "--expect-sha256", h, "--profile", "pure", "--root", att.out / "fixtures",
                                                                  "--envelope", "trusted-compiler", "--max-steps", env["max_steps"], "--max-calls", env["max_calls"],
                                                                  "--max-frames", env["max_frames"], "--metrics", "json"], timeout=300)
            e.update(pure_exit=pe, pure=driver_provenance(pse))
        return h, e

    with cf.ThreadPoolExecutor(args.jobs) as ex:
        execs = dict(ex.map(run_artifact, sorted(distinct.items())))
    for r in rows:
        r["failures"] = row_failures(r, execs, args.gens)
        r["pass"] = not r["failures"]
    mutations = source_mutations(att, tools, b)
    res = {"schema": "semantic.selfhost.differential/1", "utc": now(), "generations": args.gens, "rows": rows, "executions": execs,
           "source_mutations": mutations, "counts": {"fixtures": len(rows), "passed": sum(r["pass"] for r in rows),
                                                      "distinct_artifacts": len(execs)}}
    ok = all(r["pass"] for r in rows) and all(m["pass"] for m in mutations)
    res["status"] = "pass" if ok else "fail"
    att.write_json("differential.json", res)
    return ok


def driver_trapped(pure, frag):
    """A program trap on the driver: execution started, did not finish ok, and reports the expected text
    (runtime_error for assertions/arithmetic, quota_exceeded for a frame/step quota)."""
    return (pure.get("phase_reached") == "execute" and pure.get("outcome") not in (None, "executed_ok")
            and frag in (pure.get("error") or ""))


def row_failures(r, execs, gens):
    f = []
    expect = r["expect"]
    outs = [r["gens"][g] for g in gens]
    repeat = r["gens"][gens[-1] + ".repeat"]
    if len({o["stdout_sha256"] for o in outs}) != 1:
        f.append("generations disagree")
    if repeat["stdout_sha256"] != outs[-1]["stdout_sha256"]:
        f.append("nondeterministic compiler output")
    for o in outs:
        kind = o["kind"]
        if expect == "ok" or expect.startswith("trap:"):
            if kind != "semcode":
                f.append(f"expected SemCode, got {kind}")
                continue
            if not r["c0"]["sha256"] or o["sha256"] != r["c0"]["sha256"]:
                f.append("SemCode differs from the C0 compilation of the same source")
            e = execs.get(o["sha256"], {})
            if e.get("verify_exit") != 0:
                f.append("SemCode not admitted by sm-verify")
                continue
            if expect == "ok":
                if e.get("run_exit") != 0 or (e.get("pure") or {}).get("outcome") != "executed_ok":
                    f.append("expected a clean run on both VMs")
            else:
                frag = expect.split(":", 1)[1]
                pure = e.get("pure") or {}
                if e.get("run_exit") == 0 or frag not in e.get("run_output", ""):
                    f.append(f"expected a trap containing {frag!r} on smc run-smc")
                if not driver_trapped(pure, frag):
                    f.append(f"expected a trap containing {frag!r} on the driver")
        elif expect.startswith("reject:"):
            _, cat, frag = expect.split(":", 2)
            if kind != "diagnostic" or o.get("category") != cat or frag not in o.get("message", ""):
                f.append(f"expected diagnostic {cat}: {frag!r}, got {kind}")
            if (o.get("driver") or {}).get("outcome") != "executed_ok":
                f.append("a compiler diagnostic must come from a completed compiler run")
        elif expect.startswith("fault:"):
            frag = expect.split(":", 1)[1]
            d = o.get("driver") or {}
            if kind == "semcode" or d.get("outcome") == "executed_ok" or frag not in (d.get("error") or ""):
                f.append(f"expected a compiler process fault containing {frag!r}")
        else:
            f.append(f"unknown expectation {expect!r}")
    return f


def source_mutations(att, tools, build_record):
    """Compiler-source mutation proof: identity always changes; a comment leaves C1 unchanged, a code change does not."""
    v = load_validator()
    smc = tools.need("smc")
    c1 = next(g for g in build_record["generations"] if g["name"] == "C1")
    out = []
    for name, (fname, old, new, expect) in sorted(SOURCE_MUTATIONS.items()):
        base = f"mutations/{name}"
        for f in PINS["source_set"]["files"]:
            short = f.split("/", 1)[1]
            data = att.path(f"snapshot/{short}").read_bytes()
            if short == fname:
                text = data.decode("utf-8")
                if old is None:
                    text = text + new
                else:
                    if text.count(old) < 1:
                        raise SystemExit(f"selfhost: mutation anchor not found: {name}")
                    text = text.replace(old, new, 1)
                data = text.encode("utf-8")
            att.create(f"{base}/src/{short}", data)
        sid = v.source_set_identity(snapshot_entries(att.path(f"{base}/src")))
        art = att.path(f"{base}/C1.smc")
        ec = att.run(f"mutation.{name}.compile", [smc, "compile", att.path(f"{base}/src/main.sm"), "-o", art])[0]
        row = {"name": name, "file": fname, "expect": expect, "compile_exit": ec, "source_set_identity": sid,
               "identity_changed": sid != build_record["source_set_identity"]}
        if art.exists():
            row["sha256"] = sha256_file(art)
            row["same_as_C1"] = row["sha256"] == c1["sha256"]
        row["pass"] = ec == 0 and row["identity_changed"] and row.get("same_as_C1") == (expect == "same_artifact")
        out.append(row)
    return out


def report(att, tools, args):
    sys.path.insert(0, str(HERE))
    import check_results  # noqa: E402  (sibling module, independent checker)
    extra = [] if (att.out / "differential.json").exists() else ["--partial"]
    return check_results.main([str(att.out), "--write"] + extra) == 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("phase", choices=PHASES + ("all",))
    ap.add_argument("--out", required=True, help="attempt directory (created by preflight/all; must not exist)")
    ap.add_argument("--smc")
    ap.add_argument("--driver")
    ap.add_argument("--s2")
    ap.add_argument("--c0-checkout", dest="c0_checkout")
    ap.add_argument("--source", help="compiler source root (default <repo>/compiler)")
    ap.add_argument("--generations", type=int, default=3, choices=(3, 4, 5),
                    help="C1..Cn; at least 3, because both fixed-point modes compare C2 with C3")
    ap.add_argument("--fixed-point", default="c1c2", choices=("c1c2", "c2c3"),
                    help="c1c2: Bootstrap Contract v3 rule C1 == C2 (and C2 == C3); c2c3: post-bootstrap stability only")
    ap.add_argument("--jobs", type=int, default=6)
    ap.add_argument("--require-clean", action="store_true", help="fail preflight on a dirty repository tree")
    args = ap.parse_args(argv)
    args.gens = ["C1", "C2"]  # the differential phase always compiles the corpus with C1 and C2 (plus a C2 repeat)
    tools = Tools(args)
    att = Attempt(args.out, create=args.phase in ("preflight", "all"))
    steps = PHASES if args.phase == "all" else (args.phase,)
    fn = {"preflight": preflight, "build": build, "verify": verify, "differential": differential, "report": report}
    ok = True
    for step in steps:
        if not ok and step != "report":
            print(f"selfhost: skipping {step} after a failed phase", flush=True)
            continue
        t0 = time.perf_counter()
        ok_step = fn[step](att, tools, args)
        print(f"selfhost: {step}: {'pass' if ok_step else 'FAIL'} ({time.perf_counter() - t0:.1f}s)", flush=True)
        ok = ok and ok_step
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
