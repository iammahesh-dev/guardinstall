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
    match: (e) => e.event === 'script_blocked',
    severity: 'CRITICAL',
    message: 'Malicious behavior detected and blocked'
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
  const addr = extractAddr(args)
  if (!addr) return false;

  // Allow private/loopback ranges
  if (/^(127\.|10\.|192\.168\.|172\.(1[6-9]|2\d|3[01])\.)/.test(addr)) return false
  if (addr === 'localhost' || addr === '::1') return false;

  // Allow known CDN/registry hostnames
  const trustedHosts = ['registry.npmjs.org', 'cdn.jsdelivr.net', 'github.com',
                       'objects.githubusercontent.com', 'dl.google.com']
  if (trustedHosts.some(h => addr.includes(h))) return false;

  return true
}

function extractAddr(args: unknown): string | null {
  if (typeof args === 'string') return args
  if (typeof args === 'object' && args !== null) {
    // Handle { addr: '185.220.101.47:443' }
    if ('addr' in args && typeof (args as any).addr === 'string') {
      const addr = (args as any).addr
      // Remove port if present
      return addr.split(':')[0]
    }
    // Handle array of strings
    if (Array.isArray(args)) {
      const found = args.find(arg => typeof arg === 'string' && /https?:\/\//.test(arg))
      if (found && typeof found === 'string') {
        try {
          return new URL(found).hostname
        } catch {
          return found
        }
      }
    }
  }
  return null
}

function extractAddress(args: unknown): string | null {
  if (typeof args === 'string') return args
  if (Array.isArray(args)) {
    const found = args.find(arg => typeof arg === 'string' && /https?:\/\//.test(arg))
    if (found && typeof found === 'string') {
      try {
        return new URL(found).hostname
      } catch {
        return found
      }
    }
  }
  return null
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
