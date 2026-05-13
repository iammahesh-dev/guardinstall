/**
 * Invoke the Rust sandboxer binary
 * Spawns child process that applies seccomp + runs script
 */

import { spawnSync, SpawnSyncReturns } from 'child_process';
import * as path from 'path';
import { SandboxEvent } from '@guardinstall/policy-engine';
import * as os from 'os';
import * as fs from 'fs';

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
  const candidates = [
    path.join(__dirname, '../../policies', `${packageName}.json`),
    path.join(__dirname, '../../policy-engine/policies', `${packageName}.json`),
  ]
  for (const policyPath of candidates) {
    if (!fs.existsSync(policyPath)) continue
    try {
      return JSON.parse(fs.readFileSync(policyPath, 'utf-8'))
    } catch {}
  }
  return null
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

  // Use cargo's default binary name for development
  if (platform === 'win32') return 'sandboxer.exe';

  return 'sandboxer';
}

export function findSandboxerBinary(): string | null {
  const binaryName = getBinaryName();
  const possiblePaths = [
    path.join(__dirname, binaryName),
    path.join(__dirname, '..', 'packages', 'sandbox', 'target', 'release', binaryName),
    path.join(__dirname, '..', '..', 'packages', 'sandbox', 'target', 'release', binaryName),
    path.join(__dirname, '..', '..', '..', 'packages', 'sandbox', 'target', 'release', binaryName),
    path.join(__dirname, '..', '.bin', binaryName),
    path.join(__dirname, '..', 'native', binaryName),
    binaryName,
    ...(os.platform() === 'win32' ? ['sandboxer.exe'] : ['sandboxer']),
  ];

  for (const p of possiblePaths) {
    if (fs.existsSync(p)) return p
  }
  return null
}

/**
 * Run package install script in sandboxed environment
 * Uses standalone Rust binary (sandboxer)
 */
export function runSandboxed(scriptPath: string, packageName: string = 'unknown'): SandboxResult {
  const policy = loadPolicy(packageName);
  const isVerified = policy && policy.maintainers_verified;

  const binaryPath = findSandboxerBinary()

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
