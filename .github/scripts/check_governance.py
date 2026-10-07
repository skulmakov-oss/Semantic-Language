#!/usr/bin/env python3
"""Fail-closed governance checks for the Semantic-Language SHF architecture.

Python >= 3.11 (tomllib). Markdown links are extracted with a real CommonMark parser
(markdown-it-py, pinned in requirements.txt) instead of regular expressions, so code spans,
fenced/indented code blocks and every link form follow the CommonMark specification.
Exit code 0 = pass, 1 = any violation. Run with --self-test to prove the guards fire.
"""
import re
import sys
import tempfile
import tomllib
from pathlib import Path
from urllib.parse import unquote, urlsplit

from markdown_it import MarkdownIt

COMMONMARK = MarkdownIt("commonmark")

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
# Every track that is explicitly NOT a prerequisite for the first fixed point (AGENTS.md section 3,
# docs/FUTURE.md). None may appear inside an SHF stage of docs/ROADMAP.md.
DEFERRED_TRACKS = [
    r"instant pipeline", r"\bSRI\b", r"vm rewrite", r"verifier rewrite", r"native backend",
    r"\bPROMETHEUS\b", r"\bUI\b", r"workbench", r"studio", r"#1909", r"full sigma", r"t¤",
    r"typescript", r"\bSEMIMG\b",
    # Instant Pipeline subtracks (docs/FUTURE.md)
    r"persistent compiler", r"incremental", r"query engine", r"dependency engine",
    r"admission reuse", r"prepared execution image", r"execution image", r"near-zero startup",
    r"compiler service", r"clean-build oracle",
    # rewrite relationship in either word order, including canonical component names
    r"\brewrit\w*\s+(?:the\s+)?(?:sm-vm|sm-verify|vm|verifier)\b",
    r"\b(?:sm-vm|sm-verify)\s+rewrit\w*",
]
OFF_CRITICAL_PATH = re.compile("|".join(DEFERRED_TRACKS), re.I)
# A stage's own Non-goals line legitimately names deferred work in order to exclude it.
NON_GOALS_FIELD = "**Non-goals:**"
ROADMAP_OFF_PATH_HEADING = "## Not on the critical path"
SHA = re.compile(r"^[0-9a-f]{40}$")
REFERENCE_STATUSES = {"planning-reference", "qualified-reference"}
# Load-bearing anchors of the root operating contract; removing any of them fails the gate.
AGENTS_ANCHORS = [
    "## 2. Authority hierarchy",
    "Indexes are retrieval tools, not sources of truth.",
    "`skulmakov-oss/Semantic` + exact Git SHA",
    "Floating upstream `main` is never a qualification oracle.",
    "Upstream authority: `skulmakov-oss/Semantic#1910`.",
    "`skulmakov-oss/Semantic#1909`",
    "First self-hosting does NOT require rewriting",
    "## 10. Forbidden",
    "Do not merge without owner GO.",
]


def active_files(root):
    files = [root / "README.md", root / "CONTRIBUTING.md", root / "AGENTS.md"]
    files += sorted((root / "docs").rglob("*.md"))
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


def non_goals_lines(text):
    """0-based source lines that belong to a `- **Non-goals:**` list item, as CommonMark
    structures it (wrapped text, blank lines and nested children included)."""
    tokens = COMMONMARK.parse(text)
    lines = set()
    for i, t in enumerate(tokens):
        if t.type == "list_item_open" and t.map:
            # Only the item's own first paragraph names the field, never a nested descendant.
            own = tokens[i + 1:i + 3]
            if [x.type for x in own] == ["paragraph_open", "inline"] \
                    and own[1].content.startswith(NON_GOALS_FIELD):
                lines.update(range(*t.map))
    return lines


def h2_headings(text):
    """(0-based start line, text) of every top-level h2 heading. '##' inside code, block quotes
    or list items is not a document section."""
    tokens = COMMONMARK.parse(text)
    return [(t.map[0], tokens[i + 1].content) for i, t in enumerate(tokens)
            if t.type == "heading_open" and t.tag == "h2" and t.level == 0 and t.map]


def rendered_text(inline):
    """Visible text of an inline token: emphasis/strong/link markup dropped, code spans kept as
    their text, soft/hard line breaks as spaces, whitespace collapsed."""
    parts = []
    for c in inline.children or []:
        if c.type in ("text", "code_inline"):
            parts.append(c.content)
        elif c.type in ("softbreak", "hardbreak"):
            parts.append(" ")
    return " ".join("".join(parts).split())


HISTORICAL_QUOTE = re.compile(r"^historical\b", re.I)


