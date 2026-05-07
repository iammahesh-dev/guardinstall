//! Standalone sandboxer binary
//! Applies seccomp-BPF and runs script in sandboxed environment
//! Events are emitted to stderr (JSON lines)

use std::collections::BTreeMap;
use std::env;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::process::Command;
use serde_json::json;
use seccompiler::{SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition, SeccompFilter, SeccompRule, BpfProgram};

/// Platform-specific sandbox execution
#[cfg(target_os = "linux")]
fn run_sandboxed(script_path: &str, _package_name: &str) -> std::io::Result<std::process::Output> {
    let mut cmd = Command::new("sh");
    cmd.arg(script_path);

    unsafe {
        cmd.pre_exec(|| {
            // Apply network namespace isolation (child process only)
            apply_namespaces()?;

            // Apply seccomp-BPF filter (child process only)
            apply_seccomp()?;

            Ok(())
        });
    }

    cmd.output()
}

#[cfg(target_os = "macos")]
fn run_sandboxed(script_path: &str, _package_name: &str) -> std::io::Result<std::process::Output> {
    use crate::macos::seatbelt;
    seatbelt::sandbox_macos(script_path)
}

#[cfg(target_os = "windows")]
fn run_sandboxed(script_path: &str, _package_name: &str) -> std::io::Result<std::process::Output> {
    // Windows: Job Objects (detect mode only)
    emit_event_stderr("windows_sandbox", "Job Objects not yet implemented", "info");
    Command::new("sh").arg(script_path).output()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <script_path> [package_name]", args[0]);
        std::process::exit(1);
    }

    let script_path = &args[1];
    let package_name = if args.len() > 2 { &args[2] } else { "unknown" };

    // Execute the script with platform-specific sandbox
    let output = run_sandboxed(script_path, package_name);

    match output {
        Ok(result) => {
            if result.status.success() {
                emit_event("script_completed", package_name, None, None, "allowed");
                std::process::exit(0);
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                emit_event("script_failed", package_name, None, Some(&stderr), "blocked");
                std::process::exit(result.status.code().unwrap_or(1));
            }
        }
        Err(e) => {
            emit_event("script_error", package_name, None, Some(&e.to_string()), "error");
            std::process::exit(1);
        }
    }
}

/// Apply network namespace isolation
#[cfg(target_os = "linux")]
fn apply_namespaces() -> Result<(), std::io::Error> {
    use nix::sched::{unshare, CloneFlags};

    // Create new network namespace (child has no network interfaces)
    match unshare(CloneFlags::CLONE_NEWNET) {
        Ok(_) => {
            emit_event_stderr("namespace_applied", "network", "allowed");
            Ok(())
        }
        Err(e) => {
            // Fall back to user namespace + network namespace
            if unshare(CloneFlags::CLONE_NEWUSER).is_ok() {
                if unshare(CloneFlags::CLONE_NEWNET).is_ok() {
                    emit_event_stderr("namespace_applied", "user+network", "allowed");
                    return Ok(());
                }
            }
            // Log but continue (seccomp still provides protection)
            emit_event_stderr("namespace_failed", &e.to_string(), "info");
            Ok(())
        }
    }
}

/// Apply seccomp-BPF filter
#[cfg(target_os = "linux")]
fn apply_seccomp() -> Result<(), std::io::Error> {
    // Build seccomp filter with dangerous syscalls blocked
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();

    // Block execve, execveat, ptrace
    rules.insert(libc::SYS_execve as i64, vec![]);
    rules.insert(libc::SYS_execveat as i64, vec![]);
    rules.insert(libc::SYS_ptrace as i64, vec![]);

    // Block socket(AF_INET) - only when domain == AF_INET
    let socket_condition: SeccompCondition = SeccompCondition::new(
        0, // arg0 (domain)
        SeccompCmpArgLen::Dword,
        SeccompCmpOp::Eq,
        libc::AF_INET as u64,
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let socket_rule: SeccompRule = SeccompRule::new(vec![socket_condition])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    rules.insert(libc::SYS_socket as i64, vec![socket_rule]);

    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Errno(libc::EPERM as u32), // Action when rule matches
        SeccompAction::Allow,                      // Default action (allow others)
        get_target_arch(),
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    // Convert to BpfProgram and apply
    let bpf_program: BpfProgram = filter.try_into().map_err(|e: seccompiler::BackendError| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;
    seccompiler::apply_filter(&bpf_program)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    emit_event_stderr("seccomp_applied", "execve,socket,ptrace blocked", "allowed");
    Ok(())
}

/// Get target architecture for seccomp
#[cfg(target_os = "linux")]
fn get_target_arch() -> seccompiler::TargetArch {
    if cfg!(target_arch = "aarch64") {
        seccompiler::TargetArch::aarch64
    } else {
        seccompiler::TargetArch::x86_64
    }
}

/// Emit event to stderr (JSON)
fn emit_event(event_type: &str, package: &str, syscall: Option<&str>, details: Option<&str>, action: &str) {
    let event = json!({
        "event": event_type,
        "package": package,
        "syscall": syscall,
        "args": null,
        "path": details,
        "action": action,
        "timestamp_ns": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    });
    let _ = writeln!(io::stderr(), "{}", serde_json::to_string(&event).unwrap());
}

/// Emit event directly to stderr (for pre_exec context)
fn emit_event_stderr(event_type: &str, details: &str, action: &str) {
    let event = json!({
        "event": event_type,
        "details": details,
        "action": action,
        "timestamp_ns": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    });
    let _ = writeln!(io::stderr(), "{}", serde_json::to_string(&event).unwrap());
}
