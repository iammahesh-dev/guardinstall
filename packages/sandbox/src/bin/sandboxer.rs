//! Standalone sandboxer binary - MINIMAL WORKING VERSION
//! Just get the basic execution working, add seccomp later

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

    eprintln!("Running script: {}", script_path);

    // Just run the script without any sandboxing for now
    let output = Command::new("sh")
        .arg(script_path)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                emit_event("script_completed", package_name, "allowed");
                std::process::exit(0);
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                if !stderr.is_empty() {
                    eprintln!("Script stderr: {}", stderr);
                }
                emit_event("script_failed", package_name, "blocked");
                std::process::exit(result.status.code().unwrap_or(1));
            }
        }
        Err(e) => {
            eprintln!("Error running script: {}", e);
            emit_event("script_error", package_name, "error");
            std::process::exit(1);
        }
    }
}

fn emit_event(event_type: &str, package: &str, action: &str) {
    let event = json!({
        "event": event_type,
        "package": package,
        "action": action,
        "timestamp_ns": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    });
    let _ = writeln!(io::stderr(), "{}", serde_json::to_string(&event).unwrap());
}
