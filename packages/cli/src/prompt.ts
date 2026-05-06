import chalk from 'chalk'
import { Verdict } from '@guardinstall/policy-engine'

export async function promptUser(verdict: Verdict): Promise<'allow' | 'deny' | 'skip'> {
  console.log(chalk.red.bold(`\n⚠  BLOCKED  ${verdict.package}\n`))

  verdict.findings.forEach(f => {
    console.log(chalk.red(`  [${f.severity}] ${f.message}`))
  })

  console.log(chalk.gray(`\nAllow this package to install?`))
  console.log(chalk.gray(`  (a) Allow  (d) Deny  (s) Skip this package\n`))

  // In real implementation, use readline or inquirer for interactive input
  // For now, return 'deny' as default (CI mode behavior)
  return 'deny'
}

export function promptCI(verdicts: Verdict[]): boolean {
  const critical = verdicts.filter(v => v.severity === 'CRITICAL')

  if (critical.length > 0) {
    console.log(chalk.red.bold(`\n🚨 CI MODE: ${critical.length} critical issues found. Failing build.\n`))
    return false
  }

  return true
}
