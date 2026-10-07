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
# markdown-it drops links whose URL it deems unsafe (file:, javascript:, ...) and renders them as
# plain text. The governance check must SEE every destination to judge it, so accept them all.
COMMONMARK.validateLink = lambda url: True

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
    # FUTURE.md "Verifier or VM in Semantic": implementing either component in Semantic
    r"\b(?:semantic|self-hosted)\s+(?:vm|verifier)\b",
    r"\b(?:vm|verifier|sm-vm|sm-verify)\s+(?:written\s+)?in\s+semantic\b",
    r"\b(?:implement|write|build|create|port|reimplement)\w*\s+(?:the\s+|a\s+)?(?:sm-vm|sm-verify)\b",
]
OFF_CRITICAL_PATH = re.compile("|".join(DEFERRED_TRACKS), re.I)
# A stage's own Non-goals line legitimately names deferred work in order to exclude it.
NON_GOALS_FIELD = "**Non-goals:**"
ROADMAP_OFF_PATH_HEADING = "## Not on the critical path"
SHA = re.compile(r"^[0-9a-f]{40}$")
REFERENCE_STATUSES = {"planning-reference", "qualified-reference"}
# Load-bearing anchors of the root operating contract; removing any of them fails the gate.
# They are matched against the RENDERED document (CommonMark tokens), never raw bytes, so an
# anchor kept only inside code, an HTML comment or raw HTML does not count.
# Each required sentence must be operative prose inside its own top-level `##` section, so
# moving a rule into any other section (e.g. "## Historical contract") removes it from force.
AGENTS_CONTRACT = {  # section heading -> visible sentences required in that section
    "1. Repositories and paths": [
        "Any qualification oracle must be identified by: skulmakov-oss/Semantic + exact Git SHA.",
    ],
    "2. Authority hierarchy": [
        "Indexes are retrieval tools, not sources of truth.",
        "Floating upstream main is never a qualification oracle.",
    ],
    "3. Self-hosting scope": [
        "Upstream authority: skulmakov-oss/Semantic#1910.",
        "skulmakov-oss/Semantic#1909 (Native Reasoning / Full Sigma + t¤) is a post-self-hosting track.",
        "First self-hosting does NOT require rewriting",
    ],
    "10. Forbidden": [
        "Do not merge without owner GO.",
    ],
}
AGENTS_HEADINGS = list(AGENTS_CONTRACT)
AGENTS_PROSE = [p for sentences in AGENTS_CONTRACT.values() for p in sentences]


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


QUOTE_PREFIX = re.compile(r"^ {0,3}((?:> ?)+)")


def quote_markers(line):
    m = QUOTE_PREFIX.match(line)
    return m.group(1).count(">") if m else 0


def operative_inlines(tokens, lines):
    """(index, inline token) for every inline token that is in force. One rule for all guards:
    code blocks and HTML yield no inline tokens; a block quote explicitly classified as
    historical is cited material. Every other block quote, including GitHub callouts such as
    `> [!IMPORTANT]`, is operative content.

    A quote is historical only if its OWN first paragraph (paragraph_open + inline directly
    after blockquote_open) starts with "Historical" and every source line of that paragraph
    carries the quote's full `>` prefix. A nested descendant never classifies an ancestor, and a
    lazy-continuation line (fewer `>` markers, which CommonMark folds into the inner paragraph)
    cannot be hidden inside a historical quote."""
    stack = []  # one bool per open block quote: is it historical?
    for i, t in enumerate(tokens):
        if t.type == "blockquote_open":
            own = tokens[i + 1:i + 3]
            depth = len(stack) + 1
            marked = (
                [x.type for x in own] == ["paragraph_open", "inline"]
                and HISTORICAL_QUOTE.match(rendered_text(own[1])) is not None
                and own[1].map is not None
                and all(quote_markers(lines[n]) >= depth for n in range(*own[1].map) if n < len(lines))
            )
            stack.append(marked)
        elif t.type == "blockquote_close":
            if stack:
                stack.pop()
        elif t.type == "inline" and not any(stack):
            yield i, t


def active_prose(text):
    """(1-based line, rendered text) for each operative roadmap block, excluding a stage's own
    `- **Non-goals:**` field."""
    excluded = non_goals_lines(text)
    return [(t.map[0] + 1, rendered_text(t)) for _, t in operative_inlines(COMMONMARK.parse(text), text.splitlines())
            if t.map and t.map[0] not in excluded]


