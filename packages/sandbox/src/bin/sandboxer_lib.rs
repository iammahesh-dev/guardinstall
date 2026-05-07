//! Sandboxer using seccompiler library (which has working tests)
use libc::{self, c_ulong};
use std::os::unix::process::CommandExt;
use std::process::Command;
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
        // Child process - apply seccomp and run script
        if let Err(e) = apply_seccomp_with_lib() {
            eprintln!("Failed to apply seccomp: {}", e);
            std::process::exit(1);
        }

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

/// Apply seccomp using the library function that works in tests
fn apply_seccomp_with_lib() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Using library's build_seccomp_filter()...");
    
    // Call the library function that works in tests
    let filter = crate::linux::seccomp::build_seccomp_filter()
        .map_err(|e| format!("Failed to build filter: {}", e))?;
    
    eprintln!("Applying seccomp filter...");
    
    seccompiler::apply_filter(&filter)
        .map_err(|e| format!("Failed to apply filter: {}", e))?;
    
    eprintln!("Seccomp filter applied successfully");
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
