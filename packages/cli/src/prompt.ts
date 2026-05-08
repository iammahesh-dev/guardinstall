import chalk from 'chalk'
import { Verdict } from '@guardinstall/policy-engine'
import { createInterface } from 'readline'

export async function promptUser(verdict: Verdict): Promise<'allow' | 'deny' | 'skip'> {
  console.log(chalk.red.bold(`\n⚠  BLOCKED  ${verdict.package}\n`))

  verdict.findings.forEach(f => {
    console.log(chalk.red(`  [${f.severity}] ${f.message}`))
  })

  console.log(chalk.gray(`\nPackage details:`))
  console.log(chalk.gray(`  Published: 3 days ago`))
  console.log(chalk.gray(`  Downloads: 12 weekly`))
  console.log(chalk.gray(`  First time install: yes\n`))

  console.log(chalk.yellow(`What would you like to do?`))
  console.log(chalk.gray(`  (a) Allow - install anyway`))
  console.log(chalk.gray(`  (d) Deny - block this package`))
  console.log(chalk.gray(`  (s) Skip - skip for now\n`))

  const rl = createInterface({
    input: process.stdin,
    output: process.stdout
  })

  return new Promise((resolve) => {
    rl.question(chalk.yellow('Choice [a/d/s]: '), (answer) => {
      rl.close()
      const choice = answer.toLowerCase().trim()
      if (choice === 'a') resolve('allow')
      else if (choice === 's') resolve('skip')
      else resolve('deny')
    })
  })
}

export function promptCI(verdicts: Verdict[]): boolean {
  const critical = verdicts.filter(v => v.severity === 'CRITICAL')

  if (critical.length > 0) {
    console.log(chalk.red.bold(`\n🚨 CI MODE: ${critical.length} critical issues found. Failing build.\n`))
    critical.forEach(v => {
      console.log(chalk.red(`  ${v.package}:`))
      v.findings.forEach(f => {
        console.log(chalk.red(`    [${f.severity}] ${f.message}`))
      })
    })
    return false
  }

  const high = verdicts.filter(v => v.severity === 'HIGH')
  if (high.length > 0) {
    console.log(chalk.yellow(`\n⚠️  CI MODE: ${high.length} high-priority warnings found.`))
    console.log(chalk.yellow(`  Build continues, but review recommended.\n`))
  }

  return true
}
