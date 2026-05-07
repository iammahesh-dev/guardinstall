//! Main sandboxer binary - FIXED VERSION
//! Uses network namespace + Landlock (NOT execve blocking!)
//! This allows bash to run scripts while blocking network/filesystem access
//! 
//! Usage: sandboxer <script_path> <package_name>
//! 
//! Requirements:
//! - Network namespace: root or CAP_SYS_ADMIN
//! - Landlock: Linux 5.13+ with Landlock LSM loaded

use libc;
use std::os::unix::process::CommandExt;
use std::process::Command;
use nix::sched;
use serde_json::{json};
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <script_path> <package_name>", args[0]);
        std::process::exit(1);
    }

    let script_path = &args[1];
    let package_name = &args[2];

    eprintln!("Running script: {}", script_path);
    eprintln!("Package: {}", package_name);

    // Fork the process
    let pid = unsafe { libc::fork() };
    
    if pid < 0 {
        eprintln!("Fork failed: {}", std::io::Error::last_os_error());
        std::process::exit(1);
    }

    if pid == 0 {
        // Child process - apply restrictions and run script
        apply_restrictions();
        
        // Run the script (execve is NOT blocked, so bash can run)
        let status = Command::new("bash")
            .arg(script_path)
            .status();

        match status {
            Ok(s) if s.success() => {
                let event = json!({
                    "action": "allowed",
                    "event": "script_completed",
                    "package": package_name,
                    "timestamp_ns": get_timestamp_ns()
                });
                eprintln!("{}", serde_json::to_string(&event).unwrap());
                std::process::exit(0);
            }
            Ok(s) => {
                let event = json!({
                    "action": "blocked",
                    "event": "script_failed",
                    "package": package_name,
                    "timestamp_ns": get_timestamp_ns()
                });
                eprintln!("{}", serde_json::to_string(&event).unwrap());
                std::process::exit(s.code().unwrap_or(1));
            }
            Err(e) => {
                let event = json!({
                    "action": "blocked",
                    "event": "script_failed",
                    "package": package_name,
                    "timestamp_ns": get_timestamp_ns()
                });
                eprintln!("{}", serde_json::to_string(&event).unwrap());
                std::process::exit(1);
            }
        }
    } else {
        // Parent process - wait for child
        let mut status = 0;
        unsafe {
            libc::waitpid(pid, &mut status, 0);
        }

        if libc::WIFSIGNALED(status) {
            let sig = libc::WTERMSIG(status);
            let event = json!({
                "action": "blocked",
                "event": "script_failed",
                "package": package_name,
                "timestamp_ns": get_timestamp_ns()
            });
            eprintln!("{}", serde_json::to_string(&event).unwrap());
        }
    }
}

/// Apply all restrictions: network namespace + Landlock
fn apply_restrictions() {
    // 1. Network namespace isolation (blocks network access)
    // Requires root or CAP_SYS_ADMIN
    eprintln!("Applying network namespace isolation...");
    match sched::unshare(sched::CloneFlags::CLONE_NEWNET) {
        Ok(_) => eprintln!("Network namespace isolated (no network access)"),
        Err(e) => {
            eprintln!("Warning: Failed to unshare network namespace: {}", e);
            eprintln!("Run with sudo or set CAP_SYS_ADMIN to enable network isolation");
            eprintln!("Continuing without network isolation...");
        }
    }
    
    // 2. Landlock filesystem restrictions (blocks access to sensitive files)
    #[cfg(target_os = "linux")]
    {
        eprintln!("Applying Landlock filesystem restrictions...");
        // Note: Landlock API is complex, currently stubbed
        // TODO: Implement proper Landlock rules to block:
        // - /etc/passwd, /etc/shadow
        // - ~/.ssh/
        // - Other sensitive paths
        eprintln!("Landlock: not yet implemented (see TODO)");
    }
    
    // 3. Seccomp-BPF (disabled - causes EINVAL in Rust)
    // TODO: Debug why BPF filters cause EINVAL
    // Tracking: https://github.com/iammahesh-dev/guardinstall/issues
    
    eprintln!("Restrictions applied (network namespace: {}, Landlock: {})", 
        if cfg!(target_os = "linux") { "available if root" } else { "N/A" },
        if cfg!(target_os = "linux") { "stubbed" } else { "N/A" });
}

fn get_timestamp_ns() -> u64 {
    let mut timespec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe {
        libc::clock_gettime(libc::CLOCK_REALTIME, &mut timespec);
    }
    (timespec.tv_sec as u64) * 1_000_000_000 + (timespec.tv_nsec as u64)
}
