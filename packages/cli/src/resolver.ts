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
  const nodeModulesPath = path.join(projectRoot, 'node_modules')
  const lockfile = readLockfile(projectRoot)
  const packages = scanNodeModulesForScripts(nodeModulesPath)

  for (const pkg of packages) {
    pkg.isNew = !isInLockfile(pkg.name, pkg.version, lockfile)
  }

  return packages
}

export function scanNodeModulesForScripts(nodeModulesPath: string): PackageInfo[] {
  const packagesWithScripts: PackageInfo[] = []
  if (!fs.existsSync(nodeModulesPath)) return packagesWithScripts

  const scanDir = (dir: string, depth: number = 0) => {
    if (depth > 2) return

    try {
      const entries = fs.readdirSync(dir, { withFileTypes: true })

      for (const entry of entries) {
        if (!entry.isDirectory()) continue
        if (entry.name.startsWith('@')) {
          scanDir(path.join(dir, entry.name), depth + 1)
        } else if (depth === 0 || depth === 1) {
          const pkgPath = path.join(dir, entry.name, 'package.json')
          if (!fs.existsSync(pkgPath)) continue
          try {
            const pkgJson = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'))
            const scripts = pkgJson.scripts || {}
            if (scripts.preinstall || scripts.install || scripts.postinstall) {
              packagesWithScripts.push({
                name: pkgJson.name || entry.name,
                version: pkgJson.version || 'unknown',
                scripts: pick(scripts, ['preinstall', 'install', 'postinstall']) as Record<string, string> | undefined,
                path: path.join(dir, entry.name),
                isNew: true,
              })
            }
          } catch {
            // Ignore parse errors
          }
        }
      }
    } catch {
      // Ignore permission errors
    }
  }

  scanDir(nodeModulesPath)
  return packagesWithScripts
}

export function listPackageNames(nodeModulesPath: string): Set<string> {
  const names = new Set<string>()
  if (!fs.existsSync(nodeModulesPath)) return names

  try {
    const entries = fs.readdirSync(nodeModulesPath, { withFileTypes: true })
    for (const entry of entries) {
      if (!entry.isDirectory()) continue
      if (entry.name.startsWith('@')) {
        try {
          const scoped = fs.readdirSync(path.join(nodeModulesPath, entry.name))
          for (const s of scoped) names.add(`${entry.name}/${s}`)
        } catch {
          // Ignore permission errors
        }
      } else {
        names.add(entry.name)
      }
    }
  } catch {
    // Ignore permission errors
  }

  return names
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