def active_prose(text):
    """(1-based line, rendered text) for each prose block that states active roadmap content.
    Excluded: code blocks (no inline tokens), a stage's own `- **Non-goals:**` field, and block
    quotes explicitly classified as historical (first text starts with "Historical"). Other
    block quotes, including GitHub callouts such as `> [!IMPORTANT]`, are active content."""
    excluded = non_goals_lines(text)
    tokens = COMMONMARK.parse(text)
    historical_depth = 0  # >0 while inside a block quote marked historical
    blocks = []
    for i, t in enumerate(tokens):
        if t.type == "blockquote_open":
            first = next((x for x in tokens[i + 1:] if x.type == "inline"), None)
            if historical_depth or (first and HISTORICAL_QUOTE.match(rendered_text(first))):
                historical_depth += 1
        elif t.type == "blockquote_close":
            historical_depth = max(0, historical_depth - 1)
        elif t.type == "inline" and t.map and not historical_depth and t.map[0] not in excluded:
            blocks.append((t.map[0] + 1, rendered_text(t)))
    return blocks


def check_roadmap_critical_path(text):
    headings = h2_headings(text)
    off_path = ROADMAP_OFF_PATH_HEADING.removeprefix("## ")
    cut = next((line for line, title in headings if title == off_path), None)
    if cut is None:
        return [f"docs/ROADMAP.md: missing '{ROADMAP_OFF_PATH_HEADING}' section"]
    errors = []
    for n, prose in active_prose("\n".join(text.splitlines()[:cut])):
        if OFF_CRITICAL_PATH.search(prose):
            errors.append(f"docs/ROADMAP.md:{n}: post-Bootstrap work on the SHF critical path: {prose}")
    stages = {m.group(1) for line, title in headings if line < cut
              for m in [re.match(r"(SHF-\d+) ", title)] if m}
    expected = {f"SHF-{i}" for i in range(18)}
    if stages != expected:
        errors.append(f"docs/ROADMAP.md: SHF stage set mismatch: missing {sorted(expected - stages)}, extra {sorted(stages - expected)}")
    return errors


def check_reference(data):
    errors = []
    if data.get("repository") != "skulmakov-oss/Semantic":
        errors.append("reference: repository must be 'skulmakov-oss/Semantic'")
    if not SHA.fullmatch(str(data.get("sha", ""))):
        errors.append("reference: sha must be an exact 40-hex commit (never a branch name)")
    if data.get("status") not in REFERENCE_STATUSES:
        errors.append(f"reference: status must be one of {sorted(REFERENCE_STATUSES)}")
    if data.get("bootstrap", {}).get("upstream_issue") != 1910:
        errors.append("reference: [bootstrap].upstream_issue must be 1910")
    return errors


def check_agents_contract(text):
    return [f"AGENTS.md: required contract anchor missing: {a!r}" for a in AGENTS_ANCHORS if a not in text]


def link_targets(text):
    """Every destination CommonMark sees, still URI-encoded: link hrefs, image srcs and all
    reference definitions (used or not). Code spans and code blocks contain none."""
    env = {}
    targets = []

    def walk(tokens):
        for t in tokens:
            if t.type == "link_open":
                targets.append(t.attrGet("href"))
            elif t.type == "image":
                targets.append(t.attrGet("src"))
            if t.children:
                walk(t.children)

    walk(COMMONMARK.parse(text, env))
    targets += [ref["href"] for ref in env.get("references", {}).values()]
    return [t for t in targets if t]


