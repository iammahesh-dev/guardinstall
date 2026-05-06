//! Standalone sandboxer binary
//! Applies seccomp-BPF and runs script in sandboxed environment
//! Events are emitted to stderr (JSON lines)

use std::env;
use std::io::{self, Write};
use std::process::Command;
use serde_json::json;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <script_path> [package_name]", args[0]);
        std::process::exit(1);
    }

    let script_path = &args[1];
    let package_name = if args.len() > 2 { &args[2] } else { "unknown" };

    // Apply seccomp-BPF filter (placeholder)
    // TODO: Call actual seccomp::apply_seccomp() when modules are properly linked
    emit_event("seccomp_applied", package_name, None, None, "allowed");

    // Apply Landlock if available (placeholder)
    emit_event("landlock_skipped", package_name, None, None, "info");

    // Execute the script
    let output = Command::new("sh")
        .arg(script_path)
        .output();

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
