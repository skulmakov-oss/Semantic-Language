import { invoke } from '@tauri-apps/api/core'

export type JobKind = 'smc' | 'svm' | 'cargo' | 'release_bundle_verify'

export type AdapterJobSpec = {
  kind: JobKind
  label: string
  resolution: string
  exampleArgs: string[]
  notes: string
}

export type AdapterContract = {
  repoRoot: string
  jobs: AdapterJobSpec[]
}

export type JobRequest = {
  kind: JobKind
  args: string[]
  cwd?: string
}

export type JobResult = {
  kind: JobKind
  resolvedCommand: string[]
  cwd: string
  exitCode: number
  durationMs: number
  success: boolean
  stdout: string
  stderr: string
}

export type WorkspaceSummary = {
  repoRoot: string
  resolvedPath: string
  repoRelativePath: string | null
  isRepoRoot: boolean
}

export type OverviewDocument = {
  key: string
  title: string
  path: string
  status: string | null
  highlight: string | null
}

export type OverviewSnapshot = {
  repoRoot: string
  branch: string
  headCommit: string
  shortCommit: string
  headTags: string[]
  baselineTagName: string
  baselineTagPointsAtHead: boolean
  baselineManifestPath: string
  baselineManifestExists: boolean
  releaseDocs: OverviewDocument[]
  knownLimits: string[]
}

export async function fetchAdapterContract() {
  return invoke<AdapterContract>('get_adapter_contract')
}

export async function runCliJob(request: JobRequest) {
  return invoke<JobResult>('run_cli_job', { request })
}

export async function resolveWorkspaceRoot(candidate?: string) {
  return invoke<WorkspaceSummary>('resolve_workspace_root', { candidate })
}

export async function fetchOverviewSnapshot() {
  return invoke<OverviewSnapshot>('get_overview_snapshot')
}
