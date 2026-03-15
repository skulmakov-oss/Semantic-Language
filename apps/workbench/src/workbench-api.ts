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

export async function fetchAdapterContract() {
  return invoke<AdapterContract>('get_adapter_contract')
}

export async function runCliJob(request: JobRequest) {
  return invoke<JobResult>('run_cli_job', { request })
}
