//! Main sandboxer binary - NETWORK NAMESPACE VERSION
//! Uses network namespace isolation (requires root/CAP_SYS_ADMIN)
//! Seccomp-BPF temporarily disabled (EINVAL issues)
//! 
//! Usage: sandboxer <script_path> <package_name>

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
        
        // Run the script
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

/// Apply restrictions: network namespace (primary), seccomp (disabled)
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
    
    // 2. TODO: Seccomp-BPF (disabled - BPF program issues)
    // Tracking: https://github.com/iammahesh-dev/guardinstall/issues
    // BPF filter causes EINVAL - need to fix before enabling
    
    eprintln!("Restrictions applied (network namespace: active if root)");
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
