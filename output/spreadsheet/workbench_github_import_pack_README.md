# Workbench GitHub Import Pack

This pack converts the Workbench roadmap into one GitHub-friendly CSV with one
row per planned PR.

Files:

- `output/spreadsheet/workbench_github_import_pack.csv`
- `output/spreadsheet/workbench_label_presets.csv`
- `output/spreadsheet/workbench_milestone_presets.csv`

Column meanings:

- `Epic`: grouping key for the larger workstream
- `PR`: canonical PR title for the slice
- `Branch`: recommended working branch name
- `Depends on`: upstream PRs that should merge first
- `Acceptance Criteria`: one-line definition of done for the slice
- `Labels`: semicolon-separated label suggestions
- `Milestone`: recommended delivery wave

Recommended milestone mapping:

- `WB-0 Bootstrap`
- `WB-0.1 Cockpit`
- `WB-0.2 Authoring`
- `WB-0.3 Inspect`
- `WB-0.4 Operate`
- `WB-0.5 Protocol`

Recommended import/use pattern:

1. Create milestone names from the `Milestone` column.
2. Create labels from the `Labels` field by splitting on `;`.
3. Create one issue or draft PR seed per row.
4. Use `Depends on` to wire merge order and project sequencing.

Preset helpers:

- `workbench_label_presets.csv` contains suggested GitHub label names, colors,
  and descriptions for the backlog.
- `workbench_milestone_presets.csv` contains the recommended milestone waves
  and ordering for the Workbench roadmap.

Scope rule:

- The pack assumes Workbench remains a UI/orchestration layer over public
  Semantic surfaces.
- No row assumes ownership of parser, verifier, VM, PROMETHEUS ABI, or runtime
  semantics.
