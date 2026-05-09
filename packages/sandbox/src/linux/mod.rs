pub mod seccomp;
pub mod landlock_bin;
pub mod events;

use napi::{Error, Status};

pub fn sandbox_linux(script_path: &str) -> napi::Result<()> {
    let path = std::path::Path::new(script_path);

    if !path.exists() {
        return Err(Error::new(
            Status::GenericFailure,
            "Script not found".to_string(),
        ));
    }

    // Phase 2: Apply Linux namespaces (network isolation)
    // TODO: implement namespaces::create_namespaces()?;

    // Phase 2: Apply seccomp-BPF filter
    if let Err(e) = seccomp::apply_seccomp() {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Failed to apply seccomp: {}", e),
        ));
    }

    // Emit event that seccomp was applied
    let event = events::SandboxEvent {
        event_type: "seccomp_applied".to_string(),
        package: "unknown".to_string(),
        syscall: None,
        args: None,
        path: None,
        action: "allowed".to_string(),
        timestamp_ns: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64,
    };
    let _ = events::emit_event(&event);

    // Phase 2: Apply Landlock filesystem restrictions
    if let Err(e) = landlock_bin::apply_land_lock(script_path) {
        let _ = events::emit_event(&events::SandboxEvent {
            event_type: "landlock_failed".to_string(),
            package: "unknown".to_string(),
            syscall: None,
            args: None,
            path: Some(e.to_string()),
            action: "error".to_string(),
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_nonexistent_script() {
        let result = sandbox_linux("/nonexistent/script.sh");
        assert!(result.is_err());
    }
}
