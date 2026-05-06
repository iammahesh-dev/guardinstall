import { getInstallScripts } from './resolver'
import { runSandbox, SandboxResult } from './orchestrator'
import { evaluateEvents, Verdict } from '@guardinstall/policy-engine'
import { printReport } from './reporter'
import chalk from 'chalk'

export async function auditExisting(projectRoot: string = process.cwd()): Promise<void> {
  console.log(chalk.blue.bold('\n🔒 guardinstall — Audit Mode\n'))

  try {
    const packages = await getInstallScripts(projectRoot)
    console.log(chalk.green(`Found ${packages.length} packages with install scripts to audit\n`))

    if (packages.length === 0) {
      console.log(chalk.green('✓ No packages with install scripts found\n'))
      return
    }

    const results = await runSandbox(packages)
    const verdicts: Verdict[] = results
      .filter(r => r.events.length > 0)
      .map(r => evaluateEvents(r.events, r.package))

    printReport(results, verdicts)

    const critical = verdicts.filter(v => v.severity === 'CRITICAL')
    if (critical.length > 0) {
      console.log(chalk.red(`\n❌ Audit failed: ${critical.length} critical issues found\n`))
      process.exit(1)
    } else {
      console.log(chalk.green('\n✓ Audit passed\n'))
    }
  } catch (error) {
    console.error(chalk.red('Error:'), error instanceof Error ? error.message : error)
    process.exit(1)
  }
}
