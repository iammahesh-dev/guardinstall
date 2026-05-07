//! Simple sandboxer that uses libc::prctl directly for seccomp
use libc::{self, c_ulong, c_void};
use std::os::unix::process::CommandExt;
use std::process::Command;
use nix::sched;
use serde_json::{json, Value};
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <script_path> <package_name> [timeout_sec]", args[0]);
        std::process::exit(1);
    }

    let script_path = &args[1];
    let package_name = &args[2];
    let timeout_sec = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(300);

    eprintln!("Running script: {}", script_path);
    eprintln!("Package: {}", package_name);

    // Fork the process
    let pid = unsafe { libc::fork() };
    
    if pid < 0 {
        eprintln!("Fork failed: {}", std::io::Error::last_os_error());
        std::process::exit(1);
    }

    if pid == 0 {
        // Child process - apply seccomp and run script
        if let Err(e) = apply_seccomp_simple() {
            eprintln!("Failed to apply seccomp: {}", e);
            std::process::exit(1);
        }

        // Run the script
        let output = Command::new("bash")
            .arg(script_path)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let event = json!({
                        "action": "allowed",
                        "event": "script_completed",
                        "package": package_name,
                        "timestamp_ns": get_timestamp_ns()
                    });
                    eprintln!("{}", serde_json::to_string(&event).unwrap());
                    std::process::exit(0);
                } else {
                    let event = json!({
                        "action": "blocked",
                        "event": "script_failed",
                        "package": package_name,
                        "timestamp_ns": get_timestamp_ns()
                    });
                    eprintln!("{}", serde_json::to_string(&event).unwrap());
                    std::process::exit(output.status.code().unwrap_or(1));
                }
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

fn apply_seccomp_simple() -> Result<(), Box<dyn std::error::Error>> {
    // Use prctl to set no_new_privs first
    let res = unsafe {
        libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0)
    };
    
    if res != 0 {
        return Err(format!("prctl NO_NEW_PRIVS failed: {}", std::io::Error::last_os_error()).into());
    }
    
    eprintln!("NO_NEW_PRIVS set successfully");

    // For now, just set up the seccomp filter using prctl
    // This is a simplified approach - we'll add real BPF later
    Ok(())
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
