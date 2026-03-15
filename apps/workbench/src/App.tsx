import { startTransition, useEffect, useState, type ReactNode } from 'react'
import { NavLink, Route, Routes } from 'react-router-dom'
import {
  fetchAdapterContract,
  fetchOverviewSnapshot,
  fetchSpecCatalog,
  fetchSpecDocument,
  resolveWorkspaceRoot,
  runCliJob,
  type AdapterContract,
  type AdapterJobSpec,
  type JobKind,
  type JobResult,
  type OverviewSnapshot,
  type SpecCatalogSection,
  type SpecDocumentHeading,
  type SpecDocumentView,
  type WorkspaceSummary,
} from './workbench-api'
import {
  loadWorkbenchState,
  mergeRecentWorkspace,
  saveWorkbenchState,
  type RecentWorkspace,
  type WorkbenchSettings,
} from './workbench-state'
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
    kind: 'release_bundle_verify',
    label: 'Verify release bundle',
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
    eyebrow: 'WB-0 Bootstrap',
    title: 'Workspace context before orchestration.',
    summary:
      'Project owns workspace selection, recent roots, and local settings. It does not create an alternate package or repository model.',
    stable: [
      'Workspace resolver over canonical repository paths',
      'Recent projects list and default workspace persistence',
      'Explicit rule that all jobs inherit the selected root',
    ],
    next: [
      'Add native directory-pick affordances if needed',
      'Keep project context read-only with respect to repository semantics',
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
      'Route reserved for disasm, verify, trace, and quota summaries',
      'Clear note that source-level debugging is not promised yet',
      'Explicit separation between inspection and execution ownership',
    ],
    next: [
      'Render real svm disasm output and verified-path summaries',
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
      'Formatter and shell preference toggles',
    ],
    next: [
      'Route settings into later formatter and command surfaces',
      'Keep experimentation visibly labeled and opt-in',
    ],
  },
]

