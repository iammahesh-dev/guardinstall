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
  } catch {
    tree = await arb.loadVirtual()
  }

  const packagesWithScripts: PackageInfo[] = []
  const lockfile = readLockfile(projectRoot)

  for (const node of tree.inventory.values()) {
    const scripts = node.package?.scripts ?? {}
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
