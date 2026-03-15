import { NavLink, Route, Routes } from 'react-router-dom'
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
      'Release and readiness status derived from repository docs',
      'Source-of-truth callouts for specs, roadmap, and release artifacts',
    ],
    next: [
      'Wire real git and validation snapshots through the CLI adapter',
      'Show known-limit notes from readiness and compatibility docs',
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
            <p className="eyebrow">WB-02 Bootstrap</p>
            <h2>React + TypeScript + Tauri desktop shell</h2>
          </div>
          <div className="status-cluster">
            <span className="status-pill stable">Stable now: shell and routes</span>
            <span className="status-pill draft">Draft target: command adapter</span>
          </div>
        </header>

        <section className="hero-grid">
          <article className="hero-card">
            <p className="card-kicker">Current slice</p>
            <h3>Foundation before behavior</h3>
            <p>
              The app shell exists, routes are real, and the layout already encodes the distinction between repository truth and Workbench presentation.
            </p>
          </article>
          <article className="hero-card">
            <p className="card-kicker">Do not cross</p>
            <h3>No second compiler, verifier, or runtime</h3>
            <p>
              The next PRs will add orchestration over public commands only. Private crate internals stay outside this application boundary.
            </p>
          </article>
          <article className="hero-card">
            <p className="card-kicker">Immediate next</p>
            <h3>Command bus and CLI adapter</h3>
            <p>
              `WB-03` wires deterministic jobs and process adapters over `smc`, `svm`, `cargo`, and release scripts.
            </p>
          </article>
        </section>

        <Routes>
          {routeSpecs.map((route) => (
            <Route
              key={route.path}
              path={route.path}
              element={<WorkbenchScreen route={route} />}
            />
          ))}
        </Routes>
      </main>
    </div>
  )
}

function WorkbenchScreen({ route }: { route: ScreenSpec }) {
  return (
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
  )
}

export default App
