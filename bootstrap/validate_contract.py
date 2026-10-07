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
FROZEN_DIGESTS = {PROTOCOL: "211b8e1ef45170f18beda98b36c4a663b629cd59a64ceb30368995b464daf151"}
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
BSF_ID = re.compile(r"BSF-\d{3}")
SHF_RANGE = re.compile(r"SHF-(\d+)(?:\.\.(\d+))?")  # whole cell: SHF-n or SHF-a..b, 0..17
EMPTY_CELL = {"", "—", "-", "n/a", "tbd"}


def registry_rows(subset_text):
    """[(id, state, cells)] for every table row whose first cell is a BSF id."""
    rows = []
    for line in subset_text.splitlines():
        line = line.strip()  # indentation must not hide a row
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if cells and cells[0].upper().startswith("BSF"):  # a malformed id must not hide a row
            rows.append((cells[0], cells[2] if len(cells) > 2 else "", cells))
    return rows


def valid_stages(cell):
    """The whole cell is SHF-n or SHF-a..b with 0 <= a <= b <= 17."""
    m = SHF_RANGE.fullmatch(cell.strip())
    if not m:
        return False
    lo, hi = int(m.group(1)), int(m.group(2) or m.group(1))
    return 0 <= lo <= hi <= 17


def check_registry(subset_text):
    """Every row is complete for its state; ids are unique. Returns ({id: state}, errors).

    ADMITTED / FROZEN rows: ID | construction | state | required compiler use |
    positive / negative evidence | stages (requalification impact). Their reference authority is
    the section-level C0 authority stated above the table.
    CANDIDATE rows: ID | construction | state | gap at C0 | owner / SHF stage."""
    errors, registry = [], {}
    for bsf, state, cells in registry_rows(subset_text):
        if not BSF_ID.fullmatch(bsf):
            errors.append(f"CONTRACT_DRIFT: malformed registry id {bsf!r} (expected BSF-nnn)")
            continue
        if bsf in registry:
            errors.append(f"CONTRACT_DRIFT: registry id {bsf} appears more than once")
            continue
        registry[bsf] = state
        filled = [c for c in cells if c.lower() not in EMPTY_CELL]
        if state in ("ADMITTED", "FROZEN"):
            halves = [h.strip().lower() for h in cells[4].split(" / ")] if len(cells) > 4 else []
            if len(cells) != 6 or len(filled) != 6 or len(halves) != 2 \
                    or any(h in EMPTY_CELL for h in halves) \
                    or not valid_stages(cells[5]):
                errors.append(f"CONTRACT_DRIFT: {state} registry row {bsf} is incomplete "
                              "(needs compiler use, positive / negative evidence and SHF stages)")
        elif state == "CANDIDATE":
            owner, _, stage = cells[4].rpartition(" / ") if len(cells) == 5 else ("", "", "")
            if len(cells) != 5 or len(filled) != 5 or not owner.strip() or not valid_stages(stage):
                errors.append(f"CONTRACT_DRIFT: CANDIDATE registry row {bsf} needs a gap and an owner / SHF stage")
        else:
            errors.append(f"CONTRACT_DRIFT: registry row {bsf} has unknown state {state!r}")
    return registry, errors
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
    real_root = (repo / root).resolve()
    for p in files:
        f = repo / p
        # No symlink anywhere on the path, and the bytes must physically live under the root.
        chain = [repo.joinpath(*p.split("/")[:i]) for i in range(1, len(p.split("/")) + 1)]
        if any(c.is_symlink() for c in chain) or not f.is_file() \
                or real_root not in f.resolve().parents:
            errors.append(f"SOURCE_SET_INVALID: {p}: missing, symlinked or not a regular file under root")
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


def registry_section(subset_text):
    """The normative registry (BOOTSTRAP_SUBSET.md §5) with line endings and trailing spaces
    normalized: section authority, every row and every cell."""
    start = subset_text.find("## 5. Registry")
    end = subset_text.find("\n## ", start + 1)
    section = subset_text[start:end if end != -1 else None] if start != -1 else ""
    return "\n".join(line.rstrip() for line in section.splitlines()).strip()