def check_roadmap_critical_path(text):
    headings = h2_headings(text)
    off_path = ROADMAP_OFF_PATH_HEADING.removeprefix("## ")
    cut = next((line for line, title in headings if title == off_path), None)
    if cut is None:
        return [f"docs/ROADMAP.md: missing '{ROADMAP_OFF_PATH_HEADING}' section"]
    errors = []
    critical = "\n".join(text.splitlines()[:cut])
    # Fail closed: the SHF roadmap needs no raw HTML, and HTML content is not interpreted.
    for t in COMMONMARK.parse(critical):
        if t.type == "html_block" and t.map:
            errors.append(f"docs/ROADMAP.md:{t.map[0] + 1}: raw HTML block in the SHF critical-path "
                          "region is not allowed")
    for n, prose in active_prose(critical):
        if OFF_CRITICAL_PATH.search(prose):
            errors.append(f"docs/ROADMAP.md:{n}: post-Bootstrap work on the SHF critical path: {prose}")
    # Exactly one definition per stage, in canonical order: no duplicates, gaps or reordering.
    stages = [m.group(1) for line, title in headings if line < cut
              for m in [re.match(r"(SHF-\d+)\b", title)] if m]
    expected = [f"SHF-{i}" for i in range(18)]
    if stages != expected:
        errors.append(f"docs/ROADMAP.md: SHF stage headings must be exactly {expected[0]}..{expected[-1]} "
                      f"once each in order; found {stages}")
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


def rendered_blocks(text):
    """(kind, rendered text, section) of every operative block: kind is 'heading' (top-level
    only) or 'prose'; section is the enclosing top-level `##` heading. Uses the same operative
    rule as the roadmap guard (see operative_inlines): code, HTML and historical quotes cannot
    satisfy the contract; callouts and other quotes can."""
    tokens = COMMONMARK.parse(text)
    blocks = []
    section = None
    for i, t in operative_inlines(tokens, text.splitlines()):
        opener = tokens[i - 1] if i else None
        if opener is not None and opener.type == "heading_open":
            if opener.level != 0:
                continue  # only top-level document headings define contract sections
            kind = "heading"
            if opener.tag == "h2":
                section = rendered_text(t)
        else:
            kind = "prose"
        # A block made only of inline code is a code literal, not a stated rule.
        if any(c.type == "text" and c.content.strip() for c in t.children or []):
            blocks.append((kind, rendered_text(t), section))
    return blocks


