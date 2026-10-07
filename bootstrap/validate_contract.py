#!/usr/bin/env python3
"""SHF-0 bootstrap contract validator (issue #11).

Validates bootstrap/contract.toml, bootstrap/source-set.toml and their links to
reference/semantic-reference.toml and docs/BOOTSTRAP_SUBSET.md, and computes the
source-set identity of S.

Host-side harness only: it never tokenizes, parses, typechecks or lowers Semantic and never
constructs SemCode. It reads declared bytes and checks protocol rules.

Stdlib only (Python >= 3.11). Exit 0 = valid, 1 = violation. --self-test runs mutation tests.
"""
import copy
import hashlib
import json
import re
import struct
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = "bootstrap/contract.toml"

PROTOCOL = "shf0-bootstrap-contract-v1"
# Canonical digest of every frozen value of each protocol version (see frozen_digest).
FROZEN_DIGESTS = {PROTOCOL: "f5384570ad2c3d55e9199e3c83db906d0bfa60825c0972d572be2b0fe50fe41c"}
SOURCE_PROTOCOL = "shf0-source-set-v1"
REPOSITORY = "skulmakov-oss/Semantic"
KNOWN_COMPARISON_RULES = {"byte-equality-v1"}
REQUIRED_FAILURES = {
    "REFERENCE_MISMATCH", "CAPABILITY_GAP", "ADMISSION_REJECT", "FIXED_POINT_DELTA",
    "NONDETERMINISM", "HOST_LOGIC_LEAK", "REFERENCE_DEFECT", "SOURCE_SET_INVALID",
    "CONTRACT_DRIFT",
}
REQUIRED_FORBIDDEN_LOGIC = {
    "tokenization", "parsing", "symbol_binding", "typechecking", "ir_selection", "lowering",
    "opcode_selection", "semcode_construction", "diagnostic_meaning", "compiler_ordering",
}
REQUIRED_FORBIDDEN_INPUTS = {
    "wall_clock", "random_state", "filesystem_enumeration_order", "undeclared_env_vars",
    "absolute_host_paths", "process_or_thread_identity", "locale", "unrecorded_global_state",
}
SHA = re.compile(r"[0-9a-f]{40}")
COMPONENT = re.compile(r"[a-z0-9_]+")
SHF_STAGE = re.compile(r"SHF-(\d+)")
BSF_ROW = re.compile(r"^\|\s*(BSF-\d{3})\s*\|[^|]*\|\s*(CANDIDATE|ADMITTED|FROZEN)\s*\|", re.M)
DOMAIN = b"SHF0-SOURCE-SET\x00v1\x00"


# ------------------------------------------------------------------ source set
def check_paths(files, root):
    """Path rules of shf0-source-set-v1. Returns ['SOURCE_SET_INVALID: ...']."""
    errors = []
    seen = set()
    for p in files:
        if not isinstance(p, str) or not p.isascii():
            errors.append(f"SOURCE_SET_INVALID: non-ASCII or non-string path {p!r}")
            continue
        if "\\" in p or p.startswith("/") or re.match(r"^[A-Za-z]:", p):
            errors.append(f"SOURCE_SET_INVALID: absolute or non-POSIX path {p!r}")
            continue
        parts = p.split("/")
        if parts[0] != root:
            errors.append(f"SOURCE_SET_INVALID: path outside root {root!r}: {p!r}")
            continue
        if not p.endswith(".sm"):
            errors.append(f"SOURCE_SET_INVALID: not a .sm source file: {p!r}")
            continue
        stems = parts[:-1] + [parts[-1][:-3]]
        if any(not COMPONENT.fullmatch(c) for c in stems):
            errors.append(f"SOURCE_SET_INVALID: non-canonical path component in {p!r}")
            continue
        if p.lower() in seen:
            errors.append(f"SOURCE_SET_INVALID: duplicate path {p!r}")
        seen.add(p.lower())
    encoded = [p.encode() for p in files if isinstance(p, str)]
    if encoded != sorted(encoded) or len(set(encoded)) != len(encoded):
        errors.append("SOURCE_SET_INVALID: manifest is not strictly ascending by UTF-8 bytes")
    return errors


