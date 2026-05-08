export type Severity = 'CRITICAL' | 'HIGH' | 'WARN' | 'INFO';
export interface SandboxEvent {
    event: string;
    package: string;
    syscall?: string;
    args?: unknown;
    path?: string;
    action: 'blocked' | 'allowed' | 'logged';
    timestamp_ns: number;
}
export interface ScoringRule {
    match: (event: SandboxEvent) => boolean;
    severity: Severity;
    message: string;
}
export interface Verdict {
    package: string;
    events: SandboxEvent[];
    findings: Finding[];
    severity: Severity;
}
export interface Finding {
    severity: Severity;
    message: string;
    event: SandboxEvent;
}
