pub mod seccomp;
pub mod namespaces;
pub mod landlock;

use napi::Error;
use std::path::Path;

/// Apply all Linux sandboxing techniques
pub fn sandbox_linux(script_path: &str) -> napi::Result<()> {
    let path = Path::new(script_path);

    if !path.exists() {
        return Err(Error::new(
            napi::Status::GenericFailure,
            "Script not found".to_string(),
        ));
    }

    // Phase 2: Apply Linux namespaces (network isolation)
    namespaces::create_namespaces()?;

    // Phase 2: Apply seccomp-BPF filter
    seccomp::apply_seccomp()?;

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
