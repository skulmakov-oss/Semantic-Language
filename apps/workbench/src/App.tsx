import { startTransition, useEffect, useRef, useState, type ReactNode } from 'react'
import { NavLink, Route, Routes, useNavigate } from 'react-router-dom'
import {
  exportReleaseReportFile,
  fetchAdapterContract,
  fetchOverviewSnapshot,
  fetchSpecCatalog,
  fetchSpecDocument,
  fetchWorkspaceFile,
  fetchWorkspaceTree,
  resolveWorkspaceRoot,
  runCliJob,
  runSmlspProtocolBridge,
  saveWorkspaceFile,
  scaffoldSemanticProject,
  type AdapterContract,
  type AssetSmokeSnapshot,
  type AdapterJobSpec,
  type JobKind,
  type JobResult,
  type OverviewDocument,
  type OverviewSnapshot,
  type ReleaseBundleManifest,
  type SmlspBridgeResult,
  type SpecCatalogDocument,
  type ScaffoldProjectResult,
  type SpecCatalogSection,
  type SpecDocumentHeading,
  type SpecDocumentView,
  type WorkspaceFileDocument,
  type WorkspaceTreeNode,
  type WorkspaceSummary,
} from './workbench-api'
import {
  loadWorkbenchState,
  mergeRecentWorkspace,
  saveWorkbenchState,
  type RecentWorkspace,
  type WorkbenchSettings,
} from './workbench-state'
import {
  deriveDiagnosticsFromJobs,
  diagnosticDocMapping,
  diagnosticDocLinks,
  diagnosticFamilyLabel,
  diagnosticFamilyOrder,
  type DiagnosticFamily,
  type WorkbenchDiagnostic,
} from './diagnostics'
import './App.css'

type ScreenSpec = {
  path: string
  label: string
  eyebrow: string
  title: string
  summary: string
  stable: string[]
  next: string[]
}

type JobRecord = {
  id: string
  kind: JobKind
  label: string
  status: 'running' | 'success' | 'failed'
  commandLine: string
  cwd: string
  resolvedCommand: string[]
  durationMs?: number
  exitCode?: number
  stdout: string
  stderr: string
}

type JobActionSpec = {
  kind: JobKind
  label: string
  args: string[]
  notes: string
  cwdMode: 'repo' | 'workspace'
}

type InspectFamily = 'trace' | 'verify' | 'disasm' | 'verified-run'

type InspectableJob = {
  job: JobRecord
  family: InspectFamily
  artifactPath: string | null
  artifactSource: 'explicit-command-arg' | 'not-explicit'
  summary: string
  summarySource: 'stdout-first-line' | 'stderr-first-line' | 'job-status'
  stdoutText: string | null
  stderrText: string | null
}

type EditorTab = {
  relativePath: string
  absolutePath: string
  title: string
  content: string
  savedContent: string
  status: 'clean' | 'dirty' | 'saving'
}

type PackageManifestPreview = {
  name: string | null
  version: string | null
  edition: string | null
  entry: string | null
}

type SmlspSessionResult = {
  relativePath: string
  command: string
  result: SmlspBridgeResult
}

type EditorCursorPosition = {
  line: number
  character: number
}

type WorkspaceOpenSource = 'default' | 'manual' | 'recent' | 'preset' | 'fallback'

type WorkspaceOpenOptions = {
  persist?: boolean
  source?: WorkspaceOpenSource
  successMessage?: string | null
}

type EditorOpenOptions = {
  line?: number | null
  column?: number | null
  source?: 'diagnostic' | 'definition' | 'scaffold' | 'workspace'
}

type SaveEditorOptions = {
  applyFormatOnSave?: boolean
}

type EditorFocusTarget = {
  relativePath: string
  line: number | null
  column: number | null
  source: NonNullable<EditorOpenOptions['source']>
}

const initialWorkbenchState = loadWorkbenchState()

