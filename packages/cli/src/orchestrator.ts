import { PackageInfo } from './resolver'
import chalk from 'chalk'
import { runSandboxed } from './sandboxer'
import { SandboxEvent } from '@guardinstall/policy-engine'
import { loadConfig, isAllowed, isDenied, GuardInstallConfig } from './config'
import * as os from 'os'
import * as fs from 'fs'
import * as path from 'path'

const isWindows = os.platform() === 'win32'

export interface SandboxResult {
  package: string;
  blocked: boolean;
  events: SandboxEvent[];
  skipped?: boolean;
  reason?: string;
}

export async function runSandbox(
  packages: PackageInfo[],
  concurrency: number = 4,
): Promise<SandboxResult[]> {
  const config = loadConfig()
  const effectiveConcurrency = config.concurrency || concurrency
  console.log(chalk.blue(`\n🔒 Running sandbox for ${packages.length} packages (concurrency: ${effectiveConcurrency})...\n`))

  const results: SandboxResult[] = []
  const chunks: PackageInfo[][] = []

  for (let i = 0; i < packages.length; i += effectiveConcurrency) {
    chunks.push(packages.slice(i, i + effectiveConcurrency))
  }

  // Filter by denylist/allowlist
  const filtered: PackageInfo[] = []
  for (const pkg of packages) {
    if (isDenied(pkg.name, config)) {
      results.push({ package: pkg.name, blocked: true, events: [], skipped: true, reason: 'denied by guardinstall.json' })
      continue
    }
    if (isAllowed(pkg.name, config)) {
      results.push({ package: pkg.name, blocked: false, events: [], skipped: true, reason: 'allowlisted in guardinstall.json' })
      continue
    }
    filtered.push(pkg)
  }

  for (let i = 0; i < filtered.length; i += effectiveConcurrency) {
    const chunk = filtered.slice(i, i + effectiveConcurrency)
    const chunkResults = await Promise.all(
      chunk.map(pkg => sandboxPackage(pkg))
    )
    results.push(...chunkResults)
  }

  return results
}

function writeScriptFile(pkg: PackageInfo): { filePath: string; cleanup: () => void } {
  const scriptCommand = pkg.scripts?.postinstall || pkg.scripts?.install || pkg.scripts?.preinstall

  if (isWindows) {
    const tmpFile = path.join(os.tmpdir(), `guardinstall-${pkg.name}-${Date.now()}.bat`)
    const scriptContent = `@echo off\ncd /d "${pkg.path}"\n${scriptCommand}\n`
    fs.writeFileSync(tmpFile, scriptContent)
    return {
      filePath: tmpFile,
      cleanup: () => { try { fs.unlinkSync(tmpFile) } catch {} }
    }
  }

  const tmpFile = path.join(os.tmpdir(), `guardinstall-${pkg.name}-${Date.now()}.sh`)
  const scriptContent = `#!/bin/sh\ncd "${pkg.path}"\n${scriptCommand}\n`
  fs.writeFileSync(tmpFile, scriptContent, { mode: 0o700 })
  return {
    filePath: tmpFile,
    cleanup: () => { try { fs.unlinkSync(tmpFile) } catch {} }
  }
}

export async function sandboxPackage(pkg: PackageInfo): Promise<SandboxResult> {
  console.log(chalk.gray(`  Sandboxing ${pkg.name}@${pkg.version}...`))

  const scriptCommand = pkg.scripts?.postinstall || pkg.scripts?.install || pkg.scripts?.preinstall

  if (!scriptCommand) {
    return {
      package: pkg.name,
      blocked: false,
      events: []
    }
  }

  const { filePath, cleanup } = writeScriptFile(pkg)

  try {
    const result = runSandboxed(filePath, pkg.name)

    return {
      package: pkg.name,
      blocked: result.blocked,
      events: result.events,
    }
  } finally {
    cleanup()
  }
}
