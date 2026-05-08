import { printReport } from './reporter'
import { SandboxResult } from './orchestrator'
import { Verdict, Finding, Severity } from '@guardinstall/policy-engine'

describe('Reporter', () => {
  it('should print report with blocked packages', () => {
    const results: SandboxResult[] = [
      { package: 'malicious-pkg', blocked: true, events: [] }
    ]

    const verdicts: Verdict[] = [
      {
        package: 'malicious-pkg',
        events: [],
        findings: [{
          severity: 'CRITICAL' as Severity,
          message: 'Remote code execution',
          event: {} as any
        }],
        severity: 'CRITICAL' as Severity
      }
    ]

    expect(() => printReport(results, verdicts)).not.toThrow()
  })

  it('should print report with clean packages', () => {
    const results: SandboxResult[] = [
      { package: 'clean-pkg', blocked: false, events: [] }
    ]

    const verdicts: Verdict[] = []

    expect(() => printReport(results, verdicts)).not.toThrow()
  })
})
