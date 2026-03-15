import { startTransition, useEffect, useState } from 'react'
import { NavLink, Route, Routes } from 'react-router-dom'
import {
  fetchAdapterContract,
  runCliJob,
  type AdapterContract,
  type AdapterJobSpec,
  type JobKind,
  type JobResult,
} from './workbench-api'
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
  label: string
  status: 'running' | 'success' | 'failed'
  commandLine: string
  cwd: string
  durationMs?: number
  exitCode?: number
  stdout: string
  stderr: string
}

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
      'Real probe actions routed through the backend process adapter',
    ],
    next: [
      'Wire git and validation snapshots into overview state',
      'Add workspace-root selection instead of the fixed repository root',
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
      'Workspace cards and local context placeholders',
      'Recent projects panel and settings summary shell',
      'Explicit rule that all jobs inherit the selected root',
    ],
    next: [
      'Persist recent projects and settings through the backend adapter',
      'Expose canonical root metadata only',
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
      'Route and layout for spec and roadmap navigation',
      'Placeholder panels for trees, sections, and freshness indicators',
      'Source-path discipline called out directly in the UI',
    ],
    next: [
      'Index canonical spec and roadmap documents',
      'Add search and section navigation without mutating docs',
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
      'Release route anchored around gates, assets, and docs alignment',
      'Known-limits panel separated from pass/fail gates',
      'Reminder that release-valid comes only from real checks',
    ],
    next: [
      'Wire release bundle verification and smoke matrix status',
      'Export a validation report from real job history',
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
      'Settings route and local preference card shell',
      'Scope guard against hidden runtime or language toggles',
      'Formatter and shell preference placeholders',
    ],
    next: [
      'Persist local UI settings only',
      'Keep feature experimentation visibly labeled and opt-in',
    ],
  },
]

function App() {
  const [adapterContract, setAdapterContract] = useState<AdapterContract | null>(
    null,
  )
  const [adapterError, setAdapterError] = useState<string | null>(null)
  const [jobs, setJobs] = useState<JobRecord[]>([])
  const [activeJob, setActiveJob] = useState<JobKind | null>(null)

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

  async function runProbe(spec: AdapterJobSpec) {
    const id = crypto.randomUUID()

    startTransition(() =>
      setJobs((current) => [
        {
          id,
          label: spec.label,
          status: 'running',
          commandLine: [spec.label, ...spec.exampleArgs].join(' '),
          cwd: adapterContract?.repoRoot ?? '',
          stdout: '',
          stderr: '',
        },
        ...current,
      ]),
    )
    setActiveJob(spec.kind)

    try {
      const result = await runCliJob({
        kind: spec.kind,
        args: spec.exampleArgs,
      })
      setAdapterError(null)
      commitJob(id, spec.label, result)
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
      </aside>

      <main className="main-panel">
        <header className="topbar">
          <div>
            <p className="eyebrow">WB-03 Command bus and CLI adapter</p>
            <h2>Deterministic process orchestration over public Semantic tools</h2>
          </div>
          <div className="status-cluster">
            <span className="status-pill stable">Stable now: shell, routes, adapter contract</span>
            <span className="status-pill draft">Draft target: project open and workspace settings</span>
          </div>
        </header>

        <section className="hero-grid">
          <article className="hero-card">
            <p className="card-kicker">Current slice</p>
            <h3>Foundation before behavior</h3>
            <p>
              The shell now owns a deterministic job model and backend adapter while still refusing to absorb compiler, verifier, VM, or runtime semantics.
            </p>
          </article>
          <article className="hero-card">
            <p className="card-kicker">Do not cross</p>
            <h3>No second compiler, verifier, or runtime</h3>
            <p>
              Command execution is limited to `smc`, `svm`, `cargo`, and the release verification script. Private crate internals remain outside this boundary.
            </p>
          </article>
          <article className="hero-card">
            <p className="card-kicker">Immediate next</p>
            <h3>Project open and workspace settings</h3>
            <p>
              `WB-04` should replace the fixed repository root with user-selected workspace context and local settings.
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
                  adapterError={adapterError}
                  jobs={jobs}
                  activeJob={activeJob}
                  onRunProbe={runProbe}
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
  adapterError,
  jobs,
  activeJob,
  onRunProbe,
}: {
  route: ScreenSpec
  adapterContract: AdapterContract | null
  adapterError: string | null
  jobs: JobRecord[]
  activeJob: JobKind | null
  onRunProbe: (spec: AdapterJobSpec) => Promise<void>
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
          adapterError={adapterError}
          jobs={jobs}
          activeJob={activeJob}
          onRunProbe={onRunProbe}
        />
      ) : null}
    </div>
  )
}

function CommandBusPanel({
  adapterContract,
  adapterError,
  jobs,
  activeJob,
  onRunProbe,
}: {
  adapterContract: AdapterContract | null
  adapterError: string | null
  jobs: JobRecord[]
  activeJob: JobKind | null
  onRunProbe: (spec: AdapterJobSpec) => Promise<void>
}) {
  return (
    <section className="command-grid">
      <article className="screen-card">
        <p className="card-kicker">Adapter contract</p>
        <h3>Supported public command surfaces</h3>
        <p className="screen-summary">
          The backend adapter resolves only approved tools and keeps all job cwd values inside the repository root.
        </p>
        <div className="repo-root">
          <span className="repo-root-label">Repository root</span>
          <code>{adapterContract?.repoRoot ?? 'Loading adapter contract...'}</code>
        </div>
        {adapterError ? (
          <p className="adapter-error">{adapterError}</p>
        ) : null}
        <div className="spec-grid">
          {(adapterContract?.jobs ?? []).map((spec) => (
            <section key={spec.kind} className="adapter-spec">
              <div className="adapter-header">
                <h4>{spec.label}</h4>
                <span className="status-pill draft">{spec.kind}</span>
              </div>
              <p>{spec.notes}</p>
              <code className="code-block">{spec.resolution}</code>
              <button
                type="button"
                className="action-button"
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
        <p className="card-kicker">Deterministic jobs</p>
        <h3>Recent adapter executions</h3>
        <div className="job-list">
          {jobs.length === 0 ? (
            <p className="empty-state">
              No jobs yet. Run a probe to validate the adapter path without touching private crate internals.
            </p>
          ) : (
            jobs.map((job) => (
              <section key={job.id} className={`job-card job-card-${job.status}`}>
                <div className="job-topline">
                  <div>
                    <strong>{job.label}</strong>
                    <p className="job-meta">{job.commandLine}</p>
                  </div>
                  <span className={`status-pill ${job.status}`}>
                    {job.status}
                  </span>
                </div>
                <p className="job-meta">
                  cwd: <code>{job.cwd}</code>
                </p>
                <p className="job-meta">
                  exit: {job.exitCode ?? 'pending'} | duration:{' '}
                  {job.durationMs !== undefined ? `${job.durationMs} ms` : 'running'}
                </p>
                {job.stdout ? (
                  <pre className="terminal-output">{job.stdout}</pre>
                ) : null}
                {job.stderr ? (
                  <pre className="terminal-output terminal-output-error">
                    {job.stderr}
                  </pre>
                ) : null}
              </section>
            ))
          )}
        </div>
      </article>
    </section>
  )
}

export default App
