import { SandboxEvent } from './types';
export interface PolicyProfile {
    package: string;
    versions: string;
    maintainers_verified: boolean;
    expected_behavior: {
        network?: {
            allowed_hosts: string[];
            reason: string;
        };
        filesystem?: {
            writes: string[];
            reason: string;
        };
        exec: boolean;
    };
}
export declare function loadPolicy(packageName: string): PolicyProfile | null;
export declare function isBehaviorAllowed(profile: PolicyProfile, version: string, event: SandboxEvent): boolean;
