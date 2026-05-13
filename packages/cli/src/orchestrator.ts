import { PackageInfo } from './resolver'
import chalk from 'chalk'
import { runSandboxed } from './sandboxer'
import { SandboxEvent } from '@guardinstall/policy-engine'
import * as os from 'os'
import * as fs from 'fs'
import * as path from 'path'

const isWindows = os.platform() === 'win32'

export interface SandboxResult {
  package: string;
  blocked: boolean;
  events: SandboxEvent[]
}

export async function runSandbox(
  packages: PackageInfo[],
  concurrency: number = 4
): Promise<SandboxResult[]> {
  console.log(chalk.blue(`\n🔒 Running sandbox for ${packages.length} packages (concurrency: ${concurrency})...\n`))

  const results: SandboxResult[] = []
  const chunks: PackageInfo[][] = []

  for (let i = 0; i < packages.length; i += concurrency) {
    chunks.push(packages.slice(i, i + concurrency))
  }

  for (const chunk of chunks) {
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
