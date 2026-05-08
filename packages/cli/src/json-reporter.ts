import { SandboxResult } from './orchestrator'
import { Verdict } from '@guardinstall/policy-engine'

export interface JsonReport {
  timestamp: string
  packages_scanned: number
  packages_blocked: number
  verdicts: Array<{
    package: string
    severity: string
    findings_count: number
  }>
}

export function generateJsonReport(
  results: SandboxResult[],
  verdicts: Verdict[]
): JsonReport {
  const blocked = results.filter(r => r.blocked)

  return {
    timestamp: new Date().toISOString(),
    packages_scanned: results.length,
    packages_blocked: blocked.length,
    verdicts: verdicts.map(v => ({
      package: v.package,
      severity: v.severity,
      findings_count: v.findings.length
    }))
  }
}
