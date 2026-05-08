import * as fs from 'fs'
import * as path from 'path'
import semver from 'semver'
import { SandboxEvent } from './types'

export interface PolicyProfile {
  package: string
  versions: string
  maintainers_verified: boolean
  expected_behavior: {
    network?: {
      allowed_hosts: string[]
      reason: string
    }
    filesystem?: {
      writes: string[]
      reason: string
    }
    exec: boolean
  }
}

export function loadPolicy(packageName: string): PolicyProfile | null {
  const policiesDir = path.join(__dirname, '../../policies')
  const policyPath = path.join(policiesDir, `${packageName}.json`)

  if (!fs.existsSync(policyPath)) {
    return null
  }

  try {
    const content = JSON.parse(fs.readFileSync(policyPath, 'utf-8'))
    return content as PolicyProfile
  } catch {
    return null
  }
}

export function isBehaviorAllowed(
  profile: PolicyProfile,
  version: string,
  event: SandboxEvent
): boolean {
  if (!semver.satisfies(version, profile.versions)) {
    return false
  }

  if (!profile.maintainers_verified) return false

  // Check network events against allowed hosts
  if (event.event === 'connect' && profile.expected_behavior.network) {
    const host = extractHost(event.args)
    return profile.expected_behavior.network.allowed_hosts.some(h => host?.includes(h))
  }

  // Check filesystem writes against allowed paths
  if (event.event === 'fs_write_attempt' && event.path && profile.expected_behavior.filesystem) {
    return profile.expected_behavior.filesystem.writes.some(w => event.path?.includes(w))
  }

  return false
}

function extractHost(args: unknown): string | null {
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
