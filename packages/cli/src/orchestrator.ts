import { PackageInfo } from './resolver'
import chalk from 'chalk'
import { runSandboxed } from './sandboxer'
import { SandboxEvent } from '@guardinstall/policy-engine'
import * as os from 'os'
import * as fs from 'fs'
import * as path from 'path'

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

async function sandboxPackage(pkg: PackageInfo): Promise<SandboxResult> {
  console.log(chalk.gray(`  Sandboxing ${pkg.name}@${pkg.version}...`))

  // Get the actual script command from package.json
  const scriptCommand = pkg.scripts?.postinstall || pkg.scripts?.install || pkg.scripts?.preinstall

  if (!scriptCommand) {
    return {
      package: pkg.name,
      blocked: false,
      events: []
    }
  }

  // Write the script command to a temp shell file
  const tmpFile = path.join(os.tmpdir(), `guardinstall-${pkg.name}-${Date.now()}.sh`)
  const scriptContent = `#!/bin/sh\ncd "${pkg.path}"\n${scriptCommand}\n`

  try {
    fs.writeFileSync(tmpFile, scriptContent, { mode: 0o700 })

    // Run script in sandboxed environment using Rust binary
    const result = runSandboxed(tmpFile, pkg.name)

    return {
      package: pkg.name,
      blocked: result.blocked,
      events: result.events,
    }
  } finally {
    // Clean up temp file
    try {
      fs.unlinkSync(tmpFile)
    } catch {
      // Ignore cleanup errors
    }
  }
}