function App() {
  const [adapterContract, setAdapterContract] = useState<AdapterContract | null>(
    null,
  )
  const [overviewSnapshot, setOverviewSnapshot] = useState<OverviewSnapshot | null>(
    null,
  )
  const [adapterError, setAdapterError] = useState<string | null>(null)
  const [snapshotError, setSnapshotError] = useState<string | null>(null)
  const [specError, setSpecError] = useState<string | null>(null)
  const [jobs, setJobs] = useState<JobRecord[]>([])
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null)
  const [activeJob, setActiveJob] = useState<JobKind | null>(null)
  const [specCatalog, setSpecCatalog] = useState<SpecCatalogSection[]>([])
  const [specSearch, setSpecSearch] = useState('')
  const [selectedSpecPath, setSelectedSpecPath] = useState<string | null>(null)
  const [selectedSpecDocument, setSelectedSpecDocument] =
    useState<SpecDocumentView | null>(null)
  const [workspaceInput, setWorkspaceInput] = useState('')
  const [workspaceError, setWorkspaceError] = useState<string | null>(null)
  const [selectedWorkspace, setSelectedWorkspace] = useState<WorkspaceSummary | null>(
    null,
  )
  const [recentWorkspaces, setRecentWorkspaces] = useState<RecentWorkspace[]>(
    initialWorkbenchState.recentWorkspaces,
  )
  const [settings, setSettings] = useState<WorkbenchSettings>(
    initialWorkbenchState.settings,
  )

  useEffect(() => {
    let cancelled = false

    fetchAdapterContract()
      .then((contract) => {
        if (!cancelled) {
          startTransition(() => setAdapterContract(contract))
          setAdapterError(null)
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setAdapterError(String(error))
        }
      })

    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    let cancelled = false

    fetchOverviewSnapshot()
      .then((snapshot) => {
        if (!cancelled) {
          startTransition(() => setOverviewSnapshot(snapshot))
          setSnapshotError(null)
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setSnapshotError(String(error))
        }
      })

    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    let cancelled = false

    fetchSpecCatalog()
      .then((catalog) => {
        if (!cancelled) {
          setSpecCatalog(catalog)
          setSpecError(null)
          const firstPath = catalog[0]?.documents[0]?.relativePath
          if (firstPath) {
            setSelectedSpecPath((current) => current ?? firstPath)
          }
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setSpecError(String(error))
        }
      })

    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    if (!selectedSpecPath) {
      return
    }

    let cancelled = false

    fetchSpecDocument(selectedSpecPath)
      .then((document) => {
        if (!cancelled) {
          setSelectedSpecDocument(document)
          setSpecError(null)
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setSpecError(String(error))
        }
      })

    return () => {
      cancelled = true
    }
  }, [selectedSpecPath])

  useEffect(() => {
    saveWorkbenchState({
      recentWorkspaces,
      settings,
    })
  }, [recentWorkspaces, settings])

  useEffect(() => {
    if (!adapterContract || selectedWorkspace) {
      return
    }

    const initialWorkspacePath =
      settings.defaultWorkspacePath ?? adapterContract.repoRoot

    void (async () => {
      try {
        const workspace = await resolveWorkspaceRoot(initialWorkspacePath)
        setWorkspaceError(null)
        setSelectedWorkspace(workspace)
        setWorkspaceInput(workspace.resolvedPath)
      } catch (error) {
        setWorkspaceError(String(error))
      }
    })()
  }, [adapterContract, selectedWorkspace, settings.defaultWorkspacePath])

  async function runJobAction(action: JobActionSpec) {
    const id = crypto.randomUUID()
    const cwd =
      action.cwdMode === 'repo'
        ? adapterContract?.repoRoot ?? selectedWorkspace?.repoRoot ?? ''
        : selectedWorkspace?.resolvedPath ?? adapterContract?.repoRoot ?? ''

    startTransition(() =>
      setJobs((current) => [
        {
          id,
          kind: action.kind,
          label: action.label,
          status: 'running',
          commandLine: [action.label, ...action.args].join(' '),
          cwd,
          stdout: '',
          stderr: '',
        },
        ...current,
      ]),
    )
    setSelectedJobId(id)
    setActiveJob(action.kind)

    try {
      const result = await runCliJob({
        kind: action.kind,
        args: action.args,
        cwd,
      })
      setAdapterError(null)
      commitJob(id, action.label, result)
    } catch (error) {
      const message = String(error)
      startTransition(() =>
        setJobs((current) =>
          current.map((job) =>
            job.id === id
              ? {
                  ...job,
                  status: 'failed',
                  stderr: message,
                }
              : job,
          ),
        ),
      )
      setAdapterError(message)
    } finally {
      setActiveJob(null)
    }
  }

  async function runProbe(spec: AdapterJobSpec) {
    await runJobAction({
      kind: spec.kind,
      label: `${spec.label} probe`,
      args: spec.exampleArgs,
      notes: spec.notes,
      cwdMode: 'workspace',
    })
  }

  function commitJob(id: string, label: string, result: JobResult) {
    startTransition(() =>
      setJobs((current) =>
        current.map((job) =>
          job.id === id
            ? {
                ...job,
                label,
                status: result.success ? 'success' : 'failed',
                commandLine: result.resolvedCommand.join(' '),
                cwd: result.cwd,
                durationMs: result.durationMs,
                exitCode: result.exitCode,
                stdout: result.stdout,
                stderr: result.stderr,
              }
            : job,
        ),
      ),
    )
  }

  async function openWorkspace(candidate: string, persist = true) {
    try {
      const workspace = await resolveWorkspaceRoot(candidate)
      setWorkspaceError(null)
      setSelectedWorkspace(workspace)
      setWorkspaceInput(workspace.resolvedPath)
      if (persist) {
        setRecentWorkspaces((current) => mergeRecentWorkspace(current, workspace))
      }
      setSettings((current) => ({
        ...current,
        defaultWorkspacePath: workspace.resolvedPath,
      }))
    } catch (error) {
      setWorkspaceError(String(error))
    }
  }

  function updateSettings(next: Partial<WorkbenchSettings>) {
    setSettings((current) => ({
      ...current,
      ...next,
    }))
  }

  return (
    <div className="workbench-shell">
      <aside className="sidebar">
        <div className="brand-block">
          <p className="eyebrow">Semantic Workbench</p>
          <h1>Workbench shell for orchestration, not reinvention.</h1>
          <p className="brand-copy">
            This bootstrap slice locks the desktop shell, route map, and public-surface discipline before any command or editor logic arrives.
          </p>
        </div>

        <nav className="primary-nav" aria-label="Workbench routes">
          {routeSpecs.map((route) => (
            <NavLink
              key={route.path}
              to={route.path}
              end={route.path === '/'}
              className={({ isActive }) =>
                isActive ? 'nav-link nav-link-active' : 'nav-link'
              }
            >
              <span className="nav-label">{route.label}</span>
              <span className="nav-meta">{route.eyebrow}</span>
            </NavLink>
          ))}
        </nav>

        <section className="sidebar-card">
          <p className="card-kicker">Source of truth</p>
          <ul className="bullet-list compact">
            <li>`docs/spec/*` and `docs/roadmap/*` remain canonical.</li>
            <li>`smc`, `svm`, `cargo`, and release scripts stay the first integration path.</li>
            <li>Workbench owns only UI state, orchestration, and presentation caches.</li>
          </ul>
        </section>

        <section className="sidebar-card">
          <p className="card-kicker">Workspace context</p>
          <p className="sidebar-strong">
            {selectedWorkspace?.repoRelativePath ?? 'repository root'}
          </p>
          <p className="sidebar-copy">
            {selectedWorkspace?.resolvedPath ??
              adapterContract?.repoRoot ??
              'Loading workspace root...'}
          </p>
        </section>
      </aside>

      <main className="main-panel">
        <header className="topbar">
          <div>
            <p className="eyebrow">WB-0.1 Cockpit</p>
            <h2>Operations cockpit from repository truth, not UI guesswork</h2>
          </div>
          <div className="status-cluster">
            <span className="status-pill stable">Stable now: shell, adapter contract, workspace context</span>
            <span className="status-pill draft">Draft target: richer diagnostics, formatter, and inspectors</span>
          </div>
        </header>

        <section className="hero-grid">
          <article className="hero-card">
            <p className="card-kicker">Current slice</p>
            <h3>Repository truth and runnable workflows now sit together</h3>
            <p>
              The overview now surfaces branch, commit, baseline tag, release documents, known limits, and a command runner over approved public workflows.
            </p>
          </article>
          <article className="hero-card">
            <p className="card-kicker">Do not cross</p>
            <h3>No alternate readiness model inside the UI</h3>
            <p>
              Workbench stores only local UI state. Readiness, compatibility, and release validity still come from repository docs and real command output.
            </p>
          </article>
          <article className="hero-card">
            <p className="card-kicker">Immediate next</p>
            <h3>Spec navigation and authoring shell</h3>
            <p>
              `WB-07` and `WB-09` should extend this cockpit into spec browsing and the first editor-facing loop without inventing parser, verifier, or runtime semantics.
            </p>
          </article>
        </section>

        <Routes>
          {routeSpecs.map((route) => (
            <Route
              key={route.path}
              path={route.path}
              element={
                <WorkbenchScreen
                  route={route}
                  adapterContract={adapterContract}
                  overviewSnapshot={overviewSnapshot}
                  adapterError={adapterError}
                  snapshotError={snapshotError}
                  specCatalog={specCatalog}
                  specError={specError}
                  specSearch={specSearch}
                  selectedSpecDocument={selectedSpecDocument}
                  selectedSpecPath={selectedSpecPath}
                  jobs={jobs}
                  selectedJobId={selectedJobId}
                  activeJob={activeJob}
                  onRunAction={runJobAction}
                  onRunProbe={runProbe}
                  onSpecSearchChange={setSpecSearch}
                  onSelectSpecPath={setSelectedSpecPath}
                  onSelectJob={setSelectedJobId}
                  selectedWorkspace={selectedWorkspace}
                  workspaceInput={workspaceInput}
                  workspaceError={workspaceError}
                  recentWorkspaces={recentWorkspaces}
                  settings={settings}
                  onWorkspaceInputChange={setWorkspaceInput}
                  onOpenWorkspace={openWorkspace}
                  onUpdateSettings={updateSettings}
                />
              }
            />
          ))}
        </Routes>
      </main>
    </div>
  )
}

