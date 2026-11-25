#!/usr/bin/env python3
"""Fail-closed result checker for a tests/selfhost attempt directory.

  python -I tests/selfhost/check_results.py <attempt> [--write] [--partial]
  python -I tests/selfhost/check_results.py --selftest <passing attempt> <new scratch dir>

The checker never trusts a recorded verdict and is deliberately independent of selfhost.py
(it re-implements the framing, the classification, the import expectations and the mutation
table). From the raw files of the attempt it recomputes:
  * the exact file inventory and every command-log hash;
  * which tool ran each command (argv) and that each tool still hashes to its pin, if present;
  * the C0 checkout attestation from the logged git output;
  * the source-set identity of the snapshot;
  * every generation: decoded again from the producing run's stdout and bound to the executor
    provenance of the previous generation; under S2, the accepted record and published hash;
  * the byte-equality comparisons;
  * the corpus: it must equal the repository's tests/compiler/cases.json plus the pinned import
    fixtures, with identical fixture bytes; every fixture run is bound to its generation;
    verdicts are recomputed from stdout, stderr provenance and VM logs (both VMs for traps);
  * the two pinned compiler-source mutations, re-applied to the snapshot.
A zero exit code alone never passes anything. --partial accepts an attempt without the
differential phase (scope "partial", e.g. the historical reproduction); the verdict says so.
--selftest copies a passing attempt, injects known violations one at a time and requires every
one to be rejected while the unmodified copy passes.
"""
import hashlib
import json
import pathlib
import re
import shutil
import sys

sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent.parent
PINS = json.loads((HERE / "pins.json").read_text(encoding="utf-8"))
OUTPUTS = ("report.json", "summary.md")
IMPORT_EXPECT = {
    "positive": "ok",
    "cycle": "reject:unsupported:cyclic executable helper import detected",
    "duplicate": "reject:name:function 'score' is defined more than once",
    "escape": "reject:unsupported:import path escapes the declared source root",
    "unsupported": "reject:unsupported:unsupported import form",
    "missing": "fault:ABI FsRead failure",
}
MUTATIONS = {
    "comment_only": ("lower.sm", None, "\n// selfhost mutation probe: comment only\n", "same_artifact"),
    "diagnostic_text": ("parser.sm", "expected '{'", "expected '{' [mutated]", "different_artifact"),
}


def sha(b):
    return hashlib.sha256(b).hexdigest()


def frame_identity(entries):
    """shf0-source-set-v1 framing, reimplemented here (not imported) so the checker is independent."""
    h = hashlib.sha256(b"SHF0-SOURCE-SET\x00v1\x00")
    h.update(len(entries).to_bytes(8, "big"))
    for p, c in entries:
        pb = p.encode("utf-8")
        h.update(len(pb).to_bytes(4, "big") + pb + len(c).to_bytes(8, "big") + c)
    return "sha256:" + h.hexdigest()


def classify(out):
    if out.startswith(b"hex "):
        body = out[4:].strip()
        if len(body) % 2 or not re.fullmatch(rb"[0-9a-f]*", body):
            return {"kind": "malformed_hex"}
        return {"kind": "semcode", "semcode": bytes.fromhex(body.decode("ascii"))}
    m = re.fullmatch(rb"error (\w+) @(\d+): (.*?)\s*", out, re.S)
    if m:
        return {"kind": "diagnostic", "category": m.group(1).decode(), "message": m.group(3).decode("utf-8", "replace")}
    return {"kind": "no_output" if not out.strip() else "other"}


METRICS_SCHEMA = "semantic.run.metrics/0.1"


def provenance(stderr):
    """The executor's own report of a run, re-read from raw stderr: the `smc run-smc` metrics line (the
    Bootstrap Contract v4 executor) or the rev5 driver's provenance line (S2 engineering path only).
    `completed` = the program ran to its end."""
    for line in reversed(stderr.decode("utf-8", "replace").splitlines()):
        if not line.startswith("{"):
            continue
        try:
            j = json.loads(line)
        except ValueError:
            continue
        if not isinstance(j, dict):
            continue
        if j.get("schema") == METRICS_SCHEMA:
            return {**j, "completed": j.get("outcome") == "ok"}
        if "stored_exec_provenance" in line:
            return {**j, "completed": j.get("outcome") == "executed_ok"}
    return None


def ran(pv, sha_=None):
    """smc run-smc passed the --expect-sha256 identity check and executed (any later phase); the rev5 driver
    names the executed artifact itself."""
    if pv.get("schema") == METRICS_SCHEMA:
        return pv.get("phase_reached") in ("verify", "entry", "host_init", "execute")
    return sha_ is not None and pv.get("executed_artifact_sha256") == sha_


def pinned(name):
    t = PINS["tools"][name]
    return list(t.get("sha256_allowed") or [t["sha256"]])


def norm(p):
    """Separator normalisation only: paths are compared case-sensitively (the pipeline records the exact strings)."""
    return str(p).replace("\\", "/")


def mutated_sources(snapshot, name):
    fname, old, new, _ = MUTATIONS[name]
    out = {}
    for f, data in snapshot.items():
        if f == fname:
            text = data.decode("utf-8")
            text = text + new if old is None else text.replace(old, new, 1)
            data = text.encode("utf-8")
        out[f] = data
    return out


class Check:
    def __init__(self, root):
        self.root = pathlib.Path(root).resolve()
        self.violations = []
        self.facts = {}
        self.expected = set()

    def bad(self, msg):
        self.violations.append(msg)

    def read(self, rel):
        self.expected.add(rel)
        p = self.root / rel
        if not p.is_file():
            self.bad(f"missing file {rel}")
            return None
        return p.read_bytes()

    def json(self, rel, required=True):
        if not required and not (self.root / rel).is_file():
            return None
        b = self.read(rel)
        if b is None:
            return None
        try:
            obj = json.loads(b)
        except ValueError:
            self.bad(f"{rel} is not JSON")
            return None
        if not isinstance(obj, dict):
            self.bad(f"{rel} is not a JSON object")
            return None
        return obj


def sidecar(c, rel, data):
    """smc compile writes <artifact>.provenance.json; it must name exactly the artifact beside it."""
    b = c.read(rel + ".provenance.json")
    if b is not None and data is not None:
        try:
            if json.loads(b).get("artifact_hash") != "sha256:" + sha(data):
                c.bad(f"{rel}: compiler provenance sidecar names a different artifact")
        except ValueError:
            c.bad(f"{rel}: compiler provenance sidecar is not JSON")


