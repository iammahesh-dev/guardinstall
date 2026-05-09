import { spawn, execSync } from 'child_process'
import chalk from 'chalk'
import fs from 'fs'
import path from 'path'

export function detectPackageManager(cwd: string): string {
  const lockFiles = [
    { file: 'pnpm-lock.yaml', pm: 'pnpm' },
    { file: 'pnpm-workspace.yaml', pm: 'pnpm' },
    { file: 'yarn.lock', pm: 'yarn' },
    { file: 'bun.lockb', pm: 'bun' },
    { file: 'package-lock.json', pm: 'npm' },
  ]

  for (const { file, pm } of lockFiles) {
    if (fs.existsSync(path.join(cwd, file))) {
      return pm
    }
  }

  return 'npm'
}

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

  // For add, we need to manually add to package.json and run npm install
  if (args.includes('add') && command === 'npm') {
    return runNpmAddWithFallback(args)
  }

  // For install, just run with --ignore-scripts
  const hasIgnoreScripts = args.some(a => a.includes('ignore-scripts'))
  
  if (!hasIgnoreScripts && (args.includes('install') || args.includes('add'))) {
    args.push('--ignore-scripts')
  }

  const displayArgs = args.filter(a => a !== '--ignore-scripts')
  console.log(chalk.gray(`\n📦 Running: ${command} ${displayArgs.join(' ')} (with --ignore-scripts)\n`))

  return new Promise((resolve) => {
    const proc = spawn(command, args, {
      stdio: 'inherit',
      shell: false
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

// Workaround for npm arborist bug: manually add to package.json then run npm install
async function runNpmAddWithFallback(args: string[]): Promise<{ success: boolean; output: string }> {
  const packages = args.filter(a => !a.startsWith('-') && a !== 'add')
  
  if (packages.length === 0) {
    return { success: false, output: 'No packages specified' }
  }

  console.log(chalk.gray(`\n📦 Adding packages: ${packages.join(', ')}\n`))
  console.log(chalk.gray(`Step 1: Adding to package.json...\n`))

  try {
    // Read package.json
    const pkgPath = path.join(process.cwd(), 'package.json')
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'))

    // Add packages to dependencies
    if (!pkg.dependencies) pkg.dependencies = {}
    for (const pkgName of packages) {
      // Extract package@version if specified
      const match = pkgName.match(/^(@?[^@]+)(@.+)?$/)
      const name = match ? match[1] : pkgName
      const version = match && match[2] ? match[2] : 'latest'
      pkg.dependencies[name] = version
    }

    // Write back
    fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n')

    console.log(chalk.gray(`Step 2: Running npm install --ignore-scripts...\n`))
    
    // Detect actual package manager from project
    const detectedPM = detectPackageManager(process.cwd())
    console.log(chalk.gray(`Detected package manager: ${detectedPM}\n`))
    
    // Build install command based on detected PM
    let installCmd: string
    let installArgs: string[]
    
    switch (detectedPM) {
      case 'pnpm':
        installCmd = 'pnpm'
        installArgs = ['install', '--ignore-scripts', '--no-audit', '--no-fund']
        break
      case 'yarn':
        installCmd = 'yarn'
        installArgs = ['install', '--ignore-scripts', '--silent']
        break
      case 'bun':
        installCmd = 'bun'
        installArgs = ['install', '--ignore-scripts']
        break
      default:
        installCmd = 'npm'
        installArgs = ['install', '--ignore-scripts', '--no-audit', '--no-fund']
    }
    
    console.log(chalk.gray(`Step 2: Running ${installCmd} install...\n`))
    
    // Run install with detected package manager
    const result = execSync(`${installCmd} ${installArgs.join(' ')}`, {
      stdio: 'inherit',
      encoding: 'utf-8'
    })

    return { success: true, output: result || '' }
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e)
    console.error(chalk.red(`Error: ${errorMsg}`))
    return { success: false, output: errorMsg }
  }
}