def check_agents_contract(text):
    blocks = rendered_blocks(text)
    headings = {b for kind, b, _ in blocks if kind == "heading"}
    errors = [f"AGENTS.md: required contract heading missing: {h!r}"
              for h in AGENTS_HEADINGS if h not in headings]
    for section, sentences in AGENTS_CONTRACT.items():
        prose = [b for kind, b, sec in blocks if kind == "prose" and sec == section]
        errors += [f"AGENTS.md: required contract sentence missing from section {section!r}: {p!r}"
                   for p in sentences if not any(p in b for b in prose)]
    return errors


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
        if parts.scheme.lower() == "file":  # host filesystem, never a repository asset
            errors.append(f"{rel}: link outside the repository '{target}'")
            continue
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
    # stage definitions: exactly once each, in canonical order
    assert check_roadmap_critical_path(good.replace("## SHF-5 — x\n", "## SHF-5 — x\n## SHF-5 — again\n"))
    assert check_roadmap_critical_path(good.replace("## SHF-3 — x\n## SHF-4 — x\n", "## SHF-4 — x\n## SHF-3 — x\n"))
    assert check_roadmap_critical_path(good.replace("## SHF-17 — x\n", "## SHF-17 — x\n## SHF-18 — x\n"))
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
    # historical classification is structural: a nested historical quote never exempts its parent
    assert check_roadmap_critical_path(stage("> > Historical note\n> **Deliverable:** requires VM rewrite"))
    assert check_roadmap_critical_path(stage("> > Historical note\n> requires Instant Pipeline"))
    assert not check_roadmap_critical_path(stage("> Historical note\n>\n> B7 previously required VM rewrite."))
    assert check_roadmap_critical_path(stage("> **Deliverable:** requires VM rewrite"))
    assert check_roadmap_critical_path(stage("> > Historical note\n>\n> **Deliverable:** requires VM rewrite"))
    assert check_roadmap_critical_path(stage("> [!IMPORTANT]\n> **Deliverable:** requires VM rewrite"))
    assert not check_roadmap_critical_path(stage("> > Historical note\n> > B7 required a VM rewrite."))
    # raw HTML blocks inside the SHF critical-path region fail closed
    assert check_roadmap_critical_path(stage("<div>\nDeliverable: requires VM rewrite\n</div>"))
    assert check_roadmap_critical_path(stage("<div>\n\n**Deliverable:** requires VM rewrite\n\n</div>"))
    assert check_roadmap_critical_path(stage(
        "<details>\n<summary>Requirement</summary>\nrequires Instant Pipeline\n</details>"))
    assert check_roadmap_critical_path(stage("<!-- requires VM rewrite -->"))
    assert not check_roadmap_critical_path(good.replace(  # HTML after the real off-path heading is fine
        ROADMAP_OFF_PATH_HEADING + "\n", ROADMAP_OFF_PATH_HEADING + "\n<div>\nVM rewrite\n</div>\n"))
    # canonical component names and rewrite-first wording
    for wording in ["rewrite sm-vm in Semantic", "rewrite sm-verify in Semantic",
                    "rewriting the verifier", "sm-vm rewrite", "Rewrite the VM",
                    "implement sm-vm in Semantic", "create a Semantic VM", "port sm-verify",
                    "a verifier written in Semantic", "self-hosted VM"]:
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
    # AGENTS.md contract: only the rendered, active document counts.
    contract = "".join(f"## {h}\n\n" + "".join(f"{p}\n\n" for p in ps) for h, ps in AGENTS_CONTRACT.items())
    real_agents = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
    assert not check_agents_contract(contract)
    assert not check_agents_contract(real_agents)
    assert check_agents_contract("")
    assert check_agents_contract("unrelated text")
    assert check_agents_contract("```markdown\n" + contract + "```\n")            # fenced
    assert check_agents_contract("~~~\n" + contract + "~~~\n")
    assert check_agents_contract("<!--\n" + contract + "-->\n")                   # HTML comment
    assert check_agents_contract("<div>\n" + contract.replace("\n\n", "\n") + "</div>\n")  # raw HTML
    assert check_agents_contract("\n".join("    " + l for l in contract.splitlines()) + "\n")  # indented
    assert check_agents_contract(contract.replace("## 10. Forbidden", "`## 10. Forbidden`"))
    assert check_agents_contract(contract.replace("## 2. Authority hierarchy", "2. Authority hierarchy"))
    assert check_agents_contract(contract.replace("Do not merge without owner GO.",
                                                  "`Do not merge without owner GO.`"))  # code-only sentence
    assert len(check_agents_contract(contract.replace(AGENTS_PROSE[0], ""))) == 1
    quoted = "> Historical contract:\n>\n" + "".join(f"> {l}\n" for l in contract.splitlines())
    assert check_agents_contract(quoted)                                          # quoted history
    # rules must stay in their own section: empty headings + rules under "## Historical contract"
    hollow = ("".join(f"## {h}\n\n" for h in AGENTS_HEADINGS) + "## Historical contract\n\n"
              + "".join(f"{p}\n\n" for p in AGENTS_PROSE))
    assert len(check_agents_contract(hollow)) == len(AGENTS_PROSE)
    assert check_agents_contract(real_agents.replace(  # one rule moved to another section
        "- Do not merge without owner GO.", "").replace(
        "## 9. Working loop", "## 9. Working loop\n\nDo not merge without owner GO.\n"))
    assert check_agents_contract("- item\n\n" + "".join(f"  {l}\n" for l in contract.splitlines()))
    assert len(check_agents_contract(  # one rule moved into a historical quote is no longer in force
        contract.replace(AGENTS_PROSE[-1], "> Historical rule: " + AGENTS_PROSE[-1]))) == 1
    # a rule kept in an operative callout or plain quote remains in force
    assert not check_agents_contract(contract.replace(
        AGENTS_PROSE[-1], "> [!IMPORTANT]\n> " + AGENTS_PROSE[-1]))
    assert not check_agents_contract(contract.replace(AGENTS_PROSE[-1], "> " + AGENTS_PROSE[-1]))
    # visible text unchanged by emphasis -> still satisfied
    assert not check_agents_contract(real_agents.replace("Do not merge without owner GO.",
                                                         "Do **not** merge without *owner* GO."))
    assert not check_agents_contract(real_agents.replace("Floating upstream `main` is never",
                                                         "Floating upstream **`main`** is never"))
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
    assert check_links("x.md", "[host](file:///etc/passwd)\n", ROOT)
    assert check_links("x.md", "[host](FILE://server/share/x.md)\n", ROOT)
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
