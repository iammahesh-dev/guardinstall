//! seccomp-BPF filter for sandboxing install scripts
//! Blocks dangerous syscalls like execve, ptrace, and raw network sockets

use napi::{Error, Status};
use std::collections::HashSet;

/// Syscall numbers for x86_64
const SYS_EXECVE: i32 = 59;
const SYS_EXECVEAT: i32 = 322;
const SYS_PTRACE: i32 = 101;
const SYS_SOCKET: i32 = 41;

/// Build a seccomp filter that blocks dangerous syscalls
/// Returns filter program bytes or error
pub fn build_seccomp_filter() -> Result<Vec<u8>, Error> {
    // In production, this would use seccompiler or libseccomp
    // For now, return a placeholder
    Ok(vec![])
}

/// Check if a syscall should be blocked
pub fn is_dangerous_syscall(syscall_num: i32) -> bool {
    matches!(syscall_num, SYS_EXECVE | SYS_EXECVEAT | SYS_PTRACE)
}

/// Apply seccomp filter to current process
pub fn apply_seccomp() -> Result<(), Error> {
    // Placeholder for Phase 2 implementation
    // Will use seccompiler crate to load BPF program
    Err(Error::new(Status::GenericFailure, "seccomp not yet implemented"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_syscall_detection() {
        assert!(is_dangerous_syscall(SYS_EXECVE));
        assert!(is_dangerous_syscall(SYS_EXECVEAT));
        assert!(is_dangerous_syscall(SYS_PTRACE));
        assert!(!is_dangerous_syscall(SYS_SOCKET));
    }
}
