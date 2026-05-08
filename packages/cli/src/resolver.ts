import Arborist from '@npmcli/arborist'
import path from 'path'
import fs from 'fs'
import { readLockfile, isInLockfile } from './lockfile'

export interface PackageInfo {
  name: string
  version: string
  scripts?: Record<string, string>
  path: string
  isNew: boolean
}

export async function getInstallScripts(projectRoot: string): Promise<PackageInfo[]> {
  const arb = new Arborist({ path: projectRoot })

  let tree
  try {
    tree = await arb.loadActual()
  } catch (e) {
    console.error('Warning: Error loading package tree:', e instanceof Error ? e.message : e)
    return getInstallScriptsFromNodeModules(projectRoot) // Fallback to manual scan
  }

  const packagesWithScripts: PackageInfo[] = []
  const lockfile = readLockfile(projectRoot)

  try {
    for (const node of tree.inventory.values()) {
      const pkg = node.package
      if (!pkg) continue
      
      const scripts = pkg.scripts ?? {}
      const hasInstallScript = ['preinstall', 'install', 'postinstall']
        .some(s => s in scripts)

      if (hasInstallScript) {
        const installScripts = pick(scripts, ['preinstall', 'install', 'postinstall'])
        packagesWithScripts.push({
          name: node.name,
          version: node.version,
          scripts: installScripts as Record<string, string> | undefined,
          path: node.path,
          isNew: !isInLockfile(node.name, node.version, lockfile)
        })
      }
    }
  } catch (e) {
    console.error('Warning: Error iterating package tree:', e instanceof Error ? e.message : e)
    return getInstallScriptsFromNodeModules(projectRoot)
  }

  return packagesWithScripts
}

// Fallback: manually scan node_modules for package.json files
function getInstallScriptsFromNodeModules(projectRoot: string): PackageInfo[] {
  const packagesWithScripts: PackageInfo[] = []
  const lockfile = readLockfile(projectRoot)
  
  try {
    const nodeModulesPath = path.join(projectRoot, 'node_modules')
    if (!fs.existsSync(nodeModulesPath)) return packagesWithScripts
    
    const scanDir = (dir: string, depth: number = 0) => {
      if (depth > 2) return // Limit recursion depth
      
      try {
        const entries = fs.readdirSync(dir, { withFileTypes: true })
        
        for (const entry of entries) {
          if (entry.isDirectory()) {
            if (entry.name.startsWith('@')) {
              // Scoped package directory
              scanDir(path.join(dir, entry.name), depth + 1)
            } else if (depth === 0 || depth === 1) {
              // Check package.json in this directory
              const pkgPath = path.join(dir, entry.name, 'package.json')
              if (fs.existsSync(pkgPath)) {
                try {
                  const pkgJson = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'))
                  if (pkgJson.scripts && ('preinstall' in pkgJson.scripts || 'install' in pkgJson.scripts || 'postinstall' in pkgJson.scripts)) {
                    packagesWithScripts.push({
                      name: pkgJson.name || entry.name,
                      version: pkgJson.version || 'unknown',
                      scripts: pick(pkgJson.scripts, ['preinstall', 'install', 'postinstall']) as Record<string, string> | undefined,
                      path: path.join(dir, entry.name),
                      isNew: !isInLockfile(pkgJson.name || entry.name, pkgJson.version || 'unknown', lockfile)
                    })
                  }
                } catch (e) {
                  // Ignore parse errors
                }
              }
            }
          }
        }
      } catch (e) {
        // Ignore permission errors
      }
    }
    
    scanDir(nodeModulesPath)
  } catch (e) {
    // Ignore errors
  }
  
  return packagesWithScripts
}

function pick<T extends Record<string, unknown>>(obj: T, keys: string[]): Partial<T> {
  const result: Partial<T> = {}
  for (const key of keys) {
    if (key in obj) {
      result[key as keyof T] = obj[key] as T[keyof T]
    }
  }
  return result
}
