import { runSandbox, SandboxResult } from '../orchestrator'
import { PackageInfo } from '../resolver'

// Mock sandboxer to return controlled results
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
        args: ['/bin/sh'],
        path: undefined,
        action: 'blocked' as const,
        timestamp_ns: Date.now() * 1000000
      }] : []
    }
  })
}))

describe('Orchestrator', () => {
  it('should run sandbox for multiple packages', async () => {
    const packages: PackageInfo[] = [
      { name: 'legit-pkg', version: '1.0.0', path: '/test', isNew: true, scripts: {} },
      { name: 'malicious-pkg', version: '1.0.0', path: '/test', isNew: true, scripts: { postinstall: 'evil' } }
    ]

    const results: SandboxResult[] = await runSandbox(packages, 2)
    expect(results).toHaveLength(2)
    expect(results[0].package).toBe('legit-pkg')
    expect(results[1].package).toBe('malicious-pkg')
  })

  it('should detect blocked packages', async () => {
    const packages: PackageInfo[] = [
      { name: 'malicious-pkg', version: '1.0.0', path: '/test', isNew: true, scripts: { postinstall: 'evil' } }
    ]

    const results: SandboxResult[] = await runSandbox(packages)
    const blocked = results.filter(r => r.blocked)
    expect(blocked).toHaveLength(1)
    expect(blocked[0].package).toBe('malicious-pkg')
  })
})
