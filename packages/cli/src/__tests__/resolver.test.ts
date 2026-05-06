import { getInstallScripts } from '../resolver'
import { readLockfile, isInLockfile } from '../lockfile'
import * as fs from 'fs'
import * as path from 'path'

jest.mock('fs', () => ({
  existsSync: jest.fn(() => false),
  readFileSync: jest.fn(() => '{}')
}))

jest.mock('@npmcli/arborist', () => {
  return jest.fn().mockImplementation(() => ({
    loadActual: jest.fn().mockResolvedValue({
      inventory: new Map([
        ['1', {
          name: 'test-pkg',
          version: '1.0.0',
          path: '/test/node_modules/test-pkg',
          package: {
            scripts: {
              postinstall: 'echo hello'
            }
          }
        }]
      ]),
      package: {}
    }),
    loadVirtual: jest.fn()
  }))
})

describe('Resolver', () => {
  it('should detect packages with install scripts', async () => {
    const packages = await getInstallScripts('/test')
    expect(packages).toBeDefined()
    expect(packages.length).toBeGreaterThan(0)
  })

  it('should identify new packages not in lockfile', () => {
    const lockfile = { 'old-pkg': '1.0.0' }
    expect(isInLockfile('new-pkg', '1.0.0', lockfile)).toBe(false)
    expect(isInLockfile('old-pkg', '1.0.0', lockfile)).toBe(true)
  })
})
