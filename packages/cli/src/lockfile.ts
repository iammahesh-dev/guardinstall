import fs from 'fs'
import path from 'path'

export function readLockfile(projectRoot: string): Record<string, string> {
  const lockfiles = [
    'package-lock.json',
    'pnpm-lock.yaml',
    'yarn.lock',
    'bun.lockb'
  ]

  for (const lockfile of lockfiles) {
    const lockfilePath = path.join(projectRoot, lockfile)
    if (fs.existsSync(lockfilePath)) {
      if (lockfile.endsWith('.json')) {
        return parsePackageLock(lockfilePath)
      }
      return {}
    }
  }

  return {}
}

function parsePackageLock(lockfilePath: string): Record<string, string> {
  try {
    const content = JSON.parse(fs.readFileSync(lockfilePath, 'utf-8'))
    const packages = content.packages ?? content.dependencies ?? {}
    const result: Record<string, string> = {}

    for (const [key, value] of Object.entries(packages)) {
      const name = key.replace(/^node_modules\//, '')
      if (typeof value === 'object' && value !== null && 'version' in value) {
        result[name] = (value as { version: string }).version
      }
    }

    return result
  } catch {
    return {}
  }
}

export function isInLockfile(name: string, version: string, lockfile: Record<string, string>): boolean {
  return lockfile[name] === version
}
