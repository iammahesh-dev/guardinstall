import chalk from 'chalk'
import { SandboxResult } from './orchestrator'
import { Verdict, Finding, Severity } from '@guardinstall/policy-engine'

export function printReport(results: SandboxResult[], verdicts: Verdict[]): void {
  console.log(chalk.blue.bold('\n═══════════════════════════════════════'))
  console.log(chalk.blue.bold('  guardinstall — Security Report'))
  console.log(chalk.blue.bold('═══════════════════════════════════════\n'))

  const blocked = verdicts.filter(v => v.severity === 'CRITICAL' || v.severity === 'HIGH')
  const warned = verdicts.filter(v => v.severity === 'WARN')
  const passed = verdicts.filter(v => v.severity === 'INFO')
  const clean = results.filter(r => r.events.length === 0)

  if (blocked.length > 0) {
    console.log(chalk.red.bold(`\n🚨 BLOCKED (${blocked.length} packages):\n`))
    blocked.forEach((v: Verdict) => {
      console.log(chalk.red(`  ⚠  ${v.package}`))
      v.findings.forEach((f: Finding) => {
        console.log(chalk.red(`     [${f.severity}] ${f.message}`))
      })
    })
  }

  if (warned.length > 0) {
    console.log(chalk.yellow.bold(`\n⚠️  WARNINGS (${warned.length} packages):\n`))
    warned.forEach((v: Verdict) => {
      console.log(chalk.yellow(`  ${v.package}`))
      v.findings.forEach((f: Finding) => {
        console.log(chalk.yellow(`     [${f.severity}] ${f.message}`))
      })
    })
  }

  if (passed.length > 0) {
    console.log(chalk.green.bold(`\n✓ PASSED (${passed.length} packages - no malicious behavior):\n`))
    passed.forEach((v: Verdict) => {
      console.log(chalk.green(`  ✓ ${v.package}`))
    })
  }

  if (clean.length > 0) {
    console.log(chalk.gray.bold(`\nℹ  NO SCRIPTS (${clean.length} packages):\n`))
    clean.forEach((r: SandboxResult) => {
      console.log(chalk.gray(`  - ${r.package}`))
    })
  }

  console.log(chalk.blue.bold('\n═══════════════════════════════════════\n'))
}
