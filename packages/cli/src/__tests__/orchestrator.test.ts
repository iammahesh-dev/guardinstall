import { runSandbox, SandboxResult } from '../orchestrator'

// Mock sandboxer to return controlled results
jest.mock('../sandboxer', () => ({
  runSandboxed: jest.fn().mockImplementation((scriptPath: string, packageName: string) => {
    const isMalicious = packageName === 'malicious'
    return {
      package: packageName,
      blocked: isMalicious,
      events: isMalicious ? [{
        event: 'syscall_intercepted',
        package: packageName,
        syscall: 'execve',
        args: ['/bin/sh'],
        path: undefined,
        action: 'blocked',
        timestamp_ns: Date.now() * 1000000
      }] : []
    }
  })
}))

describe('Orchestrator', () => {
  it('should run sandbox for multiple packages', async () => {
    const packages = [
      { name: 'pkg1', version: '1.0.0', path: '/test', isNew: true },
      { name: 'pkg2', version: '1.0.0', path: '/test', isNew: true }
    ]

    const results: SandboxResult[] = await runSandbox(packages, 1)
    expect(results).toHaveLength(2)
  })

  it('should detect blocked packages', async () => {
    const packages = [
      { name: 'malicious', version: '1.0.0', path: '/test', isNew: true },
      { name: 'clean1', version: '1.0.0', path: '/test', isNew: true }
    ]

    const results: SandboxResult[] = await runSandbox(packages, 1)
    const blocked = results.filter(r => r.blocked)
    expect(blocked).toHaveLength(1)
    expect(blocked[0].package).toBe('malicious')
  })
})
