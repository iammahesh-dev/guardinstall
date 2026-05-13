import fs from 'fs'
import path from 'path'

export interface GuardInstallConfig {
  allowlist?: string[]
  denylist?: string[]
  concurrency?: number
  timeout?: number
  ci?: {
    fail_on?: 'critical' | 'high' | 'warn'
    mode?: 'strict' | 'standard'
  }
  pm?: 'npm' | 'pnpm' | 'bun' | 'yarn'
}

const DEFAULTS: GuardInstallConfig = {
  concurrency: 4,
  timeout: 30000,
  ci: { fail_on: 'critical', mode: 'standard' },
}

export function loadConfig(cwd: string = process.cwd()): GuardInstallConfig {
  const candidates = [
    path.join(cwd, 'guardinstall.json'),
    path.join(cwd, '.guardinstallrc'),
    path.join(cwd, '.guardinstallrc.json'),
  ]

  for (const file of candidates) {
    if (!fs.existsSync(file)) continue
    try {
      const user = JSON.parse(fs.readFileSync(file, 'utf-8'))
      return { ...DEFAULTS, ...user }
    } catch {
      console.warn(`Warning: Could not parse ${file}`)
    }
  }

  return { ...DEFAULTS }
}

export function isAllowed(packageName: string, config: GuardInstallConfig): boolean {
  if (config.denylist?.includes(packageName)) return false
  if (config.allowlist?.includes(packageName)) return true
  for (const pattern of config.allowlist || []) {
    if (pattern.endsWith('*') && packageName.startsWith(pattern.slice(0, -1))) return true
  }
  return false
}

export function isDenied(packageName: string, config: GuardInstallConfig): boolean {
  if (config.denylist?.includes(packageName)) return true
  for (const pattern of config.denylist || []) {
    if (pattern.endsWith('*') && packageName.startsWith(pattern.slice(0, -1))) return true
  }
  return false
}