def check(root, partial=False):
    c = Check(root)
    cmds = {}
    raw = c.read("commands.jsonl")
    for line in (raw or b"").decode("utf-8").splitlines():
        e = json.loads(line)
        if e["tag"] in cmds:
            c.bad(f"duplicate command tag {e['tag']}")
        cmds[e["tag"]] = e
        for s in ("stdout", "stderr"):
            b = c.read(f"logs/{e['tag']}.{s}")
            if b is not None and sha(b) != e[f"{s}_sha256"]:
                c.bad(f"log logs/{e['tag']}.{s} does not match its recorded hash")

    def log(tag, stream="stdout"):
        if tag not in cmds:
            c.bad(f"no command {tag}")
            return b""
        p = c.root / "logs" / f"{tag}.{stream}"
        return p.read_bytes() if p.is_file() else b""

    def exit_of(tag):
        return cmds[tag]["exit"] if tag in cmds else None

    pre = c.json("preflight.json") or {}
    tools = pre.get("tools") or {}

    root_rec = pre.get("attempt_root") or ""
    exp_cmds = {}

    def P(rel):
        return root_rec + "/" + rel

    def tool(name):
        return (tools.get(name) or {}).get("path") or f"<no {name}>"

    def expect_cmd(tag, argv):
        """Records the exact command line the pipeline must have run for this tag."""
        exp_cmds[tag] = [str(a) for a in argv]

    # ---------------------------------------------------------------- preflight
    s2_configured = "s2" in tools
    c.facts["generation_runner"] = "s2" if s2_configured else "smc-run-smc"
    ext = tools.get("executor") or {}
    repro = ext.get("reproduction_build") is True
    for name in ("smc", "executor") + (("driver",) if s2_configured else ()):
        t = tools.get(name) or {}
        allowed = pinned(name)
        if name in ("executor", "smc") and repro:
            allowed = [t.get("sha256")] if t.get("sha256") else []  # admitted by fingerprint below
        if t.get("sha256") not in allowed:
            c.bad(f"preflight: {name} identity is not the pinned identity")
        p = pathlib.Path(t.get("path") or "")
        if t.get("path") and p.is_file():
            if sha(p.read_bytes()) != t.get("sha256"):
                c.bad(f"preflight: {name} at {p} no longer hashes to its recorded identity")
            c.facts.setdefault("tools_rechecked", []).append(name)
    pins_ex = PINS["tools"]["executor"]
    expect_cmd("preflight.executor_version", [tool("executor"), "version", "--json"])
    try:
        fp = json.loads(log("preflight.executor_version")).get("source_fingerprint") if exit_of("preflight.executor_version") == 0 else None
    except ValueError:
        fp = None
    if fp != pins_ex["source_fingerprint"] or ext.get("source_fingerprint") != fp:
        c.bad("preflight: the executor's logged source fingerprint is not the pinned admitted revision")
    c.facts["executor"] = {"revision": pins_ex["revision"], "sha256": ext.get("sha256"), "source_fingerprint": fp,
                           "binary_pinned": ext.get("sha256") in pinned("executor"), "reproduction_build": repro}
    c0 = pre.get("c0_checkout") or {}
    smc_pinned = tools.get("smc", {}).get("sha256") == PINS["tools"]["smc"]["sha256"]
    if smc_pinned or repro:
        expect_cmd("preflight.smc_version", [tool("smc"), "version", "--json"])
        try:
            smc_fp = json.loads(log("preflight.smc_version")).get("source_fingerprint") if exit_of("preflight.smc_version") == 0 else None
        except ValueError:
            smc_fp = None
        # A pinned C0 binary must also report an admitted fingerprint; an unpinned (reproduction) build is
        # admitted only by it. C1 = C0(S) is then checked by identity like every other generation.
        if smc_fp not in PINS["tools"]["smc"]["source_fingerprints_allowed"] or (
                repro and (tools.get("smc") or {}).get("source_fingerprint") != smc_fp):
            c.bad("preflight: the C0 smc's logged source fingerprint is not an admitted build of the C0 SHA")
        c.facts["c0_smc"] = {"sha256": (tools.get("smc") or {}).get("sha256"), "source_fingerprint": smc_fp, "binary_pinned": smc_pinned}
    expect_cmd("preflight.c0_head", ["git", "-C", c0.get("path"), "rev-parse", "HEAD"])
    expect_cmd("preflight.c0_status", ["git", "-C", c0.get("path"), "status", "--porcelain", "--untracked-files=all"])
    head_log = log("preflight.c0_head").decode("utf-8", "replace").strip()
    if (exit_of("preflight.c0_head") != 0 or head_log != PINS["c0"]["sha"] or c0.get("head") != head_log
            or exit_of("preflight.c0_status") != 0 or log("preflight.c0_status").strip() != b"" or c0.get("clean") is not True
            or c0.get("verify_reference_checkout_errors")):
        c.bad("preflight: C0 checkout attestation does not hold (logged head/status, reference check)")
    files = PINS["source_set"]["files"]
    snapshot = {}
    for f in files:
        snapshot[f.split("/", 1)[1]] = c.read("snapshot/" + f.split("/", 1)[1]) or b""
    entries = [(f, snapshot[f.split("/", 1)[1]]) for f in files]
    snap_id = frame_identity(entries)
    src = pre.get("source_set") or {}
    if src.get("identity") != snap_id or src.get("snapshot_identity") != snap_id:
        c.bad("preflight: recorded source-set identity does not match the snapshot")
    if [r.get("path") for r in src.get("files", [])] != files:
        c.bad("preflight: source-set file list differs from pins.json")
    for (f, b), r in zip(entries, src.get("files", [])):
        if r.get("sha256") != sha(b):
            c.bad(f"preflight: snapshot file {f} differs from the recorded source")
    if s2_configured:
        if exit_of("preflight.s2_identity") != 0:
            c.bad("preflight: s2 --print-identity failed")
        expect_cmd("preflight.s2_identity", [tool("s2"), "--print-identity"])
        s2t = tools["s2"]
        if s2t.get("sha256") not in PINS["s2_harness"]["sha256_allowed"]:
            c.bad("preflight: the S2 harness identity is not pinned")
        sp = pathlib.Path(s2t.get("path") or "")
        if s2t.get("path") and sp.is_file():
            if sha(sp.read_bytes()) != s2t.get("sha256"):
                c.bad(f"preflight: s2 at {sp} no longer hashes to its recorded identity")
            c.facts.setdefault("tools_rechecked", []).append("s2")
    if not root_rec:
        c.bad("preflight: attempt root not recorded")
    if (pre.get("status") == "pass") == bool(c.violations):
        c.bad("preflight: recorded status disagrees with the evidence")
    c.facts["source_set_identity"] = snap_id
    repo_entries = [(f, (REPO / f).read_bytes() if (REPO / f).is_file() else b"") for f in files]
    c.facts["snapshot_is_repository_compiler"] = frame_identity(repo_entries) == snap_id

    # ---------------------------------------------------------------- build
    b = c.json("build.json") or {}
    gens = {}
    n_before = len(c.violations)
    if b.get("source_set_identity") != snap_id or b.get("snapshot_identity_after") != snap_id:
        c.bad("build: source-set identity changed during the build")
    prev = None
    for i, g in enumerate(b.get("generations", [])):
        name = g.get("name")
        if name != f"C{i + 1}":
            c.bad(f"build: generation order broken at {name}")
        rel = f"gen/{name}.smc"
        if g.get("artifact") != rel:
            c.bad(f"build: {name} artifact path is not {rel}")
        art = c.read(rel)
        if art is None:
            continue
        if sha(art) != g.get("sha256"):
            c.bad(f"build: {name} artifact hash differs from the record")
        if exit_of(f"build.{name}.verify") != 0:
            c.bad(f"build: {name} was not admitted by sm-verify")
        expect_cmd(f"build.{name}.verify", [tool("smc"), "verify", P(rel)])
        if exit_of(f"build.{name}.executor_verify") != 0:
            c.bad(f"build: {name} was not admitted by the executor's sm-verify")
        expect_cmd(f"build.{name}.executor_verify", [tool("executor"), "verify", P(rel)])
        if name == "C1":
            sidecar(c, rel, art)
            expect_cmd("build.C1.compile", [tool("smc"), "compile", P("snapshot/" + PINS["source_set"]["entry"]), "-o", P(rel)])
            if exit_of("build.C1.compile") != 0:
                c.bad("build: C0 compilation of S failed")
        else:
            ex = g.get("execution") or {}
            tag = f"build.{name}.execute"
            if ex.get("executor") != ("s2" if s2_configured else "smc-run-smc"):
                c.bad(f"build: {name} executor does not match the declared configuration")
            env = PINS["envelopes"]["compiler"]
            prev_rel = f"gen/{prev.get('name') if prev else '?'}.smc"
            prev_sha = (prev or {}).get("sha256")
            entry = PINS["source_set"]["entry"]
            if s2_configured:
                d = f"s2/{tag}"
                expect_cmd(tag, [tool("s2"), "--driver", tool("driver"), "--artifact", P(prev_rel), "--artifact-sha256", prev_sha, "--root", P("snapshot"),
                                 "--profile", "cli-read-only", "--envelope", "trusted-compiler", "--steps", env["max_steps"], "--calls", env["max_calls"],
                                 "--frames", env["max_frames"], "--job-memory-bytes", env["job_memory_bytes"], "--timeout-secs", env["timeout_secs"],
                                 "--active-process-limit", "1", "--source-set-id", snap_id, "--source-snapshot", P("snapshot"),
                                 "--source-files", ",".join(f.split("/", 1)[1] for f in files), "--stdout-hex-decode", "--accepted-name", f"{name}.smc",
                                 "--out-dir", P(d), "--", entry])
                expect_cmd(tag + ".verify_attempt", [tool("s2"), "--verify-attempt", P(d)])
                if exit_of(tag) != 0 or exit_of(tag + ".verify_attempt") != 0:
                    c.bad(f"build: {name}: S2 did not commit and accept the attempt")
                rec = c.json(f"{d}/s2_record.json") or {}
                acc = c.read(f"{d}/accepted/{name}.smc")
                pub = rec.get("published") or {}
                if rec.get("accepted") is not True or acc != art or pub.get("sha256") != sha(art):
                    c.bad(f"build: {name}: the S2 record does not publish exactly the generation artifact")
                stderr = c.read(f"{d}/capture/stderr.bin") or b""
                stdout = c.read(f"{d}/capture/stdout.bin") or b""
                # every file of the S2 attempt directory is read above: s2_record.json, accepted/, capture/
                c.facts.setdefault("s2", {})[name] = {"containment_verdict": (rec.get("containment") or {}).get("verdict"),
                                                       "harness": (rec.get("harness") or {}).get("sha256") if isinstance(rec.get("harness"), dict) else rec.get("harness")}
            else:
                expect_cmd(tag, [tool("executor"), "run-smc", P(prev_rel), "--expect-sha256", f"sha256:{prev_sha}", "--profile", "cli-read-only",
                                 "--root", P("snapshot"), "--envelope", "trusted-compiler", "--max-steps", env["max_steps"], "--max-calls", env["max_calls"],
                                 "--max-frames", env["max_frames"], "--metrics", "json", "--", entry])
                stderr, stdout = log(tag, "stderr"), log(tag)
                if exit_of(tag) != 0:
                    c.bad(f"build: {name}: executor exit code is not 0")
            cl = classify(stdout)
            if cl.get("semcode") != art:
                c.bad(f"build: {name}: artifact is not the decoded stdout of the producing run")
            pv = provenance(stderr) or {}
            want_q = {"max_steps": env["max_steps"], "max_calls": env["max_calls"], "max_frames": env["max_frames"]}
            got_q = {k: (pv.get("effective_quotas") or {}).get(k) for k in want_q}
            if not pv.get("completed") or pv.get("phase_reached") != "execute" or not ran(pv, prev_sha) or got_q != want_q:
                c.bad(f"build: {name}: executor provenance does not bind the previous generation under the declared envelope")
            c.facts.setdefault("counters", {})[name] = pv.get("counters")
            c.facts.setdefault("effective_quotas", {})[name] = pv.get("effective_quotas")
            c.facts.setdefault("outcomes", {})[name] = pv.get("outcome")
        gens[name] = {"sha256": sha(art), "bytes": len(art), "data": art}
        prev = g
    if len(gens) < 2:
        c.bad("build: fewer than two generations")
    if (b.get("status") == "pass") != (len(c.violations) == n_before):
        c.bad("build: recorded status disagrees with the evidence")

    # ---------------------------------------------------------------- verify
    v = c.json("verify.json") or {}
    n_before = len(c.violations)
    computed = {}
    for a, d in (("C1", "C2"), ("C2", "C3")):
        if a in gens and d in gens:
            computed[(a, d)] = gens[a]["data"] == gens[d]["data"]
    for r in v.get("comparisons", []):
        k = (r.get("a"), r.get("b"))
        if k not in computed or r.get("equal") != computed[k]:
            c.bad(f"verify: comparison {k} disagrees with the artifact bytes")
    if len(v.get("comparisons", [])) != len(computed):
        c.bad("verify: comparisons are incomplete")
    rule = v.get("expect")
    want = {"c1c2": [("C1", "C2"), ("C2", "C3")], "c2c3": [("C2", "C3")]}.get(rule)
    if want is None:
        c.bad("verify: unknown fixed-point expectation")
        want = []
    if not all(computed.get(k) for k in want):
        c.bad(f"verify: the declared fixed point ({rule}) does not hold")
    if (v.get("status") == "pass") != (len(c.violations) == n_before):
        c.bad("verify: recorded status disagrees with the evidence")
    c.facts["fixed_point_rule"] = rule
    c.facts["c1_equals_c2"] = computed.get(("C1", "C2"))
    c.facts["c2_equals_c3"] = computed.get(("C2", "C3"))
    c.facts["generations"] = {k: {"sha256": g["sha256"], "bytes": g["bytes"]} for k, g in gens.items()}

    # ---------------------------------------------------------------- differential
    d = c.json("differential.json", required=not partial)
    c.facts["scope"] = "full" if d is not None else "partial"
    if d is not None:
        if not isinstance(d, dict):
            c.bad("differential.json is not an object")
        else:
            check_differential(c, d, gens, snapshot, snap_id, log, exit_of, expect_cmd, P, tool)

    # ---------------------------------------------------------------- commands
    norm_argv = lambda a: [norm(x) for x in a]  # noqa: E731
    for tag in sorted(set(cmds) | set(exp_cmds)):
        if tag not in exp_cmds:
            c.bad(f"commands: unexpected command {tag}")
        elif tag not in cmds:
            c.bad(f"commands: expected command {tag} was not run")
        elif norm_argv(cmds[tag].get("argv") or []) != norm_argv(exp_cmds[tag]):
            c.bad(f"commands: {tag} did not run the expected command line")
    c.facts["commands"] = len(cmds)

    # ---------------------------------------------------------------- Bootstrap Contract record
    # Only a contract-eligible attempt (record_eligible) by a hash-pinned executor yields a v4 qualification
    # record; the canonical gate's verdict on it is part of this checker's verdict. A reproduction build
    # (unpinned binary whose identity rests on the operator's own build) yields reproduction identities for the
    # Seal, never a record. S2 and c2c3 attempts are narrower engineering claims: no record, no Seal identities.
    c.facts["v4_record"] = None
    if not c.violations and record_eligible(c.facts):
        if contract_record_allowed(c.facts, repro):
            record = bootstrap_record(c)
            c.facts["v4_record"] = record
            for e in contract_check(record):
                c.bad(f"contract: {e}")
        else:
            c.facts["seal_reproduction"] = seal_reproduction(c)

    # ---------------------------------------------------------------- inventory
    for tag in cmds:
        c.expected.update({f"logs/{tag}.stdout", f"logs/{tag}.stderr"})
    present = set()
    for p in c.root.rglob("*"):
        if p.is_symlink() or (p.is_dir() and getattr(p, "is_junction", lambda: False)()):
            c.bad(f"link in attempt: {p.relative_to(c.root).as_posix()}")
        if p.is_file():
            present.add(p.relative_to(c.root).as_posix())
    extra = present - c.expected - set(OUTPUTS) - {"preflight.json", "build.json", "verify.json", "differential.json"}
    if extra:
        c.bad(f"inventory: {len(extra)} unexpected file(s): {sorted(extra)[:5]}")
    missing = {x for x in c.expected if x not in present}
    if missing:
        c.bad(f"inventory: {len(missing)} missing file(s): {sorted(missing)[:5]}")
    return c