function WorkbenchScreen({
  route,
  adapterContract,
  overviewSnapshot,
  adapterError,
  snapshotError,
  specCatalog,
  specError,
  specSearch,
  selectedSpecDocument,
  selectedSpecPath,
  jobs,
  selectedJobId,
  activeJob,
  onRunAction,
  onRunProbe,
  onSpecSearchChange,
  onSelectSpecPath,
  onSelectJob,
  selectedWorkspace,
  workspaceInput,
  workspaceError,
  recentWorkspaces,
  settings,
  onWorkspaceInputChange,
  onOpenWorkspace,
  onUpdateSettings,
}: {
  route: ScreenSpec
  adapterContract: AdapterContract | null
  overviewSnapshot: OverviewSnapshot | null
  adapterError: string | null
  snapshotError: string | null
  specCatalog: SpecCatalogSection[]
  specError: string | null
  specSearch: string
  selectedSpecDocument: SpecDocumentView | null
  selectedSpecPath: string | null
  jobs: JobRecord[]
  selectedJobId: string | null
  activeJob: JobKind | null
  onRunAction: (action: JobActionSpec) => Promise<void>
  onRunProbe: (spec: AdapterJobSpec) => Promise<void>
  onSpecSearchChange: (value: string) => void
  onSelectSpecPath: (value: string) => void
  onSelectJob: (jobId: string) => void
  selectedWorkspace: WorkspaceSummary | null
  workspaceInput: string
  workspaceError: string | null
  recentWorkspaces: RecentWorkspace[]
  settings: WorkbenchSettings
  onWorkspaceInputChange: (value: string) => void
  onOpenWorkspace: (candidate: string, persist?: boolean) => Promise<void>
  onUpdateSettings: (next: Partial<WorkbenchSettings>) => void
}) {
  return (
    <div className="screen-stack">
      <section className="screen-grid">
        <article className="screen-card screen-card-primary">
          <p className="card-kicker">{route.eyebrow}</p>
          <h3>{route.title}</h3>
          <p className="screen-summary">{route.summary}</p>
        </article>

        <article className="screen-card">
          <p className="card-kicker">In this slice</p>
          <ul className="bullet-list">
            {route.stable.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Next implementation steps</p>
          <ul className="bullet-list">
            {route.next.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </article>
      </section>

      {route.path === '/' ? (
        <CommandBusPanel
          adapterContract={adapterContract}
          overviewSnapshot={overviewSnapshot}
          adapterError={adapterError}
          snapshotError={snapshotError}
          jobs={jobs}
          selectedJobId={selectedJobId}
          activeJob={activeJob}
          onRunAction={onRunAction}
          onRunProbe={onRunProbe}
          onSelectJob={onSelectJob}
          selectedWorkspace={selectedWorkspace}
        />
      ) : null}

      {route.path === '/spec' ? (
        <SpecNavigatorPanel
          specCatalog={specCatalog}
          specError={specError}
          specSearch={specSearch}
          selectedSpecDocument={selectedSpecDocument}
          selectedSpecPath={selectedSpecPath}
          onSpecSearchChange={onSpecSearchChange}
          onSelectSpecPath={onSelectSpecPath}
        />
      ) : null}

      {route.path === '/release' ? (
        <ReleasePanel
          overviewSnapshot={overviewSnapshot}
          specCatalog={specCatalog}
          jobs={jobs}
          selectedWorkspace={selectedWorkspace}
        />
      ) : null}

      {route.path === '/project' ? (
        <ProjectPanel
          adapterContract={adapterContract}
          selectedWorkspace={selectedWorkspace}
          workspaceInput={workspaceInput}
          workspaceError={workspaceError}
          recentWorkspaces={recentWorkspaces}
          onWorkspaceInputChange={onWorkspaceInputChange}
          onOpenWorkspace={onOpenWorkspace}
        />
      ) : null}

      {route.path === '/settings' ? (
        <SettingsPanel
          settings={settings}
          selectedWorkspace={selectedWorkspace}
          onUpdateSettings={onUpdateSettings}
        />
      ) : null}
    </div>
  )
}

function CommandBusPanel({
  adapterContract,
  overviewSnapshot,
  adapterError,
  snapshotError,
  jobs,
  selectedJobId,
  activeJob,
  onRunAction,
  onRunProbe,
  onSelectJob,
  selectedWorkspace,
}: {
  adapterContract: AdapterContract | null
  overviewSnapshot: OverviewSnapshot | null
  adapterError: string | null
  snapshotError: string | null
  jobs: JobRecord[]
  selectedJobId: string | null
  activeJob: JobKind | null
  onRunAction: (action: JobActionSpec) => Promise<void>
  onRunProbe: (spec: AdapterJobSpec) => Promise<void>
  onSelectJob: (jobId: string) => void
  selectedWorkspace: WorkspaceSummary | null
}) {
  const latestCargo = latestJobOfKind(jobs, 'cargo')
  const latestSmc = latestJobOfKind(jobs, 'smc')
  const latestSvm = latestJobOfKind(jobs, 'svm')
  const latestBundle = latestJobOfKind(jobs, 'release_bundle_verify')
  const selectedJob = jobs.find((job) => job.id === selectedJobId) ?? jobs[0]

  return (
    <div className="screen-stack">
      <section className="overview-grid">
        <article className="screen-card">
          <p className="card-kicker">Repository snapshot</p>
          <h3>Current git and baseline identity</h3>
          {snapshotError ? <p className="adapter-error">{snapshotError}</p> : null}
          <dl className="facts-grid">
            <div>
              <dt>Branch</dt>
              <dd>{overviewSnapshot?.branch ?? 'Loading...'}</dd>
            </div>
            <div>
              <dt>Commit</dt>
              <dd>
                <code>{overviewSnapshot?.shortCommit ?? 'Loading...'}</code>
              </dd>
            </div>
            <div>
              <dt>Head tags</dt>
              <dd>
                {overviewSnapshot?.headTags.length ? (
                  overviewSnapshot.headTags.join(', ')
                ) : (
                  'No tags on HEAD'
                )}
              </dd>
            </div>
            <div>
              <dt>Baseline tag</dt>
              <dd>
                {overviewSnapshot
                  ? overviewSnapshot.baselineTagPointsAtHead
                    ? `${overviewSnapshot.baselineTagName} on HEAD`
                    : `${overviewSnapshot.baselineTagName} exists off HEAD`
                  : 'Loading...'}
              </dd>
            </div>
            <div className="facts-grid-wide">
              <dt>Baseline manifest</dt>
              <dd>
                {overviewSnapshot?.baselineManifestExists ? 'Present' : 'Missing'}
                {overviewSnapshot ? (
                  <>
                    {' '}
                    <code>{overviewSnapshot.baselineManifestPath}</code>
                  </>
                ) : null}
              </dd>
            </div>
            <div className="facts-grid-wide">
              <dt>Workspace root</dt>
              <dd>
                <code>{selectedWorkspace?.resolvedPath ?? adapterContract?.repoRoot ?? 'Loading...'}</code>
              </dd>
            </div>
          </dl>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Baseline guardrails</p>
          <h3>What the cockpit is allowed to claim</h3>
          <ul className="bullet-list">
            <li>Only git state, release docs, manifests, and real job executions drive the overview.</li>
            <li>Known limits stay visible even when the last local commands were green.</li>
            <li>Published readiness still belongs to repository artifacts, not to cached UI percentages.</li>
          </ul>
        </article>
      </section>

      <section className="overview-grid">
        <article className="screen-card">
          <p className="card-kicker">Release docs</p>
          <h3>Readiness and compatibility pointers</h3>
          <div className="document-list">
            {(overviewSnapshot?.releaseDocs ?? []).map((document) => (
              <section key={document.key} className="document-card">
                <div className="document-topline">
                  <strong>{document.title}</strong>
                  {document.status ? (
                    <span className="status-pill stable">{document.status}</span>
                  ) : null}
                </div>
                <p className="job-meta">
                  <code>{document.path}</code>
                </p>
                {document.highlight ? (
                  <p className="document-highlight">{document.highlight}</p>
                ) : null}
              </section>
            ))}
          </div>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Current known limits</p>
          <h3>Honesty rules carried from readiness</h3>
          <ul className="bullet-list">
            {(overviewSnapshot?.knownLimits ?? []).map((limit) => (
              <li key={limit}>{limit}</li>
            ))}
          </ul>
        </article>
      </section>

      <section className="command-grid">
        <article className="screen-card">
          <p className="card-kicker">Command runner</p>
          <h3>Core workflows through approved public surfaces</h3>
          <p className="screen-summary">
            These actions run real repository workflows through the backend adapter. Repository-wide validation always executes from the repository root.
          </p>
          <div className="workflow-grid">
            {workflowActions.map((action) => (
              <section key={action.label} className="workflow-card">
                <div className="adapter-header">
                  <h4>{action.label}</h4>
                  <span className="status-pill stable">{action.kind}</span>
                </div>
                <p>{action.notes}</p>
                <p className="job-meta">
                  scope:{' '}
                  <strong>
                    {action.cwdMode === 'repo' ? 'repository root' : 'current workspace'}
                  </strong>
                </p>
                <code className="code-block">
                  {action.args.join(' ') || '(default script args)'}
                </code>
                <button
                  type="button"
                  className="action-button"
                  onClick={() => void onRunAction(action)}
                  disabled={activeJob === action.kind}
                >
                  {activeJob === action.kind ? 'Running command...' : `Run ${action.label}`}
                </button>
              </section>
            ))}
          </div>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Latest local validation activity</p>
          <h3>Signals from real commands</h3>
          <div className="activity-list">
            <ValidationRow label="Cargo" job={latestCargo} />
            <ValidationRow label="smc" job={latestSmc} />
            <ValidationRow label="svm" job={latestSvm} />
            <ValidationRow label="Release bundle verify" job={latestBundle} />
          </div>
        </article>
      </section>

      <section className="command-grid">
        <article className="screen-card">
          <p className="card-kicker">Adapter contract</p>
          <h3>Supported public command surfaces</h3>
          <p className="screen-summary">
            The backend adapter resolves only approved tools and keeps all job cwd values inside the repository root.
          </p>
          <div className="repo-root">
            <span className="repo-root-label">Active workspace root</span>
            <code>{adapterContract?.repoRoot ?? 'Loading adapter contract...'}</code>
          </div>
          <p className="job-meta">
            current workspace:{' '}
            <code>{selectedWorkspace?.resolvedPath ?? adapterContract?.repoRoot ?? 'Loading...'}</code>
          </p>
          {adapterError ? <p className="adapter-error">{adapterError}</p> : null}
          <div className="spec-grid">
            {(adapterContract?.jobs ?? []).map((spec) => (
              <section key={spec.kind} className="adapter-spec">
                <div className="adapter-header">
                  <h4>{spec.label}</h4>
                  <span className="status-pill draft">probe</span>
                </div>
                <p>{spec.notes}</p>
                <code className="code-block">{spec.resolution}</code>
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => void onRunProbe(spec)}
                  disabled={activeJob === spec.kind}
                >
                  {activeJob === spec.kind ? 'Running probe...' : `Run ${spec.label} probe`}
                </button>
              </section>
            ))}
          </div>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Job history</p>
          <h3>Deterministic execution ledger</h3>
          <div className="job-list">
            {jobs.length === 0 ? (
              <p className="empty-state">
                No jobs yet. Run a workflow or probe to populate the execution ledger.
              </p>
            ) : (
              jobs.map((job) => (
                <button
                  key={job.id}
                  type="button"
                  className={`job-card job-card-button ${selectedJob?.id === job.id ? 'job-card-selected' : ''}`}
                  onClick={() => onSelectJob(job.id)}
                >
                  <div className="job-topline">
                    <div>
                      <strong>{job.label}</strong>
                      <p className="job-meta">{job.commandLine}</p>
                    </div>
                    <span className={`status-pill ${job.status}`}>{job.status}</span>
                  </div>
                  <p className="job-meta">
                    cwd: <code>{job.cwd}</code>
                  </p>
                  <p className="job-meta">
                    exit: {job.exitCode ?? 'pending'} | duration:{' '}
                    {job.durationMs !== undefined ? `${job.durationMs} ms` : 'running'}
                  </p>
                </button>
              ))
            )}
          </div>
        </article>
      </section>

      <section className="command-grid">
        <article className="screen-card">
          <p className="card-kicker">Command output</p>
          <h3>Stdout and stderr without terminal archaeology</h3>
          {selectedJob ? (
            <div className="job-detail-stack">
              <p className="job-meta">
                command: <code>{selectedJob.commandLine}</code>
              </p>
              <p className="job-meta">
                cwd: <code>{selectedJob.cwd}</code>
              </p>
              <p className="job-meta">
                exit: {selectedJob.exitCode ?? 'pending'} | duration:{' '}
                {selectedJob.durationMs !== undefined
                  ? `${selectedJob.durationMs} ms`
                  : 'running'}
              </p>
              <div className="job-output-stack">
                <section>
                  <p className="card-kicker">stdout</p>
                  {selectedJob.stdout ? (
                    <pre className="terminal-output">{selectedJob.stdout}</pre>
                  ) : (
                    <p className="empty-state">No stdout captured for this job.</p>
                  )}
                </section>
                <section>
                  <p className="card-kicker">stderr</p>
                  {selectedJob.stderr ? (
                    <pre className="terminal-output terminal-output-error">
                      {selectedJob.stderr}
                    </pre>
                  ) : (
                    <p className="empty-state">No stderr captured for this job.</p>
                  )}
                </section>
              </div>
            </div>
          ) : (
            <p className="empty-state">
              Select a job from the history to inspect its resolved command and captured output.
            </p>
          )}
        </article>

        <article className="screen-card">
          <p className="card-kicker">Execution rule</p>
          <h3>Jobs remain explainable and reproducible</h3>
          <ul className="bullet-list">
            <li>Every command records its resolved invocation, cwd, exit code, and duration.</li>
            <li>Repository-wide workflows run from the repository root even if the active workspace points deeper.</li>
            <li>Workbench still does not interpret Semantic semantics; it only runs and presents public surfaces.</li>
          </ul>
        </article>
      </section>
    </div>
  )
}

function ValidationRow({
  label,
  job,
}: {
  label: string
  job?: JobRecord
}) {
  return (
    <section className="validation-row">
      <div>
        <strong>{label}</strong>
        <p className="job-meta">{job ? job.commandLine : 'No local run yet'}</p>
      </div>
      <span className={`status-pill ${job?.status ?? 'draft'}`}>
        {job?.status ?? 'not run'}
      </span>
    </section>
  )
}

function SpecNavigatorPanel({
  specCatalog,
  specError,
  specSearch,
  selectedSpecDocument,
  selectedSpecPath,
  onSpecSearchChange,
  onSelectSpecPath,
}: {
  specCatalog: SpecCatalogSection[]
  specError: string | null
  specSearch: string
  selectedSpecDocument: SpecDocumentView | null
  selectedSpecPath: string | null
  onSpecSearchChange: (value: string) => void
  onSelectSpecPath: (value: string) => void
}) {
  const query = specSearch.trim().toLowerCase()
  const filteredSections = specCatalog
    .map((section) => ({
      ...section,
      documents: section.documents.filter((document) => {
        if (!query) {
          return true
        }

        const haystack = `${document.title} ${document.relativePath}`.toLowerCase()
        return haystack.includes(query)
      }),
    }))
    .filter((section) => section.documents.length > 0)

  return (
    <section className="spec-shell">
      <article className="screen-card spec-sidebar-panel">
        <p className="card-kicker">Canonical document tree</p>
        <h3>Spec and roadmap navigator</h3>
        <label className="field-label" htmlFor="spec-search">
          Search titles and paths
        </label>
        <input
          id="spec-search"
          className="text-field"
          type="text"
          value={specSearch}
          onChange={(event) => onSpecSearchChange(event.target.value)}
          placeholder="Search syntax, vm, readiness, release..."
        />
        {specError ? <p className="adapter-error">{specError}</p> : null}
        <div className="spec-section-list">
          {filteredSections.length === 0 ? (
            <p className="empty-state">No canonical documents match the current search.</p>
          ) : (
            filteredSections.map((section) => (
              <section key={section.key} className="spec-section-card">
                <p className="card-kicker">{section.title}</p>
                <div className="spec-doc-list">
                  {section.documents.map((document) => (
                    <button
                      key={document.relativePath}
                      type="button"
                      className={`spec-doc-button ${selectedSpecPath === document.relativePath ? 'spec-doc-button-active' : ''}`}
                      onClick={() => onSelectSpecPath(document.relativePath)}
                    >
                      <span className="spec-doc-title">{document.title}</span>
                      <span className="spec-doc-path">{document.relativePath}</span>
                      {document.status ? (
                        <span className={`status-pill ${statusTone(document.status)}`}>
                          {document.status}
                        </span>
                      ) : null}
                    </button>
                  ))}
                </div>
              </section>
            ))
          )}
        </div>
      </article>

      <article className="screen-card spec-document-panel">
        {selectedSpecDocument ? (
          <div className="screen-stack">
            <div className="document-topline">
              <div>
                <p className="card-kicker">{selectedSpecDocument.sectionTitle}</p>
                <h3>{selectedSpecDocument.title}</h3>
              </div>
              {selectedSpecDocument.status ? (
                <span className={`status-pill ${statusTone(selectedSpecDocument.status)}`}>
                  {selectedSpecDocument.status}
                </span>
              ) : null}
            </div>
            <p className="job-meta">
              source path: <code>{selectedSpecDocument.absolutePath}</code>
            </p>
            <div className="spec-document-grid">
              <aside className="spec-outline-panel">
                <p className="card-kicker">Section navigator</p>
                <div className="spec-outline-list">
                  {selectedSpecDocument.headings.map((heading) => (
                    <button
                      key={heading.anchor}
                      type="button"
                      className={`spec-outline-button spec-outline-level-${heading.level}`}
                      onClick={() => jumpToHeading(heading.anchor)}
                    >
                      {heading.title}
                    </button>
                  ))}
                </div>
              </aside>

              <div className="markdown-sheet">
                {renderMarkdown(selectedSpecDocument.markdown, selectedSpecDocument.headings)}
              </div>
            </div>
          </div>
        ) : (
          <p className="empty-state">
            Select a canonical document to inspect its headings and body.
          </p>
        )}
      </article>
    </section>
  )
}

function ReleasePanel({
  overviewSnapshot,
  specCatalog,
  jobs,
  selectedWorkspace,
}: {
  overviewSnapshot: OverviewSnapshot | null
  specCatalog: SpecCatalogSection[]
  jobs: JobRecord[]
  selectedWorkspace: WorkspaceSummary | null
}) {
  const releaseSection = specCatalog.find((section) => section.key === 'release')
  const latestCargo = latestJobOfKind(jobs, 'cargo')
  const latestSmc = latestJobOfKind(jobs, 'smc')
  const latestSvm = latestJobOfKind(jobs, 'svm')
  const latestBundle = latestJobOfKind(jobs, 'release_bundle_verify')

  return (
    <div className="screen-stack">
      <section className="overview-grid">
        <article className="screen-card">
          <p className="card-kicker">Release identity</p>
          <h3>What the current line is anchored to</h3>
          <dl className="facts-grid">
            <div>
              <dt>Branch</dt>
              <dd>{overviewSnapshot?.branch ?? 'Loading...'}</dd>
            </div>
            <div>
              <dt>HEAD</dt>
              <dd>
                <code>{overviewSnapshot?.shortCommit ?? 'Loading...'}</code>
              </dd>
            </div>
            <div className="facts-grid-wide">
              <dt>Baseline tag</dt>
              <dd>
                {overviewSnapshot
                  ? overviewSnapshot.baselineTagPointsAtHead
                    ? `${overviewSnapshot.baselineTagName} on HEAD`
                    : `${overviewSnapshot.baselineTagName} exists off HEAD`
                  : 'Loading...'}
              </dd>
            </div>
            <div className="facts-grid-wide">
              <dt>Baseline manifest</dt>
              <dd>
                {overviewSnapshot?.baselineManifestExists ? 'Present' : 'Missing'}
                {overviewSnapshot ? (
                  <>
                    {' '}
                    <code>{overviewSnapshot.baselineManifestPath}</code>
                  </>
                ) : null}
              </dd>
            </div>
            <div className="facts-grid-wide">
              <dt>Active workspace</dt>
              <dd>
                <code>{selectedWorkspace?.resolvedPath ?? overviewSnapshot?.repoRoot ?? 'Loading...'}</code>
              </dd>
            </div>
          </dl>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Release-critical jobs</p>
          <h3>Signals from actual command history</h3>
          <div className="activity-list">
            <ValidationRow label="Workspace tests" job={latestCargo} />
            <ValidationRow label="smc workflows" job={latestSmc} />
            <ValidationRow label="svm workflows" job={latestSvm} />
            <ValidationRow label="Bundle verification" job={latestBundle} />
          </div>
        </article>
      </section>

      <section className="command-grid">
        <article className="screen-card">
          <p className="card-kicker">Release-facing docs</p>
          <h3>Source paths and freshness from repository files</h3>
          <div className="document-list">
            {(releaseSection?.documents ?? []).map((document) => (
              <section key={document.relativePath} className="document-card">
                <div className="document-topline">
                  <strong>{document.title}</strong>
                  <span className={`status-pill ${statusTone(document.status ?? 'draft')}`}>
                    {document.status ?? 'draft'}
                  </span>
                </div>
                <p className="job-meta">
                  path: <code>{document.absolutePath}</code>
                </p>
                <p className="job-meta">
                  freshness: {formatFreshness(document.modifiedEpochMs)}
                </p>
              </section>
            ))}
          </div>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Readiness truths</p>
          <h3>Known limits and validated tag callouts</h3>
          <div className="screen-stack">
            <section className="document-card">
              <div className="document-topline">
                <strong>Known limits</strong>
                <span className="status-pill draft">honesty</span>
              </div>
              <ul className="bullet-list">
                {(overviewSnapshot?.knownLimits ?? []).map((limit) => (
                  <li key={limit}>{limit}</li>
                ))}
              </ul>
            </section>
            <section className="document-card">
              <div className="document-topline">
                <strong>Current validated tag</strong>
                <span className="status-pill stable">derived</span>
              </div>
              <div className="document-list">
                {(overviewSnapshot?.releaseDocs ?? [])
                  .filter((document) => document.highlight)
                  .map((document) => (
                    <div key={document.key}>
                      <p className="job-meta">
                        <code>{document.path}</code>
                      </p>
                      <p className="document-highlight">{document.highlight}</p>
                    </div>
                  ))}
              </div>
            </section>
          </div>
        </article>
      </section>

      <section className="command-grid">
        <article className="screen-card">
          <p className="card-kicker">No second release model</p>
          <h3>What the UI must not do</h3>
          <ul className="bullet-list">
            <li>Do not invent a release score that is not backed by repository docs or job output.</li>
            <li>Do not hide known limits behind green local commands.</li>
            <li>Do not claim stable readiness unless the canonical release documents and bundle checks say so.</li>
          </ul>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Next operate slices</p>
          <h3>What still belongs to later PRs</h3>
          <ul className="bullet-list">
            <li>`WB-16` will formalize the release console around gate views and artifact lists.</li>
            <li>`WB-17` will add one-click validation runs and report export.</li>
            <li>This slice stays presentation-only over current release artifacts.</li>
          </ul>
        </article>
      </section>
    </div>
  )
}

function jumpToHeading(anchor: string) {
  const element = globalThis.document?.getElementById(anchor)
  element?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function statusTone(status: string) {
  const normalized = status.toLowerCase()
  if (
    normalized.includes('stable') ||
    normalized.includes('validated') ||
    normalized.includes('ready')
  ) {
    return 'stable'
  }

  if (normalized.includes('failed')) {
    return 'failed'
  }

  return 'draft'
}

function formatFreshness(modifiedEpochMs: number | null) {
  if (!modifiedEpochMs) {
    return 'unknown'
  }

  const deltaMs = Date.now() - modifiedEpochMs
  const minute = 60_000
  const hour = 60 * minute
  const day = 24 * hour

  if (deltaMs < hour) {
    return `${Math.max(1, Math.round(deltaMs / minute))} min ago`
  }

  if (deltaMs < day) {
    return `${Math.max(1, Math.round(deltaMs / hour))} hr ago`
  }

  return `${Math.max(1, Math.round(deltaMs / day))} day ago`
}

function renderMarkdown(markdown: string, headings: SpecDocumentHeading[]) {
  const headingAnchors = new Map(headings.map((heading) => [heading.title, heading.anchor]))
  const lines = markdown.split(/\r?\n/)
  const nodes: ReactNode[] = []
  let index = 0

  while (index < lines.length) {
    const line = lines[index]
    const trimmed = line.trim()

    if (!trimmed) {
      index += 1
      continue
    }

    if (trimmed.startsWith('```')) {
      const codeLines: string[] = []
      index += 1
      while (index < lines.length && !lines[index].trim().startsWith('```')) {
        codeLines.push(lines[index])
        index += 1
      }
      index += 1
      nodes.push(
        <pre key={`code-${index}`} className="terminal-output">
          {codeLines.join('\n')}
        </pre>,
      )
      continue
    }

    const headingMatch = /^(#{1,3})\s+(.*)$/.exec(trimmed)
    if (headingMatch) {
      const level = headingMatch[1].length
      const title = headingMatch[2].trim()
      const anchor = headingAnchors.get(title) ?? `heading-${index}`
      if (level === 1) {
        nodes.push(
          <h1 key={anchor} id={anchor} className="markdown-heading markdown-heading-1">
            {title}
          </h1>,
        )
      } else if (level === 2) {
        nodes.push(
          <h2 key={anchor} id={anchor} className="markdown-heading markdown-heading-2">
            {title}
          </h2>,
        )
      } else {
        nodes.push(
          <h3 key={anchor} id={anchor} className="markdown-heading markdown-heading-3">
            {title}
          </h3>,
        )
      }
      index += 1
      continue
    }

    if (trimmed.startsWith('- ')) {
      const bullets: string[] = []
      while (index < lines.length && lines[index].trim().startsWith('- ')) {
        bullets.push(lines[index].trim().slice(2))
        index += 1
      }
      nodes.push(
        <ul key={`list-${index}`} className="bullet-list">
          {bullets.map((bullet) => (
            <li key={`${bullet}-${index}`}>{bullet}</li>
          ))}
        </ul>,
      )
      continue
    }

    const paragraphLines: string[] = []
    while (index < lines.length) {
      const next = lines[index].trim()
      if (!next || next.startsWith('#') || next.startsWith('- ') || next.startsWith('```')) {
        break
      }
      paragraphLines.push(lines[index].trim())
      index += 1
    }

    nodes.push(
      <p key={`p-${index}`} className="markdown-paragraph">
        {paragraphLines.join(' ')}
      </p>,
    )
  }

  return nodes
}

function ProjectPanel({
  adapterContract,
  selectedWorkspace,
  workspaceInput,
  workspaceError,
  recentWorkspaces,
  onWorkspaceInputChange,
  onOpenWorkspace,
}: {
  adapterContract: AdapterContract | null
  selectedWorkspace: WorkspaceSummary | null
  workspaceInput: string
  workspaceError: string | null
  recentWorkspaces: RecentWorkspace[]
  onWorkspaceInputChange: (value: string) => void
  onOpenWorkspace: (candidate: string, persist?: boolean) => Promise<void>
}) {
  return (
    <section className="command-grid">
      <article className="screen-card">
        <p className="card-kicker">Open workspace</p>
        <h3>Canonical root selection for every job</h3>
        <p className="screen-summary">
          Enter an absolute path or repository-relative path. The backend resolver canonicalizes it and refuses anything outside the repository boundary.
        </p>
        <label className="field-label" htmlFor="workspace-path">
          Workspace path
        </label>
        <div className="field-row">
          <input
            id="workspace-path"
            className="text-field"
            type="text"
            value={workspaceInput}
            onChange={(event) => onWorkspaceInputChange(event.target.value)}
            placeholder={adapterContract?.repoRoot ?? 'Loading repository root...'}
          />
          <button
            type="button"
            className="action-button"
            onClick={() =>
              void onOpenWorkspace(
                workspaceInput.trim() || adapterContract?.repoRoot || '',
              )
            }
            disabled={!adapterContract}
          >
            Open
          </button>
        </div>
        <div className="field-actions">
          <button
            type="button"
            className="ghost-button"
            onClick={() => void onOpenWorkspace(adapterContract?.repoRoot ?? '')}
            disabled={!adapterContract}
          >
            Use repository root
          </button>
          <button
            type="button"
            className="ghost-button"
            onClick={() => void onOpenWorkspace('examples')}
            disabled={!adapterContract}
          >
            Use `examples`
          </button>
          <button
            type="button"
            className="ghost-button"
            onClick={() => void onOpenWorkspace('docs')}
            disabled={!adapterContract}
          >
            Use `docs`
          </button>
        </div>
        {workspaceError ? <p className="adapter-error">{workspaceError}</p> : null}
        <div className="repo-root">
          <span className="repo-root-label">Selected workspace</span>
          <code>{selectedWorkspace?.resolvedPath ?? 'No workspace selected yet.'}</code>
        </div>
        <p className="job-meta">
          repo-relative:{' '}
          <code>{selectedWorkspace?.repoRelativePath ?? '(repository root)'}</code>
        </p>
      </article>

      <article className="screen-card">
        <p className="card-kicker">Recent projects</p>
        <h3>Persisted local workspace history</h3>
        <div className="job-list">
          {recentWorkspaces.length === 0 ? (
            <p className="empty-state">
              No recent workspaces yet. Opening a canonical root stores it locally for future sessions.
            </p>
          ) : (
            recentWorkspaces.map((workspace) => (
              <section key={workspace.path} className="job-card">
                <div className="job-topline">
                  <div>
                    <strong>{workspace.repoRelativePath ?? 'repository root'}</strong>
                    <p className="job-meta">{workspace.path}</p>
                  </div>
                  <button
                    type="button"
                    className="ghost-button"
                    onClick={() => void onOpenWorkspace(workspace.path)}
                  >
                    Reopen
                  </button>
                </div>
                <p className="job-meta">last opened: {workspace.openedAtIso}</p>
              </section>
            ))
          )}
        </div>
      </article>
    </section>
  )
}

function SettingsPanel({
  settings,
  selectedWorkspace,
  onUpdateSettings,
}: {
  settings: WorkbenchSettings
  selectedWorkspace: WorkspaceSummary | null
  onUpdateSettings: (next: Partial<WorkbenchSettings>) => void
}) {
  return (
    <section className="command-grid">
      <article className="screen-card">
        <p className="card-kicker">Local settings</p>
        <h3>Preferences that stay in the UI layer</h3>
        <div className="settings-grid">
          <label className="toggle-row">
            <span>
              <strong>Default workspace</strong>
              <p className="job-meta">
                Persist the currently selected canonical workspace for future sessions.
              </p>
            </span>
            <code>{settings.defaultWorkspacePath ?? selectedWorkspace?.resolvedPath ?? 'unset'}</code>
          </label>

          <label className="toggle-row">
            <span>
              <strong>Preferred shell</strong>
              <p className="job-meta">Current bootstrap only supports PowerShell-based release flows.</p>
            </span>
            <span className="status-pill stable">{settings.preferredShell}</span>
          </label>

          <label className="toggle-row toggle-row-interactive">
            <span>
              <strong>Format on save</strong>
              <p className="job-meta">Preference only. Formatter integration arrives in `WB-13`.</p>
            </span>
            <input
              type="checkbox"
              checked={settings.formatOnSave}
              onChange={(event) =>
                onUpdateSettings({ formatOnSave: event.target.checked })
              }
            />
          </label>

          <label className="toggle-row toggle-row-interactive">
            <span>
              <strong>Show experimental workflows</strong>
              <p className="job-meta">Controls visibility only; it must not widen Semantic scope.</p>
            </span>
            <input
              type="checkbox"
              checked={settings.showExperimental}
              onChange={(event) =>
                onUpdateSettings({ showExperimental: event.target.checked })
              }
            />
          </label>
        </div>
      </article>

      <article className="screen-card">
        <p className="card-kicker">Scope guard</p>
        <h3>What settings cannot do</h3>
        <ul className="bullet-list">
          <li>They cannot enable hidden runtime semantics.</li>
          <li>They cannot widen PROMETHEUS scope or alter capability rules.</li>
          <li>They cannot override repository truth for readiness or compatibility.</li>
          <li>They exist only for shell behavior and local workflow preferences.</li>
        </ul>
      </article>
    </section>
  )
}

function latestJobOfKind(jobs: JobRecord[], kind: JobKind) {
  return jobs.find((job) => job.kind === kind)
}

export default App
