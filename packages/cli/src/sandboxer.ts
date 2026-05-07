/**
 * Invoke the Rust sandboxer binary
 * Spawns child process that applies seccomp + runs script
 */

import { spawnSync, SpawnSyncReturns } from 'child_process';
import * as path from 'path';
import { SandboxEvent } from '@guardinstall/policy-engine';
import * as os from 'os';
import * as fs from 'fs';
import chalk from 'chalk';

interface PolicyProfile {
  package: string;
  versions: string;
  maintainers_verified: boolean;
  expected_behavior: {
    network?: { allowed_hosts: string[]; reason: string };
    filesystem?: { writes: string[]; reason: string };
    exec: boolean;
  };
}

function loadPolicy(packageName: string): PolicyProfile | null {
  const policyPath = path.join(__dirname, '../../policy-engine/policies', `${packageName}.json`);
  if (!fs.existsSync(policyPath)) return null;
  try {
    return JSON.parse(fs.readFileSync(policyPath, 'utf-8'));
  } catch {
    return null;
  }
}

interface SandboxResult {
  package: string;
  blocked: boolean;
  events: SandboxEvent[];
}

/**
 * Get platform-specific binary name
 */
function getBinaryName(): string {
  const platform = os.platform();
  const arch = os.arch();

  // For development, just use 'sandboxer' (built by cargo)
  // For production/distribution, use platform-specific names
  if (platform === 'linux') {
    return arch === 'arm64' ? 'sandboxer-linux-arm64' : 'sandboxer';
  }
  if (platform === 'darwin') {
    return arch === 'arm64' ? 'sandboxer-macos-arm64' : 'sandboxer-macos-x64';
  }
  if (platform === 'win32') {
    return 'sandboxer-windows-x64.exe';
  }

  throw new Error(`Unsupported platform: ${platform} ${arch}`);
}

/**
 * Run package install script in sandboxed environment
 * Uses standalone Rust binary (sandboxer)
 */
export function runSandboxed(scriptPath: string, packageName: string = 'unknown'): SandboxResult {
  const binaryName = getBinaryName();
  
  // Check if package has verified policy profile
  const policy = loadPolicy(packageName);
  const isVerified = policy && policy.maintainers_verified;

  // Look for binary in multiple locations:
  // 1. Relative to CLI package (for development)
  // 2. In the guardinstall project (for development)
  // 3. In node_modules/.bin (for npm global install)
  // 4. In package's native directory (for pre-built binaries)
  const possiblePaths = [
    path.join(__dirname, '..', 'packages', 'sandbox', 'target', 'release', binaryName),
    path.join(__dirname, '..', '..', 'packages', 'sandbox', 'target', 'release', binaryName),
    path.join(__dirname, '..', '..', '..', 'packages', 'sandbox', 'target', 'release', binaryName),
    path.join(__dirname, '..', '..', '..', '.bin', binaryName),
    path.join(__dirname, '..', '..', 'bin', binaryName),
    path.join(__dirname, '..', 'native', binaryName),
  ];

  let binaryPath: string | null = null;
  for (const p of possiblePaths) {
    if (require('fs').existsSync(p)) {
      binaryPath = p;
      break;
    }
  }

  if (!binaryPath) {
    throw new Error(
      `guardinstall: sandboxer binary not found. ` +
      `Run 'cd packages/sandbox && cargo build --release --bin sandboxer' first.`
    );
  }

  try {
    // If package is verified, run in relaxed mode (no seccomp - allows network)
    const args = isVerified 
      ? [scriptPath, packageName, '--no-seccomp']
      : [scriptPath, packageName];
    
    const result: SpawnSyncReturns<string> = spawnSync(
      binaryPath,
      args,
      {
        encoding: 'utf-8',
        timeout: 30000 // 30 second timeout
      }
    );

    const events: SandboxEvent[] = [];
    const stderr = result.stderr || '';
    const stdout = result.stdout || '';
    const output = stderr + '\n' + stdout;

    // Parse JSON events from output (one per line)
    // sandboxer outputs JSON to stderr: {"action":"blocked","event":"script_blocked",...}
    output.split('\n').forEach((line: string) => {
      if (!line.trim()) return;
      try {
        const event = JSON.parse(line);
        // Check for both event.event and event.action fields
        if (event.event || event.action) {
          events.push(event);
        }
      } catch {
        // Ignore non-JSON lines (like "Applying Landlock..." etc.)
      }
    });

    // Consider blocked if action is 'blocked'
    const blocked = events.some(e => 
      e.action === 'blocked' || 
      e.event === 'script_blocked'
    );

    return {
      package: packageName,
      blocked,
      events,
    };
  } catch (error: any) {
    return {
      package: packageName,
      blocked: true,
      events: [{
        event: 'sandbox_error',
        package: packageName,
        syscall: undefined,
        args: error.message,
        path: undefined,
        action: 'blocked', // Cast to 'blocked' to satisfy type
        timestamp_ns: Date.now() * 1000000,
      } as SandboxEvent],
    };
  }
}
