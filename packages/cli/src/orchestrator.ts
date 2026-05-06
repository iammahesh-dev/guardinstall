import { PackageInfo } from './resolver'
import chalk from 'chalk'
import { runSandboxed } from './sandboxer'
import { SandboxEvent } from '@guardinstall/policy-engine'

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

  // Determine script path
  const scriptPath = `${pkg.path}/postinstall.sh`

  // Run script in sandboxed environment using Rust binary
  const result = runSandboxed(scriptPath, pkg.name)

  return {
    package: pkg.name,
    blocked: result.blocked,
    events: result.events,
  }
}
