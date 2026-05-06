import { spawn } from 'child_process'
import chalk from 'chalk'

export async function runPackageManager(
  pm: string,
  args: string[]
): Promise<{ success: boolean; output: string }> {
  const pmMap: Record<string, string> = {
    npm: 'npm',
    pnpm: 'pnpm',
    bun: 'bun'
  }

  const command = pmMap[pm] || 'npm'

  console.log(chalk.gray(`\n📦 Running: ${command} ${args.join(' ')}\n`))

  return new Promise((resolve) => {
    const proc = spawn(command, args, {
      stdio: 'inherit',
      shell: true
    })

    let output = ''
    proc.stdout?.on('data', (d: Buffer) => { output += d.toString() })
    proc.stderr?.on('data', (d: Buffer) => { output += d.toString() })

    proc.on('close', (code) => {
      resolve({
        success: code === 0,
        output
      })
    })
  })
}
