#!/usr/bin/env node
import { Command } from 'commander'
import chalk from 'chalk'
import { getInstallScripts, scanNodeModulesForScripts, listPackageNames } from './resolver'
import { runSandbox, SandboxResult } from './orchestrator'
import { evaluateEvents, Verdict } from '@guardinstall/policy-engine'
import { printReport } from './reporter'
import { promptUser, promptCI } from './prompt'
import { runPackageManager, getGlobalNodeModulesPath } from './installer'
import { readFileSync } from 'fs'
import path from 'path'

const version = JSON.parse(readFileSync(path.join(__dirname, '..', 'package.json'), 'utf-8')).version

const program = new Command()

program
  .name('guardinstall')
  .description('A kernel-level behavioral sandbox for npm/pnpm/bun install scripts')
  .version(version)

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
      // Step 1: Install without running scripts (--ignore-scripts)
      console.log(chalk.blue('📦 Installing packages without running scripts...\n'))
      const installResult = await runPackageManager(options.pm, ['install', '--ignore-scripts'])
      if (!installResult.success) {
        console.error(chalk.red('Install failed'))
        process.exit(1)
      }

      // Step 2: Find packages with install scripts
      const packages = await getInstallScripts(process.cwd())
      console.log(chalk.green(`\nFound ${packages.length} packages with install scripts:\n`))

      packages.forEach(pkg => {
        console.log(chalk.yellow(`  ${pkg.name}@${pkg.version}`))
        if (pkg.scripts) {
          Object.entries(pkg.scripts).forEach(([script, content]) => {
            console.log(chalk.gray(`    ${script}: ${content}`))
          })
        }
      })

      // Step 3: Run install scripts through sandbox
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
    } catch (error) {
      console.error(chalk.red('Error:'), error instanceof Error ? error.message : error)
      process.exit(1)
    }
  })

program
  .command('add <packages...>')
  .description('Add packages with sandbox protection')
  .option('--pm <package-manager>', 'package manager to use (npm, pnpm, bun)', 'npm')
  .option('-g, --global', 'install globally', false)
  .action(async (packages, options) => {
    console.log(chalk.blue.bold('\n🔒 guardinstall — Adding packages with protection\n'))
    console.log(chalk.gray(`Packages: ${packages.join(', ')}`))
    console.log(chalk.gray(`Using: ${options.pm}${options.global ? ' (global)' : ''}\n`))

    if (options.global) {
      const globalNodeModules = getGlobalNodeModulesPath(options.pm)
      console.log(chalk.gray(`Global path: ${globalNodeModules}\n`))

      const before = listPackageNames(globalNodeModules)

      const installResult = await runPackageManager(options.pm, ['add', '-g', '--ignore-scripts', ...packages])
      if (!installResult.success) {
        console.error(chalk.red('Global add failed'))
        process.exit(1)
      }

      const after = scanNodeModulesForScripts(globalNodeModules)
      const newPackages = after.filter(p => !before.has(p.name))

      if (newPackages.length > 0) {
        console.log(chalk.blue(`\n🔍 Sandboxing ${newPackages.length} new global package(s)...\n`))
        const results = await runSandbox(newPackages)
        const verdicts: Verdict[] = results
          .filter(r => r.events.length > 0)
          .map(r => evaluateEvents(r.events, r.package))

        printReport(results, verdicts)

        const blocked = verdicts.filter(v => v.severity === 'CRITICAL')
        if (blocked.length > 0) {
          console.log(chalk.red('\n❌ Critical security issues found!'))
          await promptUser(blocked[0])
        }
      } else {
        console.log(chalk.green('\n✓ No install scripts found in new global packages'))
      }
    } else {
      // Step 1: Install without running scripts (--ignore-scripts)
      const installResult = await runPackageManager(options.pm, ['add', '--ignore-scripts', ...packages])
      if (!installResult.success) {
        console.error(chalk.red('Add failed'))
        process.exit(1)
      }

      // Step 2: Find new packages with install scripts
      const newPackages = (await getInstallScripts(process.cwd())).filter(p => p.isNew)
      if (newPackages.length > 0) {
        console.log(chalk.blue(`\n🔍 Sandboxing ${newPackages.length} new package(s)...\n`))

        // Step 3: Run install scripts through sandbox
        const results = await runSandbox(newPackages)
        const verdicts: Verdict[] = results
          .filter(r => r.events.length > 0)
          .map(r => evaluateEvents(r.events, r.package))

        printReport(results, verdicts)

        // Step 4: Check for critical issues
        const blocked = verdicts.filter(v => v.severity === 'CRITICAL')
        if (blocked.length > 0) {
          console.log(chalk.red('\n❌ Critical security issues found!'))
          await promptUser(blocked[0])
        }
      } else {
        console.log(chalk.green('\n✓ No install scripts found in new packages'))
      }
    }
  })

program
  .command('audit')
  .description('Scan existing node_modules for install script behavior')
  .option('--json', 'Output in JSON format')
  .action(async (options) => {
    const { auditExisting } = await import('./audit')
    await auditExisting(options.json)
  })

program.parse()
