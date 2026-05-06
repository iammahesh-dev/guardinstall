import { runSandbox, SandboxResult } from './orchestrator'
import { PackageInfo } from './resolver'

jest.mock('./resolver')

describe('Orchestrator', () => {
  it('should run sandbox for multiple packages', async () => {
    const packages: PackageInfo[] = [
      { name: 'legit-pkg', version: '1.0.0', path: '/test', isNew: true },
      { name: 'malicious-pkg', version: '1.0.0', path: '/test', isNew: true }
    ]

    const results = await runSandbox(packages, 2)
    expect(results).toHaveLength(2)
    expect(results[0].package).toBe('legit-pkg')
    expect(results[1].package).toBe('malicious-pkg')
  })

  it('should detect blocked packages', async () => {
    const packages: PackageInfo[] = [
      { name: 'malicious-pkg', version: '1.0.0', path: '/test', isNew: true }
    ]

    const results = await runSandbox(packages)
    expect(results[0].blocked).toBe(true)
    expect(results[0].events.length).toBeGreaterThan(0)
  })
})
