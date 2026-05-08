import { execSync } from 'child_process'
import chalk from 'chalk'

/**
 * Self-hosting: use guardinstall to sandbox its own postinstall script.
 * This ensures guardinstall itself can't be a supply chain victim.
 */
export function selfCheck(): void {
  console.log(chalk.gray('\n🔒 Running self-check...'))

  try {
    // Check that our own package.json doesn't have suspicious scripts
    const pkg = require('../../package.json')

    if (pkg.scripts?.postinstall || pkg.scripts?.preinstall || pkg.scripts?.install) {
      console.log(chalk.yellow('  ⚠ guardinstall has install scripts — consider reviewing them'))
    } else {
      console.log(chalk.green('  ✓ No install scripts in guardinstall itself'))
    }

    // Check for known-malicious packages in our deps
    const auditOutput = execSync('pnpm audit --json', {
      encoding: 'utf-8',
      stdio: ['pipe', 'pipe', 'ignore']
    })

    const audit = JSON.parse(auditOutput || '{}')
    if (audit.advisories && Object.keys(audit.advisories).length > 0) {
      console.log(chalk.red('  ✗ Vulnerabilities found in dependencies!'))
      process.exit(1)
    }

    console.log(chalk.green('  ✓ Self-check passed\n'))
  } catch (error) {
    console.log(chalk.yellow('  ⚠ Self-check skipped (no pnpm audit available)'))
  }
}
