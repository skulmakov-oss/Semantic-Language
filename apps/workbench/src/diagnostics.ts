import type { JobKind } from './workbench-api'

export type DiagnosticSeverity = 'error' | 'warning' | 'info'

export type DiagnosticFamily =
  | 'parse'
  | 'policy'
  | 'type'
  | 'module'
  | 'verify'
  | 'runtime'
  | 'other'

export type DiagnosticSourceJob = {
  id: string
  kind: JobKind
  label: string
  status: 'running' | 'success' | 'failed'
  commandLine: string
  cwd: string
  resolvedCommand: string[]
  stdout: string
  stderr: string
}

export type WorkbenchDiagnostic = {
  id: string
  jobId: string
  jobKind: JobKind
  jobLabel: string
  family: DiagnosticFamily
  severity: DiagnosticSeverity
  code: string | null
  message: string
  filePath: string | null
  line: number | null
  column: number | null
  functionName: string | null
  instruction: number | null
  offsetHex: string | null
  helpText: string | null
  rawBlock: string
  sourceChannel: 'stdout' | 'stderr'
  commandLine: string
  cwd: string
}

const sourceDiagnosticPattern =
  /^(Error|Warning)\s+\[([A-Z]\d{4})\]:\s*(.*?)(?:\s+at line\s+(\d+):(\d+))?$/i
const verifyDiagnosticPattern =
  /^verify error \[([^\]]+)\](?: in '([^']+)')?(?: @(0x[0-9a-fA-F]+))?:\s*(.*)$/i
const runtimeDiagnosticPattern =
  /^runtime error(?: at instruction (\d+))?:\s*(.*)$/i
const ansiEscapePattern = new RegExp(`${String.fromCharCode(27)}\\[[0-9;]*m`, 'g')

export const diagnosticFamilyOrder: DiagnosticFamily[] = [
  'parse',
  'policy',
  'type',
  'module',
  'verify',
  'runtime',
  'other',
]

export function deriveDiagnosticsFromJobs(
  jobs: DiagnosticSourceJob[],
): WorkbenchDiagnostic[] {
  const diagnostics: WorkbenchDiagnostic[] = []

  jobs.forEach((job) => {
    diagnostics.push(...parseChannel(job, 'stderr', job.stderr))
    diagnostics.push(...parseChannel(job, 'stdout', job.stdout))
  })

  return diagnostics
}

export function diagnosticFamilyLabel(family: DiagnosticFamily) {
  switch (family) {
    case 'parse':
      return 'Parse'
    case 'policy':
      return 'Policy'
    case 'type':
      return 'Type'
    case 'module':
      return 'Module'
    case 'verify':
      return 'Verify'
    case 'runtime':
      return 'Runtime'
    default:
      return 'Other'
  }
}

function parseChannel(
  job: DiagnosticSourceJob,
  sourceChannel: 'stdout' | 'stderr',
  rawText: string,
): WorkbenchDiagnostic[] {
  const text = stripAnsi(rawText)
  const lines = text.split(/\r?\n/)
  const diagnostics: WorkbenchDiagnostic[] = []

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index]?.trimEnd() ?? ''
    const trimmed = line.trim()
    if (!trimmed) {
      continue
    }

    const sourceMatch = sourceDiagnosticPattern.exec(trimmed)
    if (sourceMatch) {
      const block = collectDiagnosticBlock(lines, index)
      index = block.nextIndex - 1
      const code = sourceMatch[2].toUpperCase()
      const message = sourceMatch[3].trim()
      diagnostics.push({
        id: `${job.id}-${sourceChannel}-${index}`,
        jobId: job.id,
        jobKind: job.kind,
        jobLabel: job.label,
        family: classifySourceFamily(code, message),
        severity:
          sourceMatch[1].toLowerCase() === 'warning' ? 'warning' : 'error',
        code,
        message,
        filePath: inferJobPrimaryPath(job),
        line: parseNumber(sourceMatch[4]),
        column: parseNumber(sourceMatch[5]),
        functionName: null,
        instruction: null,
        offsetHex: null,
        helpText: block.helpText,
        rawBlock: block.rawBlock,
        sourceChannel,
        commandLine: job.commandLine,
        cwd: job.cwd,
      })
      continue
    }

    const verifyMatch = verifyDiagnosticPattern.exec(trimmed)
    if (verifyMatch) {
      diagnostics.push({
        id: `${job.id}-${sourceChannel}-${index}`,
        jobId: job.id,
        jobKind: job.kind,
        jobLabel: job.label,
        family: 'verify',
        severity: 'error',
        code: verifyMatch[1].trim(),
        message: verifyMatch[4].trim(),
        filePath: inferJobPrimaryPath(job),
        line: null,
        column: null,
        functionName: verifyMatch[2] ?? null,
        instruction: null,
        offsetHex: verifyMatch[3] ?? null,
        helpText: null,
        rawBlock: trimmed,
        sourceChannel,
        commandLine: job.commandLine,
        cwd: job.cwd,
      })
      continue
    }

    const runtimeMatch = runtimeDiagnosticPattern.exec(trimmed)
    if (runtimeMatch) {
      diagnostics.push({
        id: `${job.id}-${sourceChannel}-${index}`,
        jobId: job.id,
        jobKind: job.kind,
        jobLabel: job.label,
        family: 'runtime',
        severity: 'error',
        code: null,
        message: runtimeMatch[2].trim(),
        filePath: inferJobPrimaryPath(job),
        line: null,
        column: null,
        functionName: null,
        instruction: parseNumber(runtimeMatch[1]),
        offsetHex: null,
        helpText: null,
        rawBlock: trimmed,
        sourceChannel,
        commandLine: job.commandLine,
        cwd: job.cwd,
      })
      continue
    }

    if (job.status === 'failed' && shouldPromoteLine(job, trimmed)) {
      diagnostics.push({
        id: `${job.id}-${sourceChannel}-${index}`,
        jobId: job.id,
        jobKind: job.kind,
        jobLabel: job.label,
        family: job.kind === 'svm' ? 'runtime' : 'other',
        severity: 'error',
        code: extractInlineCode(trimmed),
        message: trimmed,
        filePath: inferJobPrimaryPath(job),
        line: null,
        column: null,
        functionName: null,
        instruction: null,
        offsetHex: null,
        helpText: null,
        rawBlock: trimmed,
        sourceChannel,
        commandLine: job.commandLine,
        cwd: job.cwd,
      })
    }
  }

  return diagnostics
}

