/**
 * Invoke the Rust sandboxer binary
 * Spawns child process that applies seccomp + runs script
 */

import { spawnSync, SpawnSyncReturns } from 'child_process';
import * as path from 'path';
import { SandboxEvent } from '@guardinstall/policy-engine';
import * as os from 'os';

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

  if (platform === 'linux') {
    return arch === 'arm64' ? 'sandboxer-linux-arm64' : 'sandboxer-linux-x64';
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

  // Look for binary in multiple locations:
  // 1. Relative to CLI package (for development)
  // 2. In node_modules/.bin (for npm global install)
  // 3. In package's native directory (for pre-built binaries)
  const possiblePaths = [
    path.join(__dirname, '..', 'sandbox', 'target', 'release', binaryName),
    path.join(__dirname, '..', '..', 'sandbox', 'target', 'release', binaryName),
    path.join(__dirname, '..', '..', '..', 'node_modules', '@guardinstall', 'sandbox', 'native', binaryName),
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
    // Fallback: try to find in PATH
    binaryPath = path.join(__dirname, '..', '..', '..', '.bin', 'sandboxer') ||
                  path.join(__dirname, '..', '..', 'bin', 'sandboxer');
  }

  try {
    const result: SpawnSyncReturns<string> = spawnSync(
      binaryPath,
      [scriptPath, packageName],
      {
        encoding: 'utf-8',
        timeout: 30000, // 30 second timeout
      }
    );

    const events: SandboxEvent[] = [];
    const stderr = result.stderr || '';

    // Parse JSON events from stderr (one per line)
    stderr.split('\n').forEach((line: string) => {
      if (!line.trim()) return;
      try {
        const event = JSON.parse(line);
        if (event.event) {
          events.push(event);
        }
      } catch {
        // Ignore non-JSON lines
      }
    });

    // Consider blocked if any event has action 'blocked' or 'error'
    // Note: 'error' action is cast to satisfy SandboxEvent type
    const blocked = events.some(e => e.action === 'blocked' || (e as any).action === 'error');

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
