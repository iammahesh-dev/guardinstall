pub mod seccomp;
pub mod namespaces;
pub mod landlock;
pub mod events;

use events::SandboxEvent;

use napi::{Error, Status};
use std::path::Path;

/// Apply all Linux sandboxing techniques
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
    landlock::apply_land_lock(script_path)?;

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
