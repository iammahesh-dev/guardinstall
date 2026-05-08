import { loadPolicy, isBehaviorAllowed } from './allowlist'
import * as fs from 'fs'

jest.mock('fs')

describe('Allowlist', () => {
  it('should load policy for known package', () => {
    const mockPolicy = {
      package: 'esbuild',
      versions: '>=0.18.0',
      maintainers_verified: true,
      expected_behavior: {
        network: {
          allowed_hosts: ['registry.npmjs.org'],
          reason: 'Downloads binary'
        },
        filesystem: {
          writes: ['./bin/esbuild'],
          reason: 'Writes binary'
        },
        exec: false
      }
    }

    jest.spyOn(fs, 'existsSync').mockReturnValue(true)
    jest.spyOn(fs, 'readFileSync').mockReturnValue(JSON.stringify(mockPolicy))

    const policy = loadPolicy('esbuild')
    expect(policy).not.toBeNull()
    expect(policy?.package).toBe('esbuild')
  })

  it('should return null for unknown package', () => {
    jest.spyOn(fs, 'existsSync').mockReturnValue(false)
    const policy = loadPolicy('unknown-pkg')
    expect(policy).toBeNull()
  })
})
