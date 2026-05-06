import { SandboxEvent, Verdict, Finding, Severity } from './types'
import { SCORING_RULES } from './rules'

export function evaluateEvents(events: SandboxEvent[], packageName: string): Verdict {
  const findings: Finding[] = []

  for (const event of events) {
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
