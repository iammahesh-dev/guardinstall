import { SandboxEvent, Verdict, Finding, Severity } from './types'
import { SCORING_RULES } from './rules'
import { loadPolicy, isBehaviorAllowed } from './allowlist'
import semver from 'semver'

export function evaluateEvents(events: SandboxEvent[], packageName: string, version?: string): Verdict {
  const findings: Finding[] = []
  const profile = loadPolicy(packageName)

  for (const event of events) {
    // Skip events covered by the allowlist profile
    if (profile && version && isBehaviorAllowed(profile, version, event)) {
      continue
    }

    for (const rule of SCORING_RULES) {
      if (rule.match(event)) {
        findings.push({
          severity: rule.severity,
          message: rule.message,
          event
        })
      }
    }
  }

  const severityOrder: Severity[] = ['CRITICAL', 'HIGH', 'WARN', 'INFO']
  const worstSeverity = findings.reduce<Severity>((worst, f) => {
    return severityOrder.indexOf(f.severity) < severityOrder.indexOf(worst) ? f.severity : worst
  }, 'INFO')

  return {
    package: packageName,
    events,
    findings,
    severity: worstSeverity
  }
}

// Export for backward compatibility
export { loadPolicy, isBehaviorAllowed }
