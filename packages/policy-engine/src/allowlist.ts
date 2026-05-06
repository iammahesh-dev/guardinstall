import * as fs from 'fs'
import * as path from 'path'
import semver from 'semver'

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
  event: unknown
): boolean {
  if (!semver.satisfies(version, profile.versions)) {
    return false
  }

  return profile.maintainers_verified
}