const workflowActions: JobActionSpec[] = [
  {
    kind: 'cargo',
    label: 'Run workspace tests',
    args: ['test', '--workspace'],
    notes: 'Run the full repository validation suite from the repository root.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Compile semantic stress example',
    args: [
      'compile',
      'examples/semantic_policy_overdrive_trace.sm',
      '-o',
      'target/semantic_policy_overdrive_trace.smc',
    ],
    notes: 'Compile the canonical Workbench stress example into SemCode.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Run semantic stress source',
    args: ['run', 'examples/semantic_policy_overdrive_trace.sm'],
    notes: 'Run the source example through the public smc surface.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Trace semantic stress cache path',
    args: ['check', 'examples/semantic_policy_overdrive_trace.sm', '--trace-cache'],
    notes: 'Inspect the canonical cache-trace surface through smc check --trace-cache.',
    cwdMode: 'repo',
  },
  {
    kind: 'svm',
    label: 'Run compiled semantic stress bytecode',
    args: ['run', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Execute the compiled SemCode artifact through svm.',
    cwdMode: 'repo',
  },
  {
    kind: 'svm',
    label: 'Disassemble semantic stress bytecode',
    args: ['disasm', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Inspect the compiled SemCode artifact with the public disasm surface.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Verify compiled semantic stress bytecode',
    args: ['verify', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Verify the compiled SemCode artifact through the canonical smc verify surface.',
    cwdMode: 'repo',
  },
  {
    kind: 'release_bundle_verify',
    label: 'Verify release bundle',
    args: [],
    notes: 'Run the canonical release bundle verification script on the baseline manifest.',
    cwdMode: 'repo',
  },
]

const releaseValidationPlan: JobActionSpec[] = [
  {
    kind: 'cargo',
    label: 'Clean validation: workspace tests',
    args: ['test', '--workspace'],
    notes: 'Run the full repository validation suite from the repository root.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Clean validation: compile canonical stress example',
    args: [
      'compile',
      'examples/semantic_policy_overdrive_trace.sm',
      '-o',
      'target/semantic_policy_overdrive_trace.smc',
    ],
    notes: 'Compile the canonical Semantic stress example to a deterministic SemCode artifact.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Clean validation: verify canonical SemCode artifact',
    args: ['verify', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Run the canonical verifier over the compiled SemCode artifact.',
    cwdMode: 'repo',
  },
  {
    kind: 'svm',
    label: 'Clean validation: disasm canonical SemCode artifact',
    args: ['disasm', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Inspect the compiled SemCode artifact through the canonical disassembly surface.',
    cwdMode: 'repo',
  },
  {
    kind: 'svm',
    label: 'Clean validation: run canonical SemCode artifact',
    args: ['run', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Execute the compiled SemCode artifact through the verified svm surface.',
    cwdMode: 'repo',
  },
  {
    kind: 'release_bundle_verify',
    label: 'Clean validation: verify release bundle',
    args: [],
    notes: 'Run the canonical release bundle verification script on the baseline manifest.',
    cwdMode: 'repo',
  },
]

const routeSpecs: ScreenSpec[] = [
  {
    path: '/',
    label: 'Overview',
    eyebrow: 'WB-0.1 Cockpit',
    title: 'Repository reality without terminal drift.',
    summary:
      'Overview is the command-and-readiness cockpit. It exists to surface branch, baseline tag, recent validation signals, and known limits from real repository sources.',
    stable: [
      'Branch, commit, and baseline tag cards',
      'Source-of-truth callouts for specs, roadmap, and release artifacts',
      'Deterministic command adapter contract for smc, svm, cargo, and release verification',
      'Real workflow actions and job history routed through the backend process adapter',
    ],
    next: [
      'Split jobs and diagnostics into richer structured views',
      'Add spec navigation without mutating canonical docs',
    ],
  },
  {
    path: '/project',
    label: 'Project',
    eyebrow: 'WB-0.2 Authoring',
    title: 'Project explorer and editor shell without hidden semantics.',
    summary:
      'Project owns workspace selection, file-tree navigation, tabs, and text editing. It does not create an alternate package, parser, or repository model.',
    stable: [
      'Workspace resolver over canonical repository paths',
      'Recent projects list and default workspace persistence',
      'Canonical project bootstrap for Semantic.toml, src/main.sm, and examples/smoke.sm',
      'Read-only package metadata preview derived from Semantic.toml',
      'Workspace file tree, read/write text editor tabs, current-file compile/check, and formatter actions through smc fmt',
    ],
    next: [
      'Route current-file command results into richer diagnostics views',
      'Keep editor shell intentionally lighter than a full IDE until later slices arrive',
    ],
  },
  {
    path: '/spec',
    label: 'Spec',
    eyebrow: 'WB-0.1 Cockpit',
    title: 'Read-only entry into the contract bundle.',
    summary:
      'Spec navigation is a presentation layer over docs/spec and docs/roadmap. Workbench points at the documents; it does not fork them.',
    stable: [
      'Docs entry cards for language, execution, and release anchors',
      'Canonical tree over docs/spec, docs/roadmap, and synced language overview documents',
      'Title/path search and section navigator driven by repository markdown',
      'Source-path discipline called out directly in the UI',
    ],
    next: [
      'Add freshness hints and stronger release-document callouts',
      'Keep navigator read-only even when authoring shell arrives',
    ],
  },
  {
    path: '/diagnostics',
    label: 'Diagnostics',
    eyebrow: 'WB-0.2 Authoring',
    title: 'One structured panel, not stdout archaeology.',
    summary:
      'Diagnostics will group parse, type, module, verify, and runtime outputs. The shell here reserves the contract without duplicating parser or verifier semantics.',
    stable: [
      'Dedicated diagnostics route and panel shell',
      'Family buckets for parse, type, module, verify, and runtime',
      'Space reserved for spec-linked error drilldowns',
    ],
    next: [
      'Feed diagnostics from command results',
      'Preserve code, severity, file, and range fields exactly',
    ],
  },
  {
    path: '/inspect',
    label: 'Inspect',
    eyebrow: 'WB-0.3 Inspect',
    title: 'Disasm and verify before richer debugging.',
    summary:
      'Inspect is where Workbench will render SemCode, verifier output, and runtime summaries. It stays downstream from existing execution contracts.',
    stable: [
      'Dedicated trace, verify, disasm, and verified-run inspectors over real CLI jobs',
      'Dedicated inspector over smc verify, svm disasm, and verified-run jobs',
      'Raw command output is preserved as the only bytecode, verifier, and runtime source of truth',
      'Clear note that source-level debugging is not promised yet',
    ],
    next: [
      'Extend the same inspect route with richer quota and capability context when public outputs grow',
      'Add trace and runtime summaries without inventing VM semantics',
    ],
  },
  {
    path: '/release',
    label: 'Release',
    eyebrow: 'WB-0.4 Operate',
    title: 'Stable hardening lives on one screen.',
    summary:
      'Release is the eventual command center for gates, bundle verification, asset smoke, docs alignment, and known limits. Every signal must remain explainable.',
    stable: [
      'Release route anchored around canonical release docs and baseline artifacts',
      'Known-limits panel separated from pass/fail gates',
      'Freshness hints and source paths shown for release-facing documents',
    ],
    next: [
      'Export a validation report from real job history',
      'Keep release map in sync with future operate slices without adding UI-owned scoring',
    ],
  },
  {
    path: '/settings',
    label: 'Settings',
    eyebrow: 'WB-0 Bootstrap',
    title: 'Local preferences, not hidden feature flags.',
    summary:
      'Settings is for shell-level behavior only: display, formatter preferences, shell defaults, and workspace affordances. It must not widen Semantic scope.',
    stable: [
      'Settings route with persisted local preferences',
      'Scope guard against hidden runtime or language toggles',
      'Formatter and shell preference toggles wired only to canonical public surfaces',
    ],
    next: [
      'Route settings into later formatter and command surfaces',
      'Keep experimentation visibly labeled and opt-in',
    ],
  },
]

function App() {
  return <div />
}

export default App