def check_differential(c, d, gens, snapshot, snap_id, log, exit_of, expect_cmd, P, tool):
    n_before = len(c.violations)
    repo_cases = json.loads((REPO / "tests/compiler/cases.json").read_text(encoding="utf-8"))
    packaged = c.read("corpus/cases.json")
    if packaged is None or json.loads(packaged) != repo_cases:
        c.bad("differential: packaged corpus differs from the repository's tests/compiler/cases.json")
    expected = {f"case.{n}": (e, "tests/compiler/cases", [f"{n}.sm"]) for n, e in repo_cases.items()}
    for n, e in IMPORT_EXPECT.items():
        dd = REPO / "tests/compiler/imports" / n
        expected[f"import.{n}"] = (e, f"tests/compiler/imports/{n}", sorted(p.name for p in dd.iterdir() if p.suffix == ".sm"))
    rows = d.get("rows", [])
    ids = [r.get("id") for r in rows]
    if sorted(ids) != sorted(expected) or len(set(ids)) != len(ids):
        c.bad("differential: fixture inventory differs from the repository corpus (missing, extra or duplicate rows)")
    gl = d.get("generations") or []
    if gl != ["C1", "C2"] or any(g not in gens for g in gl):
        c.bad("differential: the corpus must be compiled by exactly C1 and C2")
        return
    fenv = PINS["envelopes"]["fixture"]
    penv = PINS["envelopes"]["program"]
    artifacts = {}  # hash -> set of attempt-relative paths holding exactly those bytes
    for r in rows:
        rid = r.get("id", "?")
        exp, src_dir, src_files = expected.get(rid, (None, None, []))
        if r.get("expect") != exp:
            c.bad(f"differential: {rid}: expectation differs from the repository corpus")
        if sorted((r.get("file_sha256") or {})) != src_files:
            c.bad(f"differential: {rid}: fixture file set differs from the repository")
        for f in src_files:
            fb = c.read(f"fixtures/{rid}/root/{f}")
            repo_bytes = (REPO / src_dir / f).read_bytes()
            if fb != repo_bytes or (r.get("file_sha256") or {}).get(f) != sha(repo_bytes):
                c.bad(f"differential: {rid}: fixture file {f} differs from the repository")
        entry = (r.get("c0") or {}).get("artifact")
        src_entry = src_files[0] if rid.startswith("case.") and src_files else "main.sm"
        expect_cmd(f"diff.{rid}.c0", [tool("smc"), "compile", P(f"fixtures/{rid}/root/{src_entry}"), "-o", P(f"fixtures/{rid}/c0.smc")])
        ref = None
        if exit_of(f"diff.{rid}.c0") == 0:
            if entry != f"fixtures/{rid}/c0.smc":
                c.bad(f"differential: {rid}: C0 artifact missing")
            ref = c.read(f"fixtures/{rid}/c0.smc")
            if ref is not None:
                artifacts.setdefault(sha(ref), set()).add(f"fixtures/{rid}/c0.smc")
                sidecar(c, f"fixtures/{rid}/c0.smc", ref)
        elif entry:
            c.bad(f"differential: {rid}: C0 artifact recorded for a failed compilation")
        outs = {}
        for gname in gl + [gl[-1] + ".repeat"]:
            base = gname.split(".")[0]
            tag = f"diff.{rid}.{gname}"
            expect_cmd(tag, [tool("executor"), "run-smc", P(f"gen/{base}.smc"), "--expect-sha256", "sha256:" + gens[base]["sha256"],
                             "--profile", "cli-read-only", "--root", P(f"fixtures/{rid}/root"), "--envelope", "trusted-compiler",
                             "--max-steps", fenv["max_steps"], "--max-calls", fenv["max_calls"], "--max-frames", fenv["max_frames"],
                             "--metrics", "json", "--", src_entry])
            so = log(tag)
            cl = classify(so)
            cl["provenance"] = provenance(log(tag, "stderr")) or {}
            cl["exit"] = exit_of(tag)
            if not ran(cl["provenance"], gens[base]["sha256"]):
                c.bad(f"differential: {rid}/{gname}: the run is not bound to generation {base}")
            if cl["kind"] == "semcode":
                rel = f"fixtures/{rid}/{gname}.smc"
                if c.read(rel) != cl["semcode"]:
                    c.bad(f"differential: {rid}/{gname}: stored SemCode is not the compiler's stdout")
                artifacts.setdefault(sha(cl["semcode"]), set()).add(rel)
            rec = (r.get("gens") or {}).get(gname) or {}
            if rec.get("kind") != cl["kind"] or rec.get("stdout_sha256") != sha(so):
                c.bad(f"differential: {rid}/{gname}: recorded outcome differs from the raw stdout")
            outs[gname] = cl
        fails = recompute_row(exp, ref, outs, gl, exit_of, log)
        if bool(r.get("pass")) != (not fails):
            c.bad(f"differential: {rid}: recorded verdict {r.get('pass')} disagrees with the evidence {fails}")
        if fails:
            c.bad(f"differential: {rid}: fails: {fails}")
    execs = d.get("executions") or {}
    if sorted(execs) != sorted(artifacts):
        c.bad("differential: executed artifact set differs from the distinct SemCode artifacts")
    for h, paths in artifacts.items():
        rel = (execs.get(h) or {}).get("artifact")
        if rel not in paths:
            c.bad(f"differential: execution of {h[:16]} is not bound to a file holding those bytes")
            continue
        expect_cmd(f"exec.{h[:16]}.verify", [tool("smc"), "verify", P(rel)])
        if exit_of(f"exec.{h[:16]}.verify") == 0:
            expect_cmd(f"exec.{h[:16]}.run_smc", [tool("smc"), "run-smc", P(rel)])
            expect_cmd(f"exec.{h[:16]}.executor_pure", [tool("executor"), "run-smc", P(rel), "--expect-sha256", f"sha256:{h}", "--profile", "pure",
                                                        "--root", P("fixtures"), "--envelope", "trusted-compiler", "--max-steps", penv["max_steps"],
                                                        "--max-calls", penv["max_calls"], "--max-frames", penv["max_frames"], "--metrics", "json"])
            pure = provenance(log(f"exec.{h[:16]}.executor_pure", "stderr")) or {}
            if not ran(pure, h):
                c.bad(f"differential: the executor run of {h[:16]} did not execute the declared artifact")
    muts = {m.get("name"): m for m in (d.get("source_mutations") or [])}
    if sorted(muts) != sorted(MUTATIONS):
        c.bad("differential: source mutation set differs from the pinned mutations")
    for name, (_, _, _, expect) in MUTATIONS.items():
        m = muts.get(name) or {}
        want_src = mutated_sources(snapshot, name)
        msrc = {f: (c.read(f"mutations/{name}/src/{f}") or b"") for f in snapshot}
        if msrc != want_src:
            c.bad(f"differential: mutation {name}: sources are not exactly the pinned edit of the snapshot")
        mid = frame_identity([("compiler/" + f, msrc[f]) for f in snapshot])
        expect_cmd(f"mutation.{name}.compile", [tool("smc"), "compile", P(f"mutations/{name}/src/main.sm"), "-o", P(f"mutations/{name}/C1.smc")])
        art = c.read(f"mutations/{name}/C1.smc") if exit_of(f"mutation.{name}.compile") == 0 else None
        if art is not None:
            sidecar(c, f"mutations/{name}/C1.smc", art)
        same = art is not None and "C1" in gens and art == gens["C1"]["data"]
        ok = art is not None and mid != snap_id and same == (expect == "same_artifact")
        if m.get("expect") != expect or m.get("source_set_identity") != mid or bool(m.get("pass")) != ok or not ok:
            c.bad(f"differential: source mutation {name} is not proven")
    if (d.get("status") == "pass") != (len(c.violations) == n_before):
        c.bad("differential: recorded status disagrees with the evidence")
    c.facts["fixtures"] = len(rows)
    c.facts["distinct_artifacts"] = len(artifacts)
    # corpus identity and case classes for the Bootstrap Contract record (rows were bound to the repository above)
    inventory = sorted([r.get("id"), r.get("expect"), sorted((r.get("file_sha256") or {}).items())] for r in rows)
    c.facts["input_corpus"] = "sha256:" + sha(json.dumps(inventory, separators=(",", ":")).encode("utf-8"))
    c.facts["case_classes"] = {
        "positive": sum(1 for r in rows if r.get("expect") == "ok"),
        "negative": sum(1 for r in rows if str(r.get("expect")).startswith(("reject:", "fault:"))),
        "boundary": sum(1 for r in rows if str(r.get("expect")).startswith("trap:"))}


