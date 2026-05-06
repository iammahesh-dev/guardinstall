#!/usr/bin/env node
import { Command } from 'commander'
import chalk from 'chalk'
import { getInstallScripts } from './resolver'
import { runSandbox, SandboxResult } from './orchestrator'
import { evaluateEvents, Verdict } from '@guardinstall/policy-engine'
import { printReport } from './reporter'
import { promptUser, promptCI } from './prompt'
import { runPackageManager } from './installer'

const program = new Command()

program
  .name('guardinstall')
  .description('A kernel-level behavioral sandbox for npm/pnpm/bun install scripts')
  .version('0.0.1')

program
  .command('install')
  .description('Run npm install with sandbox protection')
  .option('--pm <package-manager>', 'package manager to use (npm, pnpm, bun)', 'npm')
  .option('--ci', 'CI mode: fail instead of prompt')
  .action(async (options) => {
    console.log(chalk.blue.bold('\n🔒 guardinstall — Behavioral Sandbox for Install Scripts\n'))
    console.log(chalk.gray(`Using package manager: ${options.pm}`))
    console.log(chalk.gray(`CI mode: ${options.ci ? 'enabled' : 'disabled'}\n`))

    try {
      const packages = await getInstallScripts(process.cwd())
      console.log(chalk.green(`Found ${packages.length} packages with install scripts:\n`))

      packages.forEach(pkg => {
        console.log(chalk.yellow(`  ${pkg.name}@${pkg.version}`))
        if (pkg.scripts) {
          Object.entries(pkg.scripts).forEach(([script, content]) => {
            console.log(chalk.gray(`    ${script}: ${content}`))
          })
        }
      })

      if (packages.length > 0) {
        const results = await runSandbox(packages)
        const verdicts: Verdict[] = results
          .filter(r => r.events.length > 0)
          .map(r => evaluateEvents(r.events, r.package))

        printReport(results, verdicts)

        if (options.ci) {
          const passed = promptCI(verdicts)
          if (!passed) {
            console.log(chalk.red('\n❌ CI build failed due to security issues\n'))
            process.exit(1)
          }
        } else if (verdicts.some(v => v.severity === 'CRITICAL')) {
          for (const verdict of verdicts) {
            if (verdict.severity === 'CRITICAL') {
              await promptUser(verdict)
            }
          }
        }
      }

      // Run the actual install
      console.log(chalk.blue('\n📦 Running actual install...\n'))
      const result = await runPackageManager(options.pm, ['install'])
      if (!result.success) {
        console.error(chalk.red('Install failed'))
        process.exit(1)
      }
    } catch (error) {
      console.error(chalk.red('Error:'), error instanceof Error ? error.message : error)
      process.exit(1)
    }
  })

program
  .command('add <packages...>')
  .description('Add packages with sandbox protection')
  .option('--pm <package-manager>', 'package manager to use (npm, pnpm, bun)', 'npm')
  .action(async (packages, options) => {
    console.log(chalk.blue.bold('\n🔒 guardinstall — Adding packages with protection\n'))
    console.log(chalk.gray(`Packages: ${packages.join(', ')}`))
    console.log(chalk.gray(`Using: ${options.pm}\n`))

    // Run the actual add command
    const result = await runPackageManager(options.pm, ['add', ...packages])
    if (!result.success) {
      console.error(chalk.red('Add failed'))
      process.exit(1)
    }

    // Then audit the new packages
    const { auditExisting } = await import('./audit')
    await auditExisting()
  })

program
  .command('audit')
  .description('Scan existing node_modules for install script behavior')
  .action(async () => {
    const { auditExisting } = await import('./audit')
    await auditExisting()
  })

program.parse()
