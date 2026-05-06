/**
 * Integration Tests: Verify orchestrator works with real sandbox
 */

import { runSandbox, SandboxResult } from '../orchestrator'
import { PackageInfo } from '../resolver'

describe('Integration Tests', () => {
  test('should run sandbox for multiple packages', async () => {
    const packages: PackageInfo[] = [
      { name: 'pkg1', version: '1.0.0', scripts: {}, path: '/tmp/nonexistent1', isNew: true },
      { name: 'pkg2', version: '1.0.0', scripts: {}, path: '/tmp/nonexistent2', isNew: true },
    ]

    const results = await runSandbox(packages, 1)
    expect(results).toHaveLength(2)
    // All should be "blocked" because scripts don't exist
    results.forEach(r => {
      expect(r.blocked).toBe(true)
    })
  })

  test('should handle missing scripts gracefully', async () => {
    const packages: PackageInfo[] = [
      { name: 'malicious', version: '1.0.0', scripts: { postinstall: 'sh evil.sh' }, path: '/tmp/nonexistent', isNew: true },
    ]

    const results = await runSandbox(packages, 1)
    expect(results).toHaveLength(1)
    expect(results[0].blocked).toBe(true)
    expect(results[0].events.length).toBeGreaterThan(0)
  })

  test('should return events for blocked packages', async () => {
    const packages: PackageInfo[] = [
      { name: 'pkg1', version: '1.0.0', scripts: {}, path: '/tmp/nonexistent', isNew: true },
    ]

    const results = await runSandbox(packages, 1)
    const blocked = results.filter(r => r.blocked)
    expect(blocked.length).toBeGreaterThan(0)

    if (blocked.length > 0) {
      expect(blocked[0].events).toBeDefined()
    }
  })
})