def recompute_row(expect, ref, outs, gl, exit_of, log):
    f = []
    if not gl or not expect:
        return ["no generations or expectation"]
    main = [outs.get(g, {}) for g in gl]
    rep = outs.get(gl[-1] + ".repeat", {})
    key = lambda o: (o.get("kind"), o.get("semcode"), o.get("category"), o.get("message"))  # noqa: E731
    if len({key(o) for o in main}) != 1:
        f.append("generations disagree")
    if key(rep) != key(main[-1]):
        f.append("nondeterministic")
    for o in main + [rep]:
        k = o.get("kind")
        drv = o.get("provenance") or {}
        completed = o.get("exit") == 0 and drv.get("completed") is True
        if expect == "ok" or expect.startswith("trap:"):
            if k != "semcode" or not completed:
                f.append("no SemCode from a completed compiler run")
                continue
            if ref is None or o["semcode"] != ref:
                f.append("SemCode differs from C0")
            h = sha(o["semcode"])[:16]
            if exit_of(f"exec.{h}.verify") != 0:
                f.append("not admitted")
                continue
            run_exit = exit_of(f"exec.{h}.run_smc")
            run_text = (log(f"exec.{h}.run_smc") + log(f"exec.{h}.run_smc", "stderr")).decode("utf-8", "replace")
            pure = provenance(log(f"exec.{h}.executor_pure", "stderr")) or {}
            if expect == "ok":
                if run_exit != 0 or exit_of(f"exec.{h}.executor_pure") != 0 or pure.get("completed") is not True:
                    f.append("run failed")
            else:
                frag = expect.split(":", 1)[1]
                if run_exit in (0, None) or frag not in run_text:
                    f.append("no expected trap on smc run-smc")
                if pure.get("phase_reached") != "execute" or pure.get("outcome") is None or pure.get("completed") or frag not in (pure.get("error") or ""):
                    f.append("no expected trap on the executor")
        elif expect.startswith("reject:"):
            _, cat, frag = expect.split(":", 2)
            if k != "diagnostic" or o.get("category") != cat or frag not in o.get("message", "") or not completed:
                f.append("no expected diagnostic")
        elif expect.startswith("fault:"):
            frag = expect.split(":", 1)[1]
            if k == "semcode" or k == "diagnostic" or drv.get("outcome") != "runtime_error" or frag not in (drv.get("error") or ""):
                f.append("no expected process fault")
        else:
            f.append("unknown expectation")
    return sorted(set(f))


