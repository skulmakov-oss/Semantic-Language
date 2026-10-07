#!/usr/bin/env python3
"""Fail-closed governance checks for the Semantic-Language SHF architecture.

Stdlib only (Python >= 3.11 for tomllib). Exit code 0 = pass, 1 = any violation.
Run with --self-test to prove the guards fire on known-bad input.
"""
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

REQUIRED_FILES = [
    "AGENTS.md", "CLAUDE.md", "README.md", "CONTRIBUTING.md",
    "docs/ARCHITECTURE.md", "docs/BOOTSTRAP_CONTRACT.md", "docs/BOOTSTRAP_SUBSET.md",
    "docs/HOST_BOUNDARY.md", "docs/OWNERSHIP.md", "docs/QUALIFICATION.md",
    "docs/ROADMAP.md", "docs/FUTURE.md", "docs/LEGACY_PLAN_MIGRATION.md",
    "reference/semantic-reference.toml",
]
# Files that replaced the old plan; they must not come back.
RETIRED_FILES = ["docs/BOOTSTRAP.md", "docs/HOST_CAPABILITY_ABI.md", ".agents/AGENTS.md"]

# The only active document allowed to use retired milestone vocabularies (as history).
LEGACY_EXEMPT = {"docs/LEGACY_PLAN_MIGRATION.md"}

LEGACY_MILESTONE = re.compile(r"\b(?:B[0-8](?:-\d+)?|SH-\d+|IP-?\d+|SRI-\d+|BS-\d{3}|HC-\d{3})\b")
OFF_CRITICAL_PATH = re.compile(r"instant pipeline|\bSRI\b|vm rewrite|verifier rewrite", re.I)
ROADMAP_OFF_PATH_HEADING = "## Not on the critical path"
LINK = re.compile(r"\[[^\]]*\]\(([^)\s]+)\)")
SHA = re.compile(r"^[0-9a-f]{40}$")
REFERENCE_STATUSES = {"planning-reference", "qualified-reference"}


def active_files(root):
    files = [root / "README.md", root / "CONTRIBUTING.md", root / "AGENTS.md"]
    files += sorted((root / "docs").glob("*.md"))
    files += sorted((root / ".github").rglob("*.md"))
    return files


def check_legacy_vocabulary(rel, text):
    if rel in LEGACY_EXEMPT:
        return []
    errors = []
    for n, line in enumerate(text.splitlines(), 1):
        for m in LEGACY_MILESTONE.finditer(line):
            errors.append(f"{rel}:{n}: retired milestone id '{m.group(0)}' in active document")
    return errors


def check_roadmap_critical_path(text):
    head = text.split(ROADMAP_OFF_PATH_HEADING, 1)
    if len(head) != 2:
        return [f"docs/ROADMAP.md: missing '{ROADMAP_OFF_PATH_HEADING}' section"]
    errors = []
    for n, line in enumerate(head[0].splitlines(), 1):
        if OFF_CRITICAL_PATH.search(line):
            errors.append(f"docs/ROADMAP.md:{n}: post-Bootstrap work on the SHF critical path: {line.strip()}")
    stages = set(re.findall(r"^## (SHF-\d+) ", head[0], re.M))
    expected = {f"SHF-{i}" for i in range(18)}
    if stages != expected:
        errors.append(f"docs/ROADMAP.md: SHF stage set mismatch: missing {sorted(expected - stages)}, extra {sorted(stages - expected)}")
    return errors


def check_reference(data):
    errors = []
    if data.get("repository") != "skulmakov-oss/Semantic":
        errors.append("reference: repository must be 'skulmakov-oss/Semantic'")
    if not SHA.match(str(data.get("sha", ""))):
        errors.append("reference: sha must be an exact 40-hex commit (never a branch name)")
    if data.get("status") not in REFERENCE_STATUSES:
        errors.append(f"reference: status must be one of {sorted(REFERENCE_STATUSES)}")
    if data.get("bootstrap", {}).get("upstream_issue") != 1910:
        errors.append("reference: [bootstrap].upstream_issue must be 1910")
    return errors


