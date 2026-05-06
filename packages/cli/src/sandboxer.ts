/**
 * Invoke the Rust sandboxer binary
 * Spawns child process that applies seccomp + runs script
 */

import { spawnSync, SpawnSyncReturns } from 'child_process';
import * as path from 'path';
import { SandboxEvent } from '@guardinstall/policy-engine';

interface SandboxResult {
  package: string;
  blocked: boolean;
  events: SandboxEvent[];
}

/**
 * Run package install script in sandboxed environment
 * Uses standalone Rust binary (sandboxer)
 */
export function runSandboxed(scriptPath: string, packageName: string = 'unknown'): SandboxResult {
  const binaryPath = path.join(__dirname, '..', 'sandbox', 'target', 'release', 'sandboxer');
  
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