PLATFORMS = {("win32", "AMD64"): "x86_64-pc-windows-msvc"}
LIMITATIONS = [
    "one host and one operator produced this record; independent reproduction is recorded in the Seal candidate, not here",
    "platform scope is the C0 platform only (no cross-platform claim; SHF-17 is separate)",
    "C0 parity and the fixed point are shown for the qualification corpus and S; constructs outside them are not covered",
    "the --expect-sha256 binding proves which bytes ran, not who produced them; no signing",
    "VM quotas bound steps, calls and frames; memory and wall-clock are not bounded by the protocol",
]


def contract_record_allowed(f, reproduction_mode):
    """A v4 contract record needs a normal (non-reproduction) attempt whose C0 smc and executor binaries are both
    hash-pinned. Every reproduction-mode attempt yields Seal reproduction identities only."""
    return (not reproduction_mode and bool((f.get("executor") or {}).get("binary_pinned"))
            and bool((f.get("c0_smc") or {}).get("binary_pinned")))


def record_eligible(f):
    """The v4 protocol attempt: full scope, rule c1c2 and every stored generation executed by smc run-smc."""
    return f.get("scope") == "full" and f.get("fixed_point_rule") == "c1c2" and f.get("generation_runner") == "smc-run-smc"


def bootstrap_record(c):
    """The Bootstrap Contract evidence record, built only from facts this checker verified (no recorded verdict
    is copied). Returns None unless the attempt passed in full scope."""
    f = c.facts
    if f.get("scope") != "full" or not f.get("c1_equals_c2") or "C2" not in (f.get("generations") or {}):
        return None
    pre = json.loads((c.root / "preflight.json").read_text(encoding="utf-8"))
    plat = pre.get("platform") or {}
    run = (f.get("counters") or {}).get("C2") or {}
    quotas = (f.get("effective_quotas") or {}).get("C2") or {}
    ex = f.get("executor") or {}
    cls = f.get("case_classes") or {}
    revision = ex.get("revision") or ""
    return {
        "contract_protocol": "shf0-bootstrap-contract-v4", "comparison_rule": "byte-equality-v1",
        "c0_identity": f"{PINS['c0']['repository']}@{PINS['c0']['sha']}",
        "source_set_identity": f.get("source_set_identity"),
        "c1_artifact_hash": "sha256:" + f["generations"]["C1"]["sha256"],
        "c2_artifact_hash": "sha256:" + f["generations"]["C2"]["sha256"],
        "c1_verifier_binding": "admitted", "c2_verifier_binding": "admitted", "comparison_result": "equal",
        "platform": PLATFORMS.get((plat.get("sys_platform"), plat.get("machine")), f"{plat.get('machine')}-{plat.get('sys_platform')}"),
        "verifier_contract": f"{revision}:crates/sm-verify", "runtime_contract": f"{revision}:crates/sm-vm",
        "remaining_rust_responsibilities": ["oracle", "host_mechanics", "verifier_runtime_foundation"],
        "input_corpus": f.get("input_corpus"), "input_corpus_size": f.get("fixtures"),
        "positive_cases": {"count": cls.get("positive"), "result": "pass"},
        "negative_cases": {"count": cls.get("negative"), "result": "pass"},
        "boundary_cases": {"count": cls.get("boundary"), "result": "pass"},
        "mutation_proof": "detected", "unexplained_deltas": 0, "limitations": LIMITATIONS,
        "c0_checkout_head": (pre.get("c0_checkout") or {}).get("head"),
        "c0_checkout_clean": (pre.get("c0_checkout") or {}).get("clean"),
        "executor_identity": f"{revision}:crates/smc-cli",
        "effective_quotas": {k: quotas.get(k) for k in ("max_steps", "max_calls", "max_frames")},
        "c1_execution": {"outcome": (f.get("outcomes") or {}).get("C2"),
                         "counters": {k: run.get(k) for k in ("steps", "calls", "peak_frames")}},
        "determinism": "identical",
    }


