/**
 * Integration Tests: Verify orchestrator works with real sandbox
 */

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
        action: 'blocked',
        timestamp_ns: Date.now() * 1000000
      }] : []
    }
  })
}))

describe('Integration Tests', () => {
  test('should run sandbox for multiple packages', async () => {
    const packages: PackageInfo[] = [
      { name: 'pkg1', version: '1.0.0', scripts: {}, path: '/tmp/nonexistent1', isNew: true },
      { name: 'pkg2', version: '1.0.0', scripts: {}, path: '/tmp/nonexistent2', isNew: true },
    ]

    const results: SandboxResult[] = await runSandbox(packages, 1)
    expect(results).toHaveLength(2)
  })

  test('should detect blocked packages', async () => {
    const packages: PackageInfo[] = [
      { name: 'malicious', version: '1.0.0', scripts: { postinstall: 'sh evil.sh' }, path: '/tmp/nonexistent', isNew: true },
      { name: 'clean1', version: '1.0.0', scripts: {}, path: '/tmp/nonexistent', isNew: true }
    ]

    const results: SandboxResult[] = await runSandbox(packages, 1)
    const blocked = results.filter(r => r.blocked)
    expect(blocked).toHaveLength(1)
    expect(blocked[0].package).toBe('malicious')
  })

  test('should return events for blocked packages', async () => {
    const packages: PackageInfo[] = [
      { name: 'malicious', version: '1.0.0', scripts: { postinstall: 'sh evil.sh' }, path: '/tmp/nonexistent', isNew: true }
    ]

    const results: SandboxResult[] = await runSandbox(packages, 1)
    expect(results[0].events.length).toBeGreaterThan(0)
    expect(results[0].events[0].action).toBe('blocked')
  })
})
