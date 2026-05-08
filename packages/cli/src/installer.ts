import { spawn } from 'child_process'
import chalk from 'chalk'
import { execSync } from 'child_process'

export async function runPackageManager(
  pm: string,
  args: string[]
): Promise<{ success: boolean; output: string }> {
  const pmMap: Record<string, string> = {
    npm: 'npm',
    pnpm: 'pnpm',
    bun: 'bun',
  }

  const command = pmMap[pm] || 'npm'

  // For install/add, we need to:
  // 1. Run with --ignore-scripts to install packages
  // 2. Then run scripts through sandbox
  const hasIgnoreScripts = args.some(a => a.includes('ignore-scripts'))
  
  if (!hasIgnoreScripts && (args.includes('install') || args.includes('add'))) {
    // Add --ignore-scripts flag
    args.push('--ignore-scripts')
  }

  // Remove --ignore-scripts from display (it's expected)
  const displayArgs = args.filter(a => a !== '--ignore-scripts')
  console.log(chalk.gray(`\n📦 Running: ${command} ${displayArgs.join(' ')} (with --ignore-scripts)\n`))

  return new Promise((resolve) => {
    // Use spawn without shell to avoid npm arborist bug triggered by shell escaping
    const proc = spawn(command, args, {
      stdio: 'inherit',
      shell: false
    })

    let output = ''
    proc.stdout?.on('data', (d: Buffer) => { output += d.toString() })
    proc.stderr?.on('data', (d: Buffer) => { output += d.toString() })

    proc.on('close', (code) => {
      // If npm fails with arborist error, try alternative approach
      if (code !== 0 && command === 'npm') {
        console.log(chalk.yellow('\n⚠️  npm encountered an error, trying with --no-audit --no-fund...\n'))
        try {
          // Try running npm without audit/fund which sometimes triggers arborist issues
          const fallbackArgs = ['install', '--ignore-scripts', '--no-audit', '--no-fund', ...args.filter(a => a !== 'install' && a !== 'add' && a !== '--ignore-scripts')]
          const result = execSync(`npm ${fallbackArgs.join(' ')}`, { stdio: 'inherit', encoding: 'utf-8' })
          resolve({ success: true, output: result || '' })
          return
        } catch (e) {
          // Fallback failed too
        }
      }
      
      resolve({
        success: code === 0,
        output
      })
    })
  })
}