def seal_reproduction(c):
    """Identities a reproduction contributes to a Seal candidate (BOOTSTRAP_CONTRACT §10.9). host_id and
    operator are added by the reproducing operator; the executor binary is operator-attested."""
    f = c.facts
    return {"c0_identity": f"{PINS['c0']['repository']}@{PINS['c0']['sha']}",
            "source_set_identity": f.get("source_set_identity"),
            "c1_artifact_hash": "sha256:" + f["generations"]["C1"]["sha256"],
            "c2_artifact_hash": "sha256:" + f["generations"]["C2"]["sha256"],
            "comparison_result": "equal" if f.get("c1_equals_c2") else "different",
            "executor": f.get("executor"), "c0_smc": f.get("c0_smc"),
            "executor_binary": "operator-built from the pinned revision (not hash-pinned)" if not (f.get("executor") or {}).get("binary_pinned") else "pinned",
            "c0_binary": "operator-built from the C0 SHA (admitted by source fingerprint)" if not (f.get("c0_smc") or {}).get("binary_pinned") else "pinned"}


def contract_check(record):
    """Runs the canonical gate bootstrap/validate_contract.py check_evidence (read, never modified)."""
    import importlib.util
    spec = importlib.util.spec_from_file_location("bootstrap_validate_contract", REPO / "bootstrap" / "validate_contract.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.check_evidence(REPO, record)


def write_outputs(c):
    ok = not c.violations
    f = c.facts
    success = ok and f.get("scope") == "full" and f.get("fixed_point_rule") == "c1c2" and f.get("c1_equals_c2") and f.get("c2_equals_c3")
    record = f.get("v4_record")
    if record is not None:
        record_status = "valid" if ok else "rejected (see violations)"
    else:
        record_status = ("no record: reproduction build (see facts.seal_reproduction)" if f.get("seal_reproduction")
                         else "no record: the attempt did not pass" if not ok
                         else "no record: not a contract-eligible attempt (needs full scope, rule c1c2, smc run-smc)")
    rep = {"schema": "semantic.selfhost.report/1", "attempt": str(c.root), "verdict": "PASS" if ok else "FAIL",
           "engineering_self_host_success": bool(success), "v4_qualification_record": bool(ok and record is not None),
           "violations": c.violations, "facts": f, "bootstrap_record": record, "bootstrap_record_check_evidence": record_status}
    (c.root / "report.json").open("x", encoding="utf-8").write(json.dumps(rep, indent=1, sort_keys=True) + "\n")
    lines = [f"# Self-host attempt report: {rep['verdict']} (scope {f.get('scope')}, fixed-point rule {f.get('fixed_point_rule')})", "",
             f"- engineering self-host success (full scope, C1 == C2 == C3): {rep['engineering_self_host_success']}",
             f"- source-set identity: {f.get('source_set_identity')} (repository compiler: {f.get('snapshot_is_repository_compiler')})"]
    for k, g in (f.get("generations") or {}).items():
        lines.append(f"- {k}: sha256 {g['sha256']} ({g['bytes']} bytes); counters {(f.get('counters') or {}).get(k)}")
    lines += [f"- C1 == C2 (byte-equality-v1): {f.get('c1_equals_c2')}", f"- C2 == C3: {f.get('c2_equals_c3')}",
              f"- fixtures: {f.get('fixtures')}; distinct SemCode artifacts: {f.get('distinct_artifacts')}",
              f"- Bootstrap Contract v4 record (check_evidence): {record_status}",
              f"- violations: {len(c.violations)}"] + [f"  - {v}" for v in c.violations[:50]]
    (c.root / "summary.md").open("x", encoding="utf-8").write("\n".join(lines) + "\n")


def mutate_json(path, fn):
    obj = json.loads(path.read_text(encoding="utf-8"))
    fn(obj)
    path.write_text(json.dumps(obj, indent=1, sort_keys=True) + "\n", encoding="utf-8")


def rewrite_log(root, tag, stream, data):
    (root / "logs" / f"{tag}.{stream}").write_bytes(data)
    out = []
    for line in (root / "commands.jsonl").read_text().splitlines():
        e = json.loads(line)
        if e["tag"] == tag:
            e[f"{stream}_sha256"] = sha(data)
        out.append(json.dumps(e, sort_keys=True))
    (root / "commands.jsonl").write_text("\n".join(out) + "\n")


def edit_command(root, tag, fn):
    out = []
    for line in (root / "commands.jsonl").read_text().splitlines():
        e = json.loads(line)
        if e["tag"] == tag:
            fn(e)
        out.append(json.dumps(e, sort_keys=True))
    (root / "commands.jsonl").write_text("\n".join(out) + "\n")


def add_extra_command(root):
    """A well-formed extra command (hashed logs present): only the exact command-set rule can reject it."""
    for stream in ("stdout", "stderr"):
        (root / "logs" / f"extra.cmd.{stream}").write_bytes(b"")
    entry = {"tag": "extra.cmd", "argv": ["x"], "exit": 0, "stdout_sha256": sha(b""), "stderr_sha256": sha(b"")}
    with open(root / "commands.jsonl", "a", encoding="utf-8") as f:
        f.write(json.dumps(entry, sort_keys=True) + "\n")


def selftest(src, scratch):
    src, scratch = pathlib.Path(src).resolve(), pathlib.Path(scratch).resolve()
    if scratch.exists():
        sys.exit("selftest: scratch directory must not exist")
    base = check(src)
    if base.violations:
        sys.exit(f"selftest: the source attempt does not pass ({len(base.violations)} violations)")
    diff = json.loads((src / "differential.json").read_text())
    first_ok = next(r for r in diff["rows"] if r["expect"] == "ok")["id"]
    trap_row = next(r for r in diff["rows"] if r["expect"].startswith("trap:"))
    trap_h = trap_row["c0"]["sha256"][:16]
    ok_h = next(r for r in diff["rows"] if r["expect"] == "ok")["c0"]["sha256"][:16]
    s2 = (src / "s2").is_dir()

    def flip_byte(p):
        b = bytearray(p.read_bytes()); b[len(b) // 2] ^= 1; p.write_bytes(bytes(b))

    def drop_case(r):
        mutate_json(r / "corpus" / "cases.json", lambda o: o.pop(first_ok[5:]))
        mutate_json(r / "differential.json", lambda o: o.__setitem__("rows", [x for x in o["rows"] if x["id"] != first_ok]))

    def wrong_generation(r):
        # the C2 run of a fixture actually executed C1 (its command line names C1 and C1's identity)
        tag = f"diff.{first_ok}.C2"
        gens_ = json.loads((r / "build.json").read_text())["generations"]
        c1, c2 = gens_[0]["sha256"], gens_[1]["sha256"]
        edit_command(r, tag, lambda e: e.__setitem__("argv", [a.replace("C2.smc", "C1.smc").replace(c2, c1) for a in e["argv"]]))

    injections = {
        "artifact_byte_flip": lambda r: flip_byte(r / "gen" / "C2.smc"),
        "missing_log": lambda r: (r / "logs" / "build.C1.verify.stdout").unlink(),
        "extra_file": lambda r: (r / "gen" / "C9.smc").write_bytes(b"x"),
        "verify_claim_flip": lambda r: mutate_json(r / "verify.json", lambda o: o["comparisons"][0].__setitem__("equal", not o["comparisons"][0]["equal"])),
        "tool_identity": lambda r: mutate_json(r / "preflight.json", lambda o: o["tools"]["smc"].__setitem__("sha256", "0" * 64)),
        "c0_dirty_log": lambda r: rewrite_log(r, "preflight.c0_status", "stdout", b" M crates/sm-vm/src/lib.rs\n"),
        "verifier_rejects": lambda r: edit_command(r, "build.C2.verify", lambda e: e.__setitem__("exit", 1)),
        "verify_ran_other_command": lambda r: edit_command(r, "build.C2.verify", lambda e: e.__setitem__("argv", [e["argv"][0], "version"])),
        "exit_zero_but_diagnostic": lambda r: rewrite_log(r, f"diff.{first_ok}.C2", "stdout", b"error type @0: injected\n"),
        "fixture_run_wrong_generation": wrong_generation,
        "row_removed": lambda r: mutate_json(r / "differential.json", lambda o: o["rows"].pop()),
        "row_duplicated": lambda r: mutate_json(r / "differential.json", lambda o: o["rows"].append(o["rows"][0])),
        "corpus_case_dropped": drop_case,
        "expectation_rewritten": lambda r: mutate_json(r / "differential.json", lambda o: o["rows"][0].__setitem__("expect", "reject:type:x")),
        "import_expectation_rewritten": lambda r: mutate_json(r / "differential.json", lambda o: next(x for x in o["rows"] if x["id"] == "import.positive").__setitem__("expect", "reject:type:x")),
        "mutation_expectation_flipped": lambda r: mutate_json(r / "differential.json", lambda o: o["source_mutations"][0].__setitem__("expect", "different_artifact")),
        "mutation_source_altered": lambda r: (r / "mutations" / "diagnostic_text" / "src" / "lower.sm").write_bytes(b"// other\n" + (r / "mutations" / "diagnostic_text" / "src" / "lower.sm").read_bytes()),
        # run-smc reports traps on stderr: replace both streams so the expected trap text is really gone
        "trap_log_altered": lambda r: [rewrite_log(r, f"exec.{trap_h}.run_smc", s, b"runtime trap: SomethingElse\n") for s in ("stdout", "stderr")],
        "snapshot_mutated": lambda r: flip_byte(r / "snapshot" / "lexer.sm"),
        "status_forged": lambda r: mutate_json(r / "differential.json", lambda o: o["rows"][0].__setitem__("pass", False)),
        # the run of an artifact that must execute cleanly (an ok fixture), not of a trap artifact
        "vm_run_failed": lambda r: edit_command(r, f"exec.{ok_h}.run_smc", lambda e: e.__setitem__("exit", 3)),
        "differential_deleted": lambda r: (r / "differential.json").unlink(),
        "differential_null": lambda r: (r / "differential.json").write_text("null\n"),
        "differential_single_generation": lambda r: mutate_json(r / "differential.json", lambda o: o.__setitem__("generations", ["C2"])),
        "unexpected_command": add_extra_command,
        # the producing run of C2 did not complete (its own metrics say so) although stdout is intact
        "executor_not_completed": lambda r: rewrite_log(r, "build.C2.execute", "stderr", (r / "logs" / "build.C2.execute.stderr").read_bytes().replace(b'"outcome":"ok"', b'"outcome":"trap"')),
        # the producing run of C2 used a smaller envelope than the declared one
        "executor_envelope_changed": lambda r: rewrite_log(r, "build.C2.execute", "stderr", (r / "logs" / "build.C2.execute.stderr").read_bytes().replace(b'"max_calls":2000000', b'"max_calls":1999999')),
        "executor_fingerprint_forged": lambda r: rewrite_log(r, "preflight.executor_version", "stdout", (r / "logs" / "preflight.executor_version.stdout").read_bytes().replace(b'"source_fingerprint": "', b'"source_fingerprint": "0')),
        "c0_fingerprint_forged": lambda r: rewrite_log(r, "preflight.smc_version", "stdout", (r / "logs" / "preflight.smc_version.stdout").read_bytes().replace(b'"source_fingerprint": "', b'"source_fingerprint": "0')),
        "executor_admission_missing": lambda r: edit_command(r, "build.C2.executor_verify", lambda e: e.__setitem__("exit", 1)),
        # an otherwise clean attempt whose contract record the canonical gate rejects (platform outside C0's scope)
        "record_rejected_by_contract": lambda r: mutate_json(r / "preflight.json", lambda o: o["platform"].__setitem__("machine", "ARM64")),
    }
    if s2:
        injections["s2_record_not_accepted"] = lambda r: mutate_json(r / "s2" / "build.C2.execute" / "s2_record.json", lambda o: o.__setitem__("accepted", False))
        injections["s2_stray_file"] = lambda r: (r / "s2" / "build.C2.execute" / "stray.txt").write_bytes(b"x")
        injections["s2_verify_other_attempt"] = lambda r: edit_command(r, "build.C2.execute.verify_attempt", lambda e: e.__setitem__("argv", [e["argv"][0], "--verify-attempt", e["argv"][2].replace("C2", "C3")]))
    results = {}
    scratch.mkdir()
    for name, inject in injections.items():
        d = scratch / name
        shutil.copytree(src, d, ignore=shutil.ignore_patterns(*OUTPUTS))
        inject(d)
        v = check(d).violations
        results[name] = {"detected": bool(v), "violations": v[:3]}
    clean = scratch / "unmodified"
    shutil.copytree(src, clean, ignore=shutil.ignore_patterns(*OUTPUTS))
    results["unmodified_copy"] = {"detected": bool(check(clean).violations)}
    ok = all(r["detected"] for k, r in results.items() if k != "unmodified_copy") and not results["unmodified_copy"]["detected"]
    out = {"schema": "semantic.selfhost.checker_selftest/1", "source": str(src), "injections": len(injections), "result": "PASS" if ok else "FAIL", "cases": results}
    (scratch / "selftest.json").open("x", encoding="utf-8").write(json.dumps(out, indent=1) + "\n")
    print(json.dumps({k: v["detected"] for k, v in results.items()}, indent=1))
    print(f"checker self-test: {out['result']} ({len(injections)} injections)")
    return 0 if ok else 1


def main(argv):
    if argv and argv[0] == "--selftest":
        return selftest(argv[1], argv[2])
    c = check(argv[0], partial="--partial" in argv)
    if "--write" in argv:
        write_outputs(c)
    print(f"check_results: {'PASS' if not c.violations else 'FAIL'} ({len(c.violations)} violations, scope {c.facts.get('scope')})")
    for v in c.violations[:20]:
        print("  - " + v)
    return 0 if not c.violations else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
