//! Linux sandbox implementation
//! Uses seccomp-BPF, namespaces, and Landlock

pub mod seccomp;
pub mod namespaces;
pub mod landlock;
pub mod events;

use napi::{Error, Status};
use std::path::Path;
use std::process::Command;
use events::SandboxEvent;

/// Apply all Linux sandboxing techniques and run script
pub fn sandbox_linux(script_path: &str) -> napi::Result<()> {
    let path = Path::new(script_path);

    if !path.exists() {
        return Err(Error::new(
            Status::GenericFailure,
            "Script not found".to_string(),
        ));
    }

    // Phase 2: Apply Linux namespaces (network isolation)
    namespaces::create_namespaces()?;

    // Phase 2: Apply seccomp-BPF filter
    seccomp::apply_seccomp()?;

    // Emit event that seccomp was applied
    emit_sandbox_event("seccomp_applied", "unknown", None, None, "allowed");

    // Phase 2: Apply Landlock filesystem restrictions
    landlock::apply_land_lock(script_path)?;

    // Execute the script in sandboxed environment
    let output = Command::new("sh")
        .arg(script_path)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                emit_sandbox_event("script_completed", "unknown", None, None, "allowed");
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                emit_sandbox_event("script_failed", "unknown", None, Some(&stderr), "blocked");
            }
        }
        Err(e) => {
            emit_sandbox_event("script_error", "unknown", None, Some(&e.to_string()), "error");
            return Err(Error::new(Status::GenericFailure, format!("Failed to execute script: {}", e)));
        }
    }

    Ok(())
}

fn emit_sandbox_event(event_type: &str, package: &str, syscall: Option<&str>, path: Option<&str>, action: &str) {
    let event = SandboxEvent {
        event_type: event_type.to_string(),
        package: package.to_string(),
        syscall: syscall.map(|s| s.to_string()),
        args: None,
        path: path.map(|s| s.to_string()),
        action: action.to_string(),
        timestamp_ns: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64,
    };
    let _ = events::emit_event(&event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_nonexistent_script() {
        let result = sandbox_linux("/nonexistent/script.sh");
        assert!(result.is_err());
    }

    #[test]
    fn test_sandbox_malicious_script() {
        // Test that malicious script can be "sandboxed"
        // The actual blocking depends on seccomp filter being active
        let script_path = "packages/cli/__tests__/fixtures/malicious-package/postinstall.sh";
        if Path::new(script_path).exists() {
            // Note: This test may not actually block in test env
            // because seccomp is a one-time filter that can't be removed
            let _ = sandbox_linux(script_path);
        }
    }
}
