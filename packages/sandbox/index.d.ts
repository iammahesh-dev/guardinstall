// Type declarations for @guardinstall/sandbox native addon

export function getPlatform(): string;
export function sandboxProcess(scriptPath: string): void;

// JSON event structure from Rust
export interface SandboxEvent {
  event_type: string;
  package: string;
  syscall?: string;
  args?: unknown;
  path?: string;
  action: string;
  timestamp_ns: number;
}