function collectDiagnosticBlock(lines: string[], startIndex: number) {
  const blockLines: string[] = []
  let helpText: string | null = null
  let index = startIndex

  while (index < lines.length) {
    const line = lines[index] ?? ''
    const trimmed = line.trim()

    if (index !== startIndex && isDiagnosticStart(trimmed)) {
      break
    }

    if (!trimmed) {
      blockLines.push(line)
      index += 1
      break
    }

    if (trimmed.startsWith('help:')) {
      helpText = trimmed.slice('help:'.length).trim()
    }

    blockLines.push(line)
    index += 1
  }

  return {
    rawBlock: blockLines.join('\n').trimEnd(),
    helpText,
    nextIndex: index,
  }
}

function isDiagnosticStart(line: string) {
  return (
    sourceDiagnosticPattern.test(line) ||
    verifyDiagnosticPattern.test(line) ||
    runtimeDiagnosticPattern.test(line)
  )
}

function shouldPromoteLine(job: DiagnosticSourceJob, line: string) {
  if (!line) {
    return false
  }

  if (job.kind === 'cargo' || job.kind === 'release_bundle_verify') {
    return false
  }

  return !line.startsWith('help:')
}

function classifySourceFamily(code: string, message: string): DiagnosticFamily {
  const normalizedMessage = message.toLowerCase()

  if (normalizedMessage.startsWith('policy violation:')) {
    return 'policy'
  }

  if (code >= 'E0238' && code <= 'E0245') {
    return 'module'
  }

  if (
    normalizedMessage.includes('unknown variable') ||
    normalizedMessage.includes('unknown function') ||
    normalizedMessage.includes('argument count mismatch') ||
    normalizedMessage.includes('argument type mismatch') ||
    normalizedMessage.includes('let-binding type mismatch') ||
    normalizedMessage.includes('return type mismatch') ||
    normalizedMessage.includes('invalid `if` condition') ||
    normalizedMessage.includes('invalid `match`') ||
    normalizedMessage.includes('unsupported operator') ||
    normalizedMessage.includes('main must have signature')
  ) {
    return 'type'
  }

  return 'parse'
}

function inferJobPrimaryPath(job: DiagnosticSourceJob) {
  const args = job.resolvedCommand.slice(1)

  for (let index = 0; index < args.length; index += 1) {
    const value = args[index]
    const previous = args[index - 1]
    if (!value || value === '--') {
      continue
    }

    if (
      previous === '-o' ||
      previous === '--bin' ||
      previous === '--manifest-path' ||
      previous === '-ManifestPath' ||
      previous === '-File'
    ) {
      continue
    }

    if (job.kind === 'smc' && value.toLowerCase().endsWith('.sm')) {
      return value
    }

    if (job.kind === 'svm' && value.toLowerCase().endsWith('.smc')) {
      return value
    }
  }

  return null
}

function extractInlineCode(line: string) {
  const match = /\b([A-Z]\d{4})\b/.exec(line)
  return match?.[1] ?? null
}

function parseNumber(value?: string | null) {
  if (!value) {
    return null
  }

  const parsed = Number.parseInt(value, 10)
  return Number.isNaN(parsed) ? null : parsed
}

function stripAnsi(text: string) {
  return text.replace(ansiEscapePattern, '')
}