def check_links(rel, text, base):
    errors = []
    for target in LINK.findall(text):
        if re.match(r"^[a-z]+:", target) or target.startswith("#"):
            continue
        path = target.split("#", 1)[0]
        if path and not (base / path).exists():
            errors.append(f"{rel}: broken relative link '{target}'")
    return errors


def run(root):
    errors = []
    for f in REQUIRED_FILES:
        if not (root / f).is_file():
            errors.append(f"missing required file: {f}")
    for f in RETIRED_FILES:
        if (root / f).exists():
            errors.append(f"retired file present: {f}")
    for nested in root.rglob("AGENTS.md"):
        if nested != root / "AGENTS.md" and ".git" not in nested.parts:
            errors.append(f"competing nested AGENTS.md: {nested.relative_to(root).as_posix()}")
    claude = root / "CLAUDE.md"
    if claude.is_file() and claude.read_text(encoding="utf-8").strip() != "@AGENTS.md":
        errors.append("CLAUDE.md must contain only '@AGENTS.md'")
    for path in active_files(root):
        if not path.is_file():
            continue
        rel = path.relative_to(root).as_posix()
        text = path.read_text(encoding="utf-8")
        errors += check_legacy_vocabulary(rel, text)
        errors += check_links(rel, text, path.parent)
    roadmap = root / "docs/ROADMAP.md"
    if roadmap.is_file():
        errors += check_roadmap_critical_path(roadmap.read_text(encoding="utf-8"))
    ref = root / "reference/semantic-reference.toml"
    if ref.is_file():
        try:
            errors += check_reference(tomllib.loads(ref.read_text(encoding="utf-8")))
        except tomllib.TOMLDecodeError as e:
            errors.append(f"reference: invalid TOML: {e}")
    return errors


def self_test():
    """Each guard must fire on a known-bad input and stay quiet on a known-good one."""
    assert check_legacy_vocabulary("docs/X.md", "Phase B3 parser")
    assert check_legacy_vocabulary("docs/X.md", "see SH-00 manifest")
    assert check_legacy_vocabulary("docs/X.md", "IP-07 admission")
    assert not check_legacy_vocabulary("docs/X.md", "SHF-3 and SHF-10, sha b0f3, SHA-256")
    assert not check_legacy_vocabulary("docs/LEGACY_PLAN_MIGRATION.md", "B0-00")
    good = "".join(f"## SHF-{i} — x\n" for i in range(18)) + ROADMAP_OFF_PATH_HEADING + "\nInstant Pipeline, SRI\n"
    assert not check_roadmap_critical_path(good)
    assert check_roadmap_critical_path("## SHF-0 — x\nneeds the Instant Pipeline\n" + ROADMAP_OFF_PATH_HEADING)
    assert check_roadmap_critical_path(good.replace("## SHF-17 — x\n", ""))
    assert check_roadmap_critical_path("no heading")
    ok = {"repository": "skulmakov-oss/Semantic", "sha": "a" * 40, "status": "planning-reference",
          "bootstrap": {"upstream_issue": 1910}}
    assert not check_reference(ok)
    assert check_reference({**ok, "sha": "main"})
    assert check_reference({**ok, "repository": "someone/Semantic"})
    assert check_reference({**ok, "status": "canonical"})
    assert check_reference({**ok, "bootstrap": {}})
    assert check_links("x.md", "[a](definitely-missing-file.md)", ROOT)
    assert not check_links("x.md", "[a](https://example.com) [b](#anchor)", ROOT)
    print("self-test: PASS")


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        self_test()
        sys.exit(0)
    problems = run(ROOT)
    for p in problems:
        print(f"FAIL: {p}")
    print("governance: " + ("FAIL" if problems else "PASS"))
    sys.exit(1 if problems else 0)
