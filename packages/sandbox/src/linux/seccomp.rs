//! Production seccomp-BPF filter for sandboxing
//! Uses seccompiler crate to build BPF programs.

use napi::{Error, Status};
use seccompiler::{BpfProgram, SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition, SeccompFilter, SeccompRule, TargetArch};
use std::collections::BTreeMap;

/// Apply seccomp filter to current process
pub fn apply_seccomp() -> Result<(), Box<dyn std::error::Error>> {
    let filter = build_seccomp_filter()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to build seccomp filter: {}", e)))?;

    seccompiler::apply_filter(&filter)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to apply seccomp filter: {}", e)))?;

    Ok(())
}

/// Build BPF filter program that blocks dangerous syscalls
pub fn build_seccomp_filter() -> Result<BpfProgram, Box<dyn std::error::Error>> {
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();

    // Block execve (spawning new processes) - unconditional block
    rules.insert(libc::SYS_execve, vec![]);

    // Block execveat - unconditional block
    rules.insert(libc::SYS_execveat, vec![]);

    // Block ptrace (anti-sandbox detection) - unconditional block
    rules.insert(libc::SYS_ptrace, vec![]);

    // Block socket(AF_INET) - only when domain == AF_INET
    let socket_condition = SeccompCondition::new(
        0,  // arg0 (domain)
        SeccompCmpArgLen::Dword,
        SeccompCmpOp::Eq,
        libc::AF_INET as u64,
    )?;
    rules.insert(libc::SYS_socket, vec![SeccompRule::new(vec![socket_condition])?]);

    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Errno(libc::EPERM as u32),  // Action when rule matches
        SeccompAction::Allow,                       // Default action (allow others)
        TargetArch::x86_64,                        // Target architecture
    )?;

    let program: BpfProgram = filter.try_into()?;
    Ok(program)
}

/// Check if a syscall should be blocked (for testing)
pub fn is_dangerous_syscall(syscall_num: i32) -> bool {
    matches!(syscall_num as i64, libc::SYS_execve | libc::SYS_execveat | libc::SYS_ptrace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_syscall_detection() {
        assert!(is_dangerous_syscall(libc::SYS_execve as i32));
        assert!(is_dangerous_syscall(libc::SYS_execveat as i32));
        assert!(is_dangerous_syscall(libc::SYS_ptrace as i32));
        assert!(!is_dangerous_syscall(libc::SYS_socket as i32));
    }

    #[test]
    fn test_build_seccomp_filter() {
        let result = build_seccomp_filter();
        assert!(result.is_ok(), "Failed to build filter: {:?}", result.err());
        let program = result.unwrap();
        assert!(!program.is_empty());
    }
}