def check_links(rel, text, base, root=ROOT):
    errors = []
    root = root.resolve()
    for target in link_targets(text):
        # Classify on the encoded URI; decode only the path component for the file lookup.
        parts = urlsplit(target)
        drive = len(parts.scheme) == 1  # "C:/x" is a Windows absolute path, not a URI scheme
        if (parts.scheme and not drive) or parts.netloc:
            continue
        path = unquote(target.split("#", 1)[0] if drive else parts.path)
        if not path:
            continue
        # Only repository-relative assets count: absolute paths and escapes from the checkout
        # would otherwise be satisfied by arbitrary files on the CI host.
        resolved = (base / path).resolve()
        if drive or path.startswith(("/", "\\")) or Path(path).is_absolute() or \
                (resolved != root and root not in resolved.parents):
            errors.append(f"{rel}: link outside the repository '{target}'")
        elif not resolved.exists():
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
    agents = root / "AGENTS.md"
    if agents.is_file():
        errors += check_agents_contract(agents.read_text(encoding="utf-8"))
    claude = root / "CLAUDE.md"
    if claude.is_file() and claude.read_text(encoding="utf-8").strip() != "@AGENTS.md":
        errors.append("CLAUDE.md must contain only '@AGENTS.md'")
    for path in active_files(root):
        if not path.is_file():
            continue
        rel = path.relative_to(root).as_posix()
        text = path.read_text(encoding="utf-8")
        errors += check_legacy_vocabulary(rel, text)
        errors += check_links(rel, text, path.parent, root)
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
    # a fenced example of the marker is not the real heading; validation must continue past it
    fenced = good.replace("## SHF-17 — x\n", "## SHF-17 — x\n```md\n" + ROADMAP_OFF_PATH_HEADING
                          + "\n```\n- **Deliverable:** requires VM rewrite\n")
    assert check_roadmap_critical_path(fenced)
    assert check_roadmap_critical_path("```\n" + good + "```\n")  # stage headings in code don't count
    quoted = good.replace("## SHF-17 — x\n", "## SHF-17 — x\n> " + ROADMAP_OFF_PATH_HEADING
                          + "\n\n- **Deliverable:** requires VM rewrite\n")
    assert check_roadmap_critical_path(quoted)
    listed = good.replace("## SHF-17 — x\n", "## SHF-17 — x\n- " + ROADMAP_OFF_PATH_HEADING
                          + "\n\n- **Deliverable:** requires VM rewrite\n")
    assert check_roadmap_critical_path(listed)
    # Policy: a deferred track is detected by its rendered meaning, not its Markdown spelling.
    def stage(body):
        return good.replace("## SHF-5 — x\n", f"## SHF-5 — x\n{body}\n")
    for spelling in ["requires VM rewrite", "requires VM **rewrite**", "requires **VM** rewrite",
                     "requires VM\nrewrite", "requires VM *rewrite*", "requires VM `rewrite`",
                     "- **Deliverable:** requires the Instant\n  **Pipeline**"]:
        assert check_roadmap_critical_path(stage(spelling)), spelling
    # ...but the vocabulary itself is not banned where the architecture allows it.
    assert not check_roadmap_critical_path(stage("- **Non-goals:** VM **rewrite**"))
    assert not check_roadmap_critical_path(stage("> Historical plan: B7 required a VM **rewrite**."))
    assert not check_roadmap_critical_path(good.replace(
        ROADMAP_OFF_PATH_HEADING + "\n", ROADMAP_OFF_PATH_HEADING + "\nVM **rewrite** is deferred.\n"))
    assert not check_roadmap_critical_path(stage("```text\nVM rewrite example\n```"))
    # callouts and plain quotes inside a stage are active content
    assert check_roadmap_critical_path(stage("> [!IMPORTANT]\n> **Deliverable:** requires VM rewrite"))
    assert check_roadmap_critical_path(stage("> requires VM rewrite"))
    # canonical component names and rewrite-first wording
    for wording in ["rewrite sm-vm in Semantic", "rewrite sm-verify in Semantic",
                    "rewriting the verifier", "sm-vm rewrite", "Rewrite the VM"]:
        assert check_roadmap_critical_path(stage(f"- **Deliverable:** {wording}")), wording
    assert not check_roadmap_critical_path(stage("- **Deliverable:** artifact admitted by `sm-verify`"))
    for term in ["native backend", "PROMETHEUS", "UI", "Workbench", "Studio", "Semantic#1909",
                 "Full Sigma", "TypeScript", "SEMIMG", "verifier rewrite", "VM rewrite", "SRI",
                 "persistent compiler service", "incremental syntax", "incremental IR",
                 "query engine", "admission reuse", "prepared execution image", "near-zero startup"]:
        bad = good.replace("## SHF-5 — x\n", f"## SHF-5 — x\n- **Deliverable:** requires {term}\n")
        assert check_roadmap_critical_path(bad), term
        excluded = good.replace("## SHF-5 — x\n", f"## SHF-5 — x\n- **Non-goals:** {term}\n")
        assert not check_roadmap_critical_path(excluded), term
        wrapped = good.replace("## SHF-5 — x\n", f"## SHF-5 — x\n- **Non-goals:** other work;\n  {term}\n")
        assert not check_roadmap_critical_path(wrapped), term
        child = good.replace("## SHF-5 — x\n", f"## SHF-5 — x\n- **Non-goals:**\n  - {term}\n")
        assert not check_roadmap_critical_path(child), term
        loose = good.replace("## SHF-5 — x\n", f"## SHF-5 — x\n- **Non-goals:**\n\n  - {term}\n\n  - other\n")
        assert not check_roadmap_critical_path(loose), term
        nested = good.replace("## SHF-5 — x\n", f"## SHF-5 — x\n-\n  - **Non-goals:** x\n  - **Deliverable:** requires {term}\n")
        assert check_roadmap_critical_path(nested), term
        peer = good.replace("## SHF-5 — x\n", f"## SHF-5 — x\n- **Non-goals:**\n\n  - x\n\n{term} is required\n")
        assert check_roadmap_critical_path(peer), term
        after = good.replace("## SHF-5 — x\n", f"## SHF-5 — x\n- **Non-goals:** x\n- **Deliverable:** {term}\n")
        assert check_roadmap_critical_path(after), term
    ok = {"repository": "skulmakov-oss/Semantic", "sha": "a" * 40, "status": "planning-reference",
          "bootstrap": {"upstream_issue": 1910}}
    assert not check_reference(ok)
    assert check_reference({**ok, "sha": "main"})
    assert check_reference({**ok, "sha": "a" * 40 + "\n"})
    assert check_reference({**ok, "repository": "someone/Semantic"})
    assert check_reference({**ok, "status": "canonical"})
    assert check_reference({**ok, "bootstrap": {}})
    assert check_links("x.md", "[a](definitely-missing-file.md)", ROOT)
    assert check_agents_contract("")
    assert check_agents_contract("unrelated text")
    assert not check_agents_contract("\n".join(AGENTS_ANCHORS))
    assert len(check_agents_contract("\n".join(AGENTS_ANCHORS[1:]))) == 1
    # nested docs are governed: build a throwaway tree with a nested bad doc
    with tempfile.TemporaryDirectory() as tmp:
        nested = Path(tmp) / "docs" / "design"
        nested.mkdir(parents=True)
        (nested / "new.md").write_text("Phase B3 [x](missing.md)\n", encoding="utf-8")
        errs = run(Path(tmp))
        assert any("docs/design/new.md" in e and "B3" in e for e in errs)
        assert any("docs/design/new.md" in e and "broken relative link" in e for e in errs)
    assert not check_links("x.md", "[a](https://example.com) [b](#anchor)", ROOT)
    assert check_links("x.md", '[a](missing.md "details")', ROOT)
    assert check_links("x.md", "[a](<missing file.md>)", ROOT)
    assert check_links("x.md", "[a](<missing.md> 'details')", ROOT)
    assert check_links("x.md", "[contract][c]\n\n[c]: missing.md\n", ROOT)
    assert check_links("x.md", '[c]: <missing file.md> "t"\n', ROOT)
    assert not check_links("x.md", "[c]: https://example.com\n[d]: #anchor\n", ROOT)
    assert not check_links("x.md", "text[^1]\n\n[^1]: Explanatory text\n", ROOT)
    assert not check_links("x.md", "use `[x](missing.md)` syntax\n", ROOT)
    assert not check_links("x.md", "```md\n[x](missing.md)\n[c]: missing.md\n```\n", ROOT)
    assert check_links("x.md", "```md\nok\n```\n[x](missing.md)\n", ROOT)
    assert not check_links("x.md", "```\n[x](missing.md)\n````\n", ROOT)
    assert not check_links("x.md", "~~~\n[x](missing.md)\n```\nstill code\n~~~\n", ROOT)
    assert check_links("x.md", "````\nok\n```\n[y](inside.md)\n````\n[x](missing.md)\n", ROOT)
    # a backtick info string containing a backtick is not a fence opener: the link stays live
    assert check_links("x.md", "```md`example`\n[x](missing.md)\n", ROOT)
    # indented code blocks are literal text
    assert not check_links("x.md", "para\n\n    [x](missing.md)\n", ROOT)
    assert check_links("x.md", "[x](missing%20file.md)\n", ROOT)
    assert check_links("x.md", "![diagram](missing.png)\n", ROOT)
    assert check_links("x.md", "[draft](%23missing.md)\n", ROOT)
    here = Path(__file__).resolve()
    assert check_links("x.md", f"[abs]({here.as_posix()})\n", ROOT)          # absolute, though it exists
    assert check_links("x.md", "[host](/etc/passwd)\n", ROOT)
    with tempfile.TemporaryDirectory() as tmp:
        if sys.platform != "win32":  # on POSIX a checkout can really contain a "C:" directory
            (Path(tmp) / "C:").mkdir()
            (Path(tmp) / "C:" / "x.md").write_text("x", encoding="utf-8")
        assert check_links("x.md", "[drive](C:/x.md)\n", Path(tmp), Path(tmp))
    assert check_links("x.md", "[up](../../../../../../etc/hosts)\n", ROOT)  # escapes the checkout
    assert not check_links("x.md", "[ok](.github/scripts/check_governance.py)\n", ROOT)
    assert not check_links("x.md", "[a](check_governance.py#L1) [b](mailto:x@y.z)\n", Path(__file__).parent)
    assert not check_links("x.md", '[a](check_governance.py "self")', Path(__file__).parent)
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
