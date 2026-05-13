import chalk from 'chalk'
import os from 'os'
import fs from 'fs'
import path from 'path'
import { execFileSync, execSync } from 'child_process'

interface CheckResult {
  name: string
  status: 'pass' | 'warn' | 'fail' | 'info'
  detail: string
}

export async function runCheck(): Promise<void> {
  console.log(chalk.blue.bold('\n═══════════════════════════════════════'))
  console.log(chalk.blue.bold('  guardinstall — Environment Check'))
  console.log(chalk.blue.bold('═══════════════════════════════════════\n'))

  const results: CheckResult[] = []

  // Platform
  const platform = os.platform()
  const arch = os.arch()
  results.push({ name: 'Platform', status: 'info', detail: `${platform} (${arch})` })

  // Kernel version
  if (platform === 'linux') {
    try {
      const release = execSync('uname -r', { encoding: 'utf-8' }).trim()
      const parts = release.split('.').map(Number)
      const major = parts[0] || 0
      results.push({
        name: 'Kernel',
        status: major >= 5 ? 'pass' : 'fail',
        detail: `${release} ${major >= 5 ? '(Landlock ready)' : '(Landlock requires 5.13+)'}`,
      })
    } catch {
      results.push({ name: 'Kernel', status: 'fail', detail: 'Could not detect kernel version' })
    }
  }

  // Landlock
  if (platform === 'linux') {
    try {
      const lsm = fs.readFileSync('/sys/kernel/security/lsm', 'utf-8')
      const hasLandlock = lsm.includes('landlock')
      if (hasLandlock) {
        let abi = -1
        try {
          abi = parseInt(fs.readFileSync('/sys/kernel/security/landlock/abi', 'utf-8').trim(), 10)
        } catch {}
        if (abi >= 1) {
          const featStr = abi >= 2 ? 'file read/write + file append + fs topology' : 'file read/write'
          results.push({ name: 'Landlock', status: 'pass', detail: `ABI v${abi} (supports ${featStr})` })
        } else if (abi === 0) {
          results.push({ name: 'Landlock', status: 'warn', detail: 'In LSM config but securityfs not mounted (mount -t securityfs securityfs /sys/kernel/security)' })
        } else {
          results.push({ name: 'Landlock', status: 'pass', detail: 'In LSM config (will apply at sandbox time)' })
        }
      } else {
        results.push({ name: 'Landlock', status: 'warn', detail: 'Not in LSM config (enable with lsm=landlock in kernel cmdline)' })
      }
    } catch {
      results.push({ name: 'Landlock', status: 'warn', detail: 'Could not detect' })
    }
  } else if (platform === 'darwin') {
    results.push({ name: 'Sandbox', status: 'info', detail: 'macOS Seatbelt (experimental, untested)' })
  } else if (platform === 'win32') {
    results.push({ name: 'Sandbox', status: 'info', detail: 'Windows Job Objects (experimental)' })
  }

  // Seccomp
  if (platform === 'linux') {
    let seccompOk = false
    try {
      const val = fs.readFileSync('/proc/sys/kernel/seccomp', 'utf-8').trim()
      if (val === '1' || val === '2') {
        results.push({ name: 'Seccomp', status: 'pass', detail: `Available (mode ${val})` })
        seccompOk = true
      }
    } catch {}
    if (!seccompOk) {
      try {
        const configGz = '/proc/config.gz'
        if (fs.existsSync(configGz)) {
          const data = fs.readFileSync(configGz, 'utf-8')
          if (data.includes('SECCOMP')) {
            results.push({ name: 'Seccomp', status: 'pass', detail: 'Built into kernel (will apply at sandbox time)' })
            seccompOk = true
          }
        }
      } catch {}
    }
    if (!seccompOk) {
      try {
        execFileSync('uname', ['-r'], { encoding: 'utf-8' })
        results.push({ name: 'Seccomp', status: 'info', detail: 'Seccomp file not found, but sandbox will try to apply it at runtime' })
      } catch {}
    }
  }

  // Architecture
  if (platform === 'linux' && arch === 'arm64') {
    results.push({ name: 'ARM64 Seccomp', status: 'warn', detail: 'BPF filter hardcodes x86-64 syscall numbers — seccomp is a no-op on ARM64. Consider contributing a fix.' })
  }

  // User namespaces
  if (platform === 'linux') {
    const usernsPaths = [
      '/proc/sys/kernel/unprivileged_user_namespaces',
      '/proc/sys/user/max_user_namespaces',
    ]
    let found = false
    for (const p of usernsPaths) {
      try {
        const val = parseInt(fs.readFileSync(p, 'utf-8').trim(), 10)
        if (val > 0) {
          results.push({ name: 'User Namespaces', status: 'pass', detail: `Available (max: ${val})` })
          found = true
          break
        }
      } catch {}
    }
    if (!found) {
      results.push({ name: 'User Namespaces', status: 'info', detail: 'Not detected (sandbox still works without it)' })
    }
  }

  // Sandboxer binary
  const { findSandboxerBinary } = await import('./sandboxer')
  const binPath = findSandboxerBinary()
  if (binPath) {
    results.push({ name: 'Sandboxer Binary', status: 'pass', detail: `Found at ${binPath}` })
  } else {
    results.push({ name: 'Sandboxer Binary', status: 'fail', detail: 'Not found — run cargo build first' })
  }

  // Package managers
  for (const pm of ['npm', 'pnpm', 'bun']) {
    try {
      const ver = execFileSync(pm, ['--version'], { encoding: 'utf-8', timeout: 3000 }).trim().split('\n')[0]
      results.push({ name: `${pm}`, status: 'pass', detail: `v${ver}` })
    } catch {
      results.push({ name: `${pm}`, status: 'info', detail: 'Not found' })
    }
  }

  // Print results
  for (const r of results) {
    const icon = r.status === 'pass' ? chalk.green('✓') : r.status === 'warn' ? chalk.yellow('⚠') : r.status === 'fail' ? chalk.red('✗') : chalk.gray('ℹ')
    const label = r.status === 'pass' ? chalk.green(r.name) : r.status === 'warn' ? chalk.yellow(r.name) : r.status === 'fail' ? chalk.red(r.name) : chalk.white(r.name)
    console.log(`  ${icon}  ${label}`)
    console.log(`     ${chalk.gray(r.detail)}`)
  }

  // Summary
  const passed = results.filter(r => r.status === 'pass').length
  const warned = results.filter(r => r.status === 'warn').length
  const failed = results.filter(r => r.status === 'fail').length
  console.log(chalk.blue.bold('\n═══════════════════════════════════════'))
  console.log(`  ${chalk.green(`✓ ${passed} passed`)}  ${chalk.yellow(`⚠ ${warned} warnings`)}  ${chalk.red(`✗ ${failed} failed`)}`)
  console.log(chalk.blue.bold('═══════════════════════════════════════\n'))
}
