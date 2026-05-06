import { SandboxEvent, ScoringRule, Severity } from './types'

export const SCORING_RULES: ScoringRule[] = [
  {
    match: (e) => e.syscall === 'execve' && isCurlOrWget(e.args),
    severity: 'CRITICAL',
    message: 'Attempted to download and execute remote code'
  },
  {
    match: (e) => e.syscall === 'connect' && isExternalIP(e.args),
    severity: 'HIGH',
    message: 'Attempted outbound network connection during install'
  },
  {
    match: (e) => e.event === 'fs_write_attempt' && isSensitivePath(e.path),
    severity: 'CRITICAL',
    message: 'Attempted to write to sensitive path'
  },
  {
    match: (e) => e.event === 'fs_write_attempt' && !isPackagePath(e.path),
    severity: 'WARN',
    message: 'Wrote outside package directory'
  }
]

function isCurlOrWget(args: unknown): boolean {
  if (!args) return false

  const searchString = (s: string) => s.includes('curl') || s.includes('wget') || s.includes('http')

  if (typeof args === 'string') return searchString(args)
  if (Array.isArray(args)) {
    return args.some(arg => {
      if (typeof arg === 'string') return searchString(arg)
      if (Array.isArray(arg)) return arg.some(a => typeof a === 'string' && searchString(a))
      return false
    })
  }
  return false
}

function isExternalIP(args: unknown): boolean {
  return true
}

function isSensitivePath(path?: string): boolean {
  if (!path) return false
  const sensitive = ['~/.ssh', '~/.aws', '.env', '.npmrc']
  return sensitive.some(s => path.includes(s))
}

function isPackagePath(path?: string): boolean {
  if (!path) return false
  return path.includes('node_modules') || path.includes('/tmp')
}