def check_content(path, data):
    errors = []
    if data.startswith(b"\xef\xbb\xbf"):
        errors.append(f"SOURCE_SET_INVALID: {path}: UTF-8 BOM")
    if b"\r" in data:
        errors.append(f"SOURCE_SET_INVALID: {path}: CR byte (only LF newlines are canonical)")
    if b"\x00" in data:
        errors.append(f"SOURCE_SET_INVALID: {path}: NUL byte")
    if not data.endswith(b"\n"):  # an empty file has no final newline either
        errors.append(f"SOURCE_SET_INVALID: {path}: missing final newline")
    try:
        data.decode("utf-8")
    except UnicodeDecodeError:
        errors.append(f"SOURCE_SET_INVALID: {path}: not valid UTF-8")
    return errors


def source_set_identity(entries):
    """sha256 over the framed stream; entries = [(path, content_bytes)] in manifest order."""
    h = hashlib.sha256(DOMAIN)
    h.update(struct.pack(">Q", len(entries)))
    for path, content in entries:
        p = path.encode()
        h.update(struct.pack(">I", len(p)) + p)
        h.update(struct.pack(">Q", len(content)) + content)
    return "sha256:" + h.hexdigest()


def load_source_set(repo, manifest, root):
    """Validate the manifest and file bytes; return (identity_or_None, errors)."""
    if manifest.get("protocol") != SOURCE_PROTOCOL:
        return None, [f"SOURCE_SET_INVALID: protocol must be {SOURCE_PROTOCOL!r}"]
    if manifest.get("root") != root:
        return None, [f"SOURCE_SET_INVALID: root must be {root!r}"]
    files = manifest.get("files")
    if not isinstance(files, list):
        return None, ["SOURCE_SET_INVALID: files must be a list"]
    errors = check_paths(files, root)
    if errors:
        return None, errors
    entries = []
    for p in files:
        f = repo / p
        if not f.is_file() or f.is_symlink():
            errors.append(f"SOURCE_SET_INVALID: {p}: missing or not a regular file")
            continue
        data = f.read_bytes()
        errors += check_content(p, data)
        entries.append((p, data))
    return (None if errors else source_set_identity(entries)), errors


# ------------------------------------------------------------------ contract
def check_c0(contract, reference):
    errors = []
    c0 = contract.get("c0", {})
    sha = str(c0.get("sha", ""))
    if c0.get("repository") != REPOSITORY or reference.get("repository") != REPOSITORY:
        errors.append(f"REFERENCE_MISMATCH: C0 repository must be {REPOSITORY!r}")
    if not SHA.fullmatch(sha):
        errors.append(f"REFERENCE_MISMATCH: C0 must be an exact 40-hex SHA, not {sha!r}")
    if sha != reference.get("sha"):
        errors.append("REFERENCE_MISMATCH: contract C0 SHA differs from the reference manifest")
    if reference.get("status") != c0.get("required_reference_status") \
            or reference.get("status") != "qualified-reference":
        errors.append("REFERENCE_MISMATCH: C0 reference is not a qualified-reference")
    drift = reference.get("drift", {})
    if sha and sha == drift.get("planning_reference") and not drift.get("inherits_qualification"):
        errors.append("REFERENCE_MISMATCH: C0 is the unqualified development main (floating main as oracle)")
    return errors


def check_capabilities(contract):
    errors = []
    caps = contract.get("capabilities", {})
    for name, cap in caps.items():
        if not isinstance(cap.get("permitted"), bool):
            errors.append(f"CAPABILITY_GAP: capability {name!r} must declare permitted = true|false")
            continue
        if cap["permitted"]:
            if not isinstance(cap.get("available_at_c0"), bool):
                errors.append(f"CAPABILITY_GAP: capability {name!r} must declare available_at_c0")
            elif not cap["available_at_c0"]:
                stage = SHF_STAGE.fullmatch(str(cap.get("gap_stage", "")))
                if cap.get("gap_owner") != REPOSITORY or not stage or not 1 <= int(stage.group(1)) <= 17:
                    errors.append(f"CAPABILITY_GAP: unavailable capability {name!r} needs gap_owner "
                                  f"{REPOSITORY!r} and a gap_stage SHF-1..SHF-17")
    host = contract.get("host", {})
    for name in host.get("required_capabilities", []):
        if name not in caps:
            errors.append(f"CAPABILITY_GAP: required capability {name!r} is undeclared")
        elif not caps[name].get("permitted"):
            errors.append(f"HOST_LOGIC_LEAK: required capability {name!r} is not permitted by policy")
    missing = REQUIRED_FORBIDDEN_LOGIC - set(host.get("forbidden_logic", []))
    if missing:
        errors.append(f"HOST_LOGIC_LEAK: forbidden host logic not declared: {sorted(missing)}")
    if host.get("violation") != "HOST_LOGIC_LEAK":
        errors.append("HOST_LOGIC_LEAK: host violation class must be HOST_LOGIC_LEAK")
    return errors


