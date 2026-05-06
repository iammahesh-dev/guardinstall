import { runSandbox, SandboxResult } from '../orchestrator'

describe('Integration Tests', () => {
  it('should run sandbox for multiple packages with concurrency', async () => {
    const packages = [
      { name: 'pkg1', version: '1.0.0', path: '/test/1', isNew: true },
      { name: 'pkg2', version: '1.0.0', path: '/test/2', isNew: true },
      { name: 'malicious', version: '1.0.0', path: '/test/3', isNew: true }
    ]

    const results = await runSandbox(packages as any, 2)
    expect(results).toHaveLength(3)

    const blocked = results.filter(r => r.blocked)
    expect(blocked).toHaveLength(1)
    expect(blocked[0].package).toBe('malicious')
  })

  it('should return empty events for clean packages', async () => {
    const packages = [
      { name: 'clean1', version: '1.0.0', path: '/test/1', isNew: true },
      { name: 'clean2', version: '1.0.0', path: '/test/2', isNew: true }
    ]

    const results = await runSandbox(packages as any)
    results.forEach(r => {
      expect(r.events).toHaveLength(0)
      expect(r.blocked).toBe(false)
    })
  })

  it('should handle single package', async () => {
    const packages = [
      { name: 'single', version: '1.0.0', path: '/test/1', isNew: true }
    ]

    const results = await runSandbox(packages as any)
    expect(results).toHaveLength(1)
  })
})
