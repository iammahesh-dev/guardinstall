import { PackageInfo } from './resolver'
import chalk from 'chalk'
import { sandboxProcess } from '@guardinstall/sandbox'

export interface SandboxResult {
  package: string
  blocked: boolean
  events: SandboxEvent[]
}

export interface SandboxEvent {
  event: string
  package: string
  syscall?: string
  args?: unknown
  path?: string
  action: 'blocked' | 'allowed' | 'logged'
  timestamp_ns: number
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

  // Run the install script in Rust sandbox
  // The sandbox will apply seccomp-BPF, namespaces, Landlock
  // and emit events via stdout (JSON lines)
  const scriptPath = `${pkg.path}/postinstall.sh`  // or the actual script path

  const events: SandboxEvent[] = []

  try {
    // Call Rust sandbox via napi
    const result = sandboxProcess(scriptPath)

    // Parse events from Rust stdout (JSON lines)
    // For now, just return basic result
    return {
      package: pkg.name,
      blocked: false,  // TODO: determine from events
      events
    }
  } catch (error: any) {
    // Sandbox blocked the script or it failed
    events.push({
      event: 'script_blocked',
      package: `${pkg.name}@${pkg.version}`,
      syscall: undefined,
      args: error.message,
      path: undefined,
      action: 'blocked',
      timestamp_ns: Date.now() * 1000000
    })

    return {
      package: pkg.name,
      blocked: true,
      events
    }
  }
}