def frozen_digest(contract):
    """sha256 of the canonical form of every frozen contract value. [subset] is excluded: it
    grows as constructions are admitted and is checked through the registry linkage instead."""
    frozen = {k: v for k, v in contract.items() if k != "subset"}
    canonical = json.dumps(frozen, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    return hashlib.sha256(canonical.encode()).hexdigest()


def check_contract(contract, reference, subset_text):
    errors = []
    if contract.get("protocol") != PROTOCOL:
        errors.append(f"CONTRACT_DRIFT: protocol must be {PROTOCOL!r}")
    elif frozen_digest(contract) != FROZEN_DIGESTS[PROTOCOL]:
        errors.append(f"CONTRACT_DRIFT: a frozen value of {PROTOCOL} changed; a contract change needs "
                      "a new protocol identifier and a recorded revision")
    errors += check_c0(contract, reference)

    fp = contract.get("fixed_point", {})
    rule = fp.get("comparison_rule")
    if rule not in KNOWN_COMPARISON_RULES or rule not in contract.get("comparison_rules", {}):
        errors.append(f"CONTRACT_DRIFT: unknown or undefined comparison rule {rule!r}")
    if fp.get("c1") != "C0(S)" or fp.get("c2") != "C1(S)":
        errors.append("CONTRACT_DRIFT: fixed point must be C1 = C0(S), C2 = C1(S)")
    if sorted(fp.get("admission_required", [])) != ["C1", "C2"] or fp.get("executed") != ["C1"]:
        errors.append("CONTRACT_DRIFT: C1 and C2 must be admitted; only C1 is executed")

    ss = contract.get("source_set", {})
    if ss.get("protocol") != SOURCE_PROTOCOL or ss.get("newline") != "lf-only" \
            or ss.get("ordering") != "strictly-ascending-utf8-bytes" or ss.get("duplicates") != "reject":
        errors.append("SOURCE_SET_INVALID: source_set rules differ from shf0-source-set-v1")
    if ss.get("identity", {}).get("algorithm") != "sha256":
        errors.append("SOURCE_SET_INVALID: identity algorithm must be sha256")

    inputs = contract.get("deterministic_inputs", {})
    missing = REQUIRED_FORBIDDEN_INPUTS - set(inputs.get("forbidden", []))
    if missing:
        errors.append(f"NONDETERMINISM: forbidden hidden inputs not declared: {sorted(missing)}")
    if contract.get("compiler", {}).get("declared_config") != []:
        errors.append("NONDETERMINISM: protocol v1 admits no configuration inputs")

    errors += check_capabilities(contract)

    failures = contract.get("failures", {})
    for name in sorted(REQUIRED_FAILURES - set(failures)):
        errors.append(f"CONTRACT_DRIFT: failure class {name} is missing")
    for name, f in failures.items():
        if not (f.get("trigger") and f.get("response")):
            errors.append(f"CONTRACT_DRIFT: failure class {name} needs trigger and response")
        if f.get("continue") is not False:  # protocol v1: every failure stops qualification
            errors.append(f"CONTRACT_DRIFT: failure class {name} must set continue = false")

    sub = contract.get("subset", {})
    admitted, candidate, frozen = (set(sub.get(k, [])) for k in ("admitted", "candidate", "frozen"))
    if admitted & candidate or admitted & frozen or candidate & frozen:
        errors.append("CONTRACT_DRIFT: a BSF id appears in more than one subset state")
    registry = {bsf: state for bsf, state in BSF_ROW.findall(subset_text)}
    declared = {**{b: "ADMITTED" for b in admitted}, **{b: "CANDIDATE" for b in candidate},
                **{b: "FROZEN" for b in frozen}}
    if registry != declared:
        errors.append(f"CONTRACT_DRIFT: subset registry and contract disagree: "
                      f"registry={sorted(registry.items())} contract={sorted(declared.items())}")
    return errors


def check_evidence(contract, record):
    """An evidence record is valid only under the exact contract it claims."""
    errors = [f"CONTRACT_DRIFT: evidence field {f!r} missing"
              for f in contract["evidence"]["required_fields"] if f not in record]
    if record.get("contract_protocol") != contract["protocol"]:
        errors.append("CONTRACT_DRIFT: evidence bound to a different contract protocol")
    if record.get("comparison_rule") != contract["fixed_point"]["comparison_rule"]:
        errors.append("CONTRACT_DRIFT: evidence bound to a different comparison rule")
    if record.get("c0_identity") != f"{contract['c0']['repository']}@{contract['c0']['sha']}":
        errors.append("CONTRACT_DRIFT: evidence bound to a different C0")
    return errors


def validate(repo):
    contract = tomllib.loads((repo / CONTRACT).read_text(encoding="utf-8"))
    reference = tomllib.loads((repo / contract["c0"]["reference_manifest"]).read_text(encoding="utf-8"))
    subset_text = (repo / contract["subset"]["registry"]).read_text(encoding="utf-8")
    manifest = tomllib.loads((repo / contract["source_set"]["manifest"]).read_text(encoding="utf-8"))
    errors = check_contract(contract, reference, subset_text)
    identity, ss_errors = load_source_set(repo, manifest, contract["source_set"]["root"])
    return identity, errors + ss_errors


# ------------------------------------------------------------------ self-test
def self_test():
    contract = tomllib.loads((ROOT / CONTRACT).read_text(encoding="utf-8"))
    reference = tomllib.loads((ROOT / "reference/semantic-reference.toml").read_text(encoding="utf-8"))
    subset = (ROOT / "docs/BOOTSTRAP_SUBSET.md").read_text(encoding="utf-8")
    assert not check_contract(contract, reference, subset), check_contract(contract, reference, subset)

    def mutated(fn, ref=False):
        c, r = copy.deepcopy(contract), copy.deepcopy(reference)
        fn(r if ref else c)
        return check_contract(c, r, subset)

    def fails(cls, fn, ref=False):
        errs = mutated(fn, ref)
        assert any(e.startswith(cls + ":") for e in errs), (cls, errs)

    # C0 identity
    fails("REFERENCE_MISMATCH", lambda c: c["c0"].update(sha="main"))                  # branch name
    fails("REFERENCE_MISMATCH", lambda c: c["c0"].update(repository="someone/Semantic"))
    fails("REFERENCE_MISMATCH", lambda c: c["c0"].update(sha="0" * 40))                # != manifest
    fails("REFERENCE_MISMATCH", lambda r: r.update(status="planning-reference"), ref=True)
    fails("REFERENCE_MISMATCH", lambda r: r.update(sha=r["drift"]["planning_reference"]), ref=True)
    fails("REFERENCE_MISMATCH", lambda c: c["c0"].update(sha=reference["drift"]["planning_reference"]))
    # protocol and comparison rule
    fails("CONTRACT_DRIFT", lambda c: c.update(protocol="shf0-bootstrap-contract-v0"))
    fails("CONTRACT_DRIFT", lambda c: c["fixed_point"].update(comparison_rule="normalized-equality-v1"))
    fails("CONTRACT_DRIFT", lambda c: c["fixed_point"].update(executed=["C1", "C2"]))   # hidden C3
    fails("CONTRACT_DRIFT", lambda c: c["fixed_point"].update(admission_required=["C2"]))
    # capabilities and host boundary
    fails("CAPABILITY_GAP", lambda c: c["host"]["required_capabilities"].append("binary_net"))
    fails("CAPABILITY_GAP", lambda c: c["capabilities"]["artifact_write"].pop("gap_stage"))
    fails("HOST_LOGIC_LEAK", lambda c: c["host"]["required_capabilities"].append("clock_read"))
    fails("HOST_LOGIC_LEAK", lambda c: c["host"]["forbidden_logic"].remove("parsing"))
    fails("NONDETERMINISM", lambda c: c["deterministic_inputs"]["forbidden"].remove("wall_clock"))
    fails("NONDETERMINISM", lambda c: c["compiler"].update(declared_config=["opt_level"]))
    # failure taxonomy and subset linkage
    for name in REQUIRED_FAILURES:
        fails("CONTRACT_DRIFT", lambda c, n=name: c["failures"].pop(n))
    fails("CONTRACT_DRIFT", lambda c: c["failures"]["NONDETERMINISM"].pop("continue"))
    fails("CONTRACT_DRIFT", lambda c: c["subset"]["admitted"].append("BSF-101"))
    fails("CONTRACT_DRIFT", lambda c: c["subset"]["admitted"].remove("BSF-001"))
    fails("SOURCE_SET_INVALID", lambda c: c["source_set"].update(newline="normalize"))
    # every frozen value is covered by the protocol digest, not only the explicitly checked ones
    fails("CONTRACT_DRIFT", lambda c: c["source_set"].update(root="src"))
    fails("CONTRACT_DRIFT", lambda c: c["source_set"].update(manifest="other.toml"))
    fails("CONTRACT_DRIFT", lambda c: c["source_set"].update(final_newline="optional"))
    fails("CONTRACT_DRIFT", lambda c: c["source_set"]["identity"].update(domain="other"))
    fails("CONTRACT_DRIFT", lambda c: c["capabilities"]["artifact_write"].update(available_at_c0=True))
    for name in REQUIRED_FAILURES:  # no failure class may allow qualification to continue
        fails("CONTRACT_DRIFT", lambda c, n=name: c["failures"][n].update({"continue": True}))
    assert not mutated(lambda c: c["subset"].update(frozen=[])), "subset is not digest-frozen"

    # source-set paths
    good = ["compiler/a.sm", "compiler/b/c.sm"]
    assert not check_paths(good, "compiler")
    for bad in (["compiler/a.sm", "compiler/a.sm"],          # duplicate
                ["compiler/a.sm", "compiler/A.sm"],          # case collision (and non-canonical)
                ["/compiler/a.sm"], ["C:/compiler/a.sm"],     # absolute
                ["compiler/../x.sm"], ["compiler/./a.sm"],    # escape / dot component
                ["compiler\\a.sm"], ["compiler//a.sm"],        # non-POSIX / empty component
                ["other/a.sm"], ["compiler/a.txt"],           # out of root / not .sm
                ["compiler/Lexer.sm"],                         # non-canonical spelling
                ["compiler/b.sm", "compiler/a.sm"]):          # unsorted: rejected, never re-sorted
        assert check_paths(bad, "compiler"), bad

    # source-set identity
    base = [("compiler/a.sm", b"fn a() {}\n"), ("compiler/b.sm", b"fn b() {}\n")]
    ident = source_set_identity(base)
    assert ident.startswith("sha256:") and len(ident) == 71
    assert ident == source_set_identity(list(base))                                    # deterministic
    assert ident != source_set_identity([base[0], ("compiler/b.sm", b"fn b() { }\n")])  # content
    assert ident != source_set_identity([base[0], ("compiler/c.sm", base[1][1])])       # path
    assert ident != source_set_identity(base[::-1])                                     # order is identity
    assert source_set_identity([("ab", b"c")]) != source_set_identity([("a", b"bc")])   # framing
    assert source_set_identity([]) == source_set_identity([])
    for bad in (b"\xef\xbb\xbffn a() {}\n", b"fn a() {}\r\n", b"fn a() {}", b"fn\x00\n", b"\xff\n"):
        assert check_content("x.sm", bad), bad
    assert not check_content("x.sm", b"fn a() {}\n")
    assert check_content("x.sm", b"")  # empty file: no final newline

    # end-to-end on a throwaway tree, including on-disk bytes
    with tempfile.TemporaryDirectory() as tmp:
        repo = Path(tmp)
        (repo / "compiler").mkdir()
        (repo / "compiler/a.sm").write_bytes(b"fn a() {}\n")
        man = {"protocol": SOURCE_PROTOCOL, "root": "compiler", "files": ["compiler/a.sm"]}
        ident, errs = load_source_set(repo, man, "compiler")
        assert not errs and ident == source_set_identity([("compiler/a.sm", b"fn a() {}\n")])
        (repo / "compiler/a.sm").write_bytes(b"fn a() {}\r\n")                       # CRLF checkout
        assert load_source_set(repo, man, "compiler")[1]
        assert load_source_set(repo, {**man, "files": ["compiler/missing.sm"]}, "compiler")[1]
        assert load_source_set(repo, {**man, "protocol": "v0"}, "compiler")[1]

    # evidence binding
    record = {f: "x" for f in contract["evidence"]["required_fields"]}
    record.update(contract_protocol=PROTOCOL, comparison_rule="byte-equality-v1",
                  c0_identity=f"{REPOSITORY}@{contract['c0']['sha']}")
    assert not check_evidence(contract, record)
    assert check_evidence(contract, {**record, "comparison_rule": "normalized-equality-v1"})
    assert check_evidence(contract, {**record, "contract_protocol": "shf0-bootstrap-contract-v0"})
    assert check_evidence(contract, {**record, "c0_identity": f"{REPOSITORY}@main"})
    assert check_evidence(contract, {k: v for k, v in record.items() if k != "c2_artifact_hash"})
    print("shf0 self-test: PASS")


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        self_test()
        sys.exit(0)
    identity, problems = validate(ROOT)
    for p in problems:
        print(f"FAIL: {p}")
    if not problems:
        print(f"source-set identity: {identity}")
    print("shf0 contract: " + ("FAIL" if problems else "PASS"))
    sys.exit(1 if problems else 0)