def frozen_digest(contract, reference, subset_text):
    """sha256 of the canonical form of every frozen value: the whole contract (including the
    [subset] state lists), the whole C0 reference manifest (verdict, limits, drift, contract
    paths) and the normative registry section of BOOTSTRAP_SUBSET.md. Admitting, dropping or
    redefining a construction is a contract change (BOOTSTRAP_SUBSET.md §8)."""
    canonical = json.dumps({"contract": contract, "reference": reference,
                            "registry": registry_section(subset_text)},
                           sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    return hashlib.sha256(canonical.encode()).hexdigest()


def check_contract(contract, reference, subset_text):
    errors = []
    if contract.get("protocol") != PROTOCOL:
        errors.append(f"CONTRACT_DRIFT: protocol must be {PROTOCOL!r}")
    elif frozen_digest(contract, reference, subset_text) != FROZEN_DIGESTS[PROTOCOL]:
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
    registry, registry_errors = check_registry(subset_text)
    errors += registry_errors
    declared = {**{b: "ADMITTED" for b in admitted}, **{b: "CANDIDATE" for b in candidate},
                **{b: "FROZEN" for b in frozen}}
    if registry != declared:
        errors.append(f"CONTRACT_DRIFT: subset registry and contract disagree: "
                      f"registry={sorted(registry.items())} contract={sorted(declared.items())}")
    return errors


HASH = re.compile(r"sha256:[0-9a-f]{64}")


def check_evidence(contract, reference, record, source_set_identity):
    """A record supports the Bootstrap Seal only if it is bound to this exact contract AND
    records an admitted C1 and C2 whose comparison held, for the CURRENT S, on a qualified C0
    platform, with the pinned verifier/runtime and the Seal provenance of §7.
    `source_set_identity` is required and must equal the record's."""
    ev = contract["evidence"]
    errors = [f"CONTRACT_DRIFT: evidence field {f!r} missing"
              for f in ev["required_fields"] if f not in record]
    qualified = reference.get("qualification", {}).get("platform_scope")
    qualified = [qualified] if isinstance(qualified, str) else list(qualified or [])
    if "platform" in record and record["platform"] not in qualified:
        errors.append(f"REFERENCE_MISMATCH: platform {record['platform']!r} is outside the qualified "
                      f"C0 platforms {qualified}")
    for field in ("verifier_contract", "runtime_contract"):
        if field in record and record[field] != ev[field]:
            errors.append(f"REFERENCE_MISMATCH: {field} is not the pinned {ev[field]!r}")
    if "remaining_rust_responsibilities" in record:
        resp = record["remaining_rust_responsibilities"]
        if not isinstance(resp, list) or not resp \
                or any(r not in ev["rust_responsibility_categories"] for r in resp):
            errors.append("CONTRACT_DRIFT: remaining_rust_responsibilities must be a non-empty list of "
                          f"{ev['rust_responsibility_categories']}")
    for field in ("source_set_identity", "c1_artifact_hash", "c2_artifact_hash"):
        if field in record and not HASH.fullmatch(str(record[field])):
            errors.append(f"SOURCE_SET_INVALID: evidence {field} is not {ev['artifact_hash_format']}"
                          if field == "source_set_identity" else
                          f"FIXED_POINT_DELTA: evidence {field} is not {ev['artifact_hash_format']}")
    if not HASH.fullmatch(str(source_set_identity)) \
            or record.get("source_set_identity") != source_set_identity:
        errors.append("SOURCE_SET_INVALID: evidence was produced for a different source set")
    for field in ("c1_verifier_binding", "c2_verifier_binding"):
        if field in record and record[field] != ev["verifier_binding_admitted"]:
            errors.append(f"ADMISSION_REJECT: {field} is not {ev['verifier_binding_admitted']!r}")
    if "comparison_result" in record and record["comparison_result"] != ev["comparison_result_holds"]:
        errors.append("FIXED_POINT_DELTA: comparison_result does not record a holding fixed point")
    if contract["fixed_point"]["comparison_rule"] == "byte-equality-v1" \
            and record.get("c1_artifact_hash") != record.get("c2_artifact_hash"):
        errors.append("FIXED_POINT_DELTA: byte-equality-v1 requires identical C1 and C2 hashes")
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
    # registry rows must be complete for their state, and ids unique
    assert not check_registry(subset)[1]
    with_row = lambda row: subset + "\n" + row + "\n"  # noqa: E731
    assert check_registry(with_row("| BSF-999 | x | ADMITTED |"))[1]
    assert check_registry(with_row("| BSF-999 | x | ADMITTED | use | evidence | SHF-10 |"))[1]  # no "/"
    assert check_registry(with_row("| BSF-999 | x | ADMITTED | — | a / b | SHF-10 |"))[1]      # empty cell
    assert check_registry(with_row("| BSF-999 | x | ADMITTED | use | T A F I B / — | SHF-10 |"))[1]
    assert check_registry(with_row("| BSF-999 | x | ADMITTED | use | — / negatives | SHF-10 |"))[1]
    assert check_registry(with_row("| BSF-999 | x | FROZEN | use | a / b | stages |"))[1]       # no SHF
    assert check_registry(with_row("| BSF-999 | x | CANDIDATE | gap | upstream |"))[1]        # no stage
    assert check_registry(with_row("| BSF-999 | x | MAYBE | a | b |"))[1]
    assert check_registry(subset.replace("| BSF-001 |", "| BSF-01 |"))[1]        # malformed id
    indented = subset.replace("\n| BSF-001 |", "\n   | BSF-001 |", 1)
    assert check_registry(indented)[0].get("BSF-001") == "ADMITTED"              # still visible
    for stage in ("SHF-99", "junk SHF-10 junk", "SHF-14..10", "SHF-10..18", "SHF-"):
        assert check_registry(with_row(f"| BSF-999 | x | ADMITTED | use | a / b | {stage} |"))[1], stage
        assert check_registry(with_row(f"| BSF-999 | x | CANDIDATE | gap | owner / {stage} |"))[1], stage
    assert not check_registry(with_row("| BSF-999 | x | CANDIDATE | gap | `skulmakov-oss/Semantic` / SHF-17 |"))[1]
    assert check_registry(subset.replace("| BSF-001 |", "| bsf-001 |"))[1]
    assert not check_registry(with_row("| BSF-999 | x | ADMITTED | use | a / b | SHF-10..14 |"))[1]
    assert check_registry(with_row("| BSF-001 | other | CANDIDATE | gap | `skulmakov-oss/Semantic` / SHF-1 |"))[1]
    c999 = copy.deepcopy(contract)
    c999["subset"]["admitted"].append("BSF-999")
    assert check_contract(c999, reference, with_row("| BSF-999 | x | ADMITTED |"))
    fails("SOURCE_SET_INVALID", lambda c: c["source_set"].update(newline="normalize"))
    # every frozen value is covered by the protocol digest, not only the explicitly checked ones
    fails("CONTRACT_DRIFT", lambda c: c["source_set"].update(root="src"))
    fails("CONTRACT_DRIFT", lambda c: c["source_set"].update(manifest="other.toml"))
    fails("CONTRACT_DRIFT", lambda c: c["source_set"].update(final_newline="optional"))
    fails("CONTRACT_DRIFT", lambda c: c["source_set"]["identity"].update(domain="other"))
    fails("CONTRACT_DRIFT", lambda c: c["capabilities"]["artifact_write"].update(available_at_c0=True))
    for name in REQUIRED_FAILURES:  # no failure class may allow qualification to continue
        fails("CONTRACT_DRIFT", lambda c, n=name: c["failures"][n].update({"continue": True}))
    # the subset state lists are frozen with the protocol: they cannot be edited to hide a row
    fails("CONTRACT_DRIFT", lambda c: c["subset"]["candidate"].remove("BSF-106"))
    # the normative registry content (rows, cells, section authority) is frozen too
    for edit in [("Items and functions", "Anything at all"),             # construction
                 ("T A F I B / arity", "T / arity"),                       # evidence form
                 ("| SHF-12..14 |", "| SHF-13..14 |"),                     # stages
                 ("semantic.foundation.source/1.2", "semantic.foundation.source/9.9")]:  # authority
        errs = check_contract(contract, reference, subset.replace(*edit, 1))
        assert any("frozen value" in e for e in errs), edit
    # formatting-only noise (trailing spaces, CRLF) does not change the registry digest
    assert registry_section(subset) == registry_section(subset.replace("\n", "  \r\n"))
    fails("CONTRACT_DRIFT", lambda c: c["subset"].update(authority="docs/spec/other_profile.md"))
    fails("CONTRACT_DRIFT", lambda c: c["subset"].update(registry="docs/OTHER.md"))
    # the C0 reference manifest (verdict, limits, drift, contract paths) is frozen too
    fails("CONTRACT_DRIFT", lambda r: r["qualification"].update(verdict="NOT QUALIFIED"), ref=True)
    fails("CONTRACT_DRIFT", lambda r: r["qualification"]["limits"].pop(), ref=True)
    fails("CONTRACT_DRIFT", lambda r: r["drift"].update(inherits_qualification=True), ref=True)
    fails("CONTRACT_DRIFT", lambda r: r["contracts"].update(semcode_spec="docs/x.md"), ref=True)

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

    # a symlinked root or ancestor must not let bytes come from outside the declared root
    with tempfile.TemporaryDirectory() as tmp:
        repo, outside = Path(tmp) / "repo", Path(tmp) / "outside"
        repo.mkdir()
        outside.mkdir()
        (outside / "x.sm").write_bytes(b"fn x() {}\n")
        man = {"protocol": SOURCE_PROTOCOL, "root": "compiler", "files": ["compiler/x.sm"]}
        try:
            (repo / "compiler").symlink_to(outside, target_is_directory=True)
        except OSError:
            pass  # symlink creation needs privileges on some Windows hosts; CI runs on POSIX
        else:
            assert load_source_set(repo, man, "compiler")[1]
            (repo / "compiler").unlink()
            (repo / "compiler" / "sub").mkdir(parents=True)
            (repo / "compiler" / "sub" / "link").symlink_to(outside, target_is_directory=True)
            nested = {**man, "files": ["compiler/sub/link/x.sm"]}
            assert load_source_set(repo, nested, "compiler")[1]

    # evidence binding
    h = "sha256:" + "ab" * 32
    record = dict(contract_protocol=PROTOCOL, comparison_rule="byte-equality-v1",
                  c0_identity=f"{REPOSITORY}@{contract['c0']['sha']}",
                  source_set_identity="sha256:" + "cd" * 32, c1_artifact_hash=h, c2_artifact_hash=h,
                  c1_verifier_binding="admitted", c2_verifier_binding="admitted",
                  comparison_result="equal", platform="x86_64-pc-windows-msvc",
                  verifier_contract=contract["evidence"]["verifier_contract"],
                  runtime_contract=contract["evidence"]["runtime_contract"],
                  remaining_rust_responsibilities=["oracle", "verifier_runtime_foundation"])
    assert set(record) == set(contract["evidence"]["required_fields"])
    current = record["source_set_identity"]
    assert not check_evidence(contract, reference, record, current)
    assert check_evidence(contract, reference, record, None)                     # current S is mandatory
    # values, not only keys: a failed or malformed run is never Seal evidence
    for bad, cls in [({"comparison_result": "different"}, "FIXED_POINT_DELTA"),
                     ({"comparison_result": False}, "FIXED_POINT_DELTA"),
                     ({"c2_artifact_hash": "sha256:" + "ef" * 32}, "FIXED_POINT_DELTA"),
                     ({"c1_artifact_hash": "x", "c2_artifact_hash": "x"}, "FIXED_POINT_DELTA"),
                     ({"c1_verifier_binding": "rejected"}, "ADMISSION_REJECT"),
                     ({"c2_verifier_binding": "x"}, "ADMISSION_REJECT"),
                     ({"source_set_identity": "x"}, "SOURCE_SET_INVALID"),
                     ({"platform": "x86_64-unknown-linux-gnu"}, "REFERENCE_MISMATCH"),
                     ({"verifier_contract": "skulmakov-oss/Semantic@main:crates/sm-verify"}, "REFERENCE_MISMATCH"),
                     ({"runtime_contract": "local-build"}, "REFERENCE_MISMATCH"),
                     ({"remaining_rust_responsibilities": []}, "CONTRACT_DRIFT"),
                     ({"remaining_rust_responsibilities": ["parsing"]}, "CONTRACT_DRIFT")]:
        errs = check_evidence(contract, reference, {**record, **bad}, current)
        assert any(e.startswith(cls + ":") for e in errs), (bad, errs)
    assert check_evidence(contract, reference, record, "sha256:" + "00" * 32)  # stale evidence
    assert check_evidence(contract, reference, {**record, "comparison_rule": "normalized-equality-v1"}, current)
    assert check_evidence(contract, reference, {**record, "contract_protocol": "shf0-bootstrap-contract-v0"}, current)
    assert check_evidence(contract, reference, {**record, "c0_identity": f"{REPOSITORY}@main"}, current)
    assert check_evidence(contract, reference, {k: v for k, v in record.items() if k != "c2_artifact_hash"}, current)
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
