pub mod job_objects;

use napi::Error;

/// Apply Windows sandbox using Job Objects
pub fn sandbox_windows(script_path: &str) -> napi::Result<()> {
    // Phase 4: Windows implementation
    // 1. Create Job Object
    // 2. Set restrictions (no new processes, limited resources)
    // 3. Assign process to Job Object

    // For now, detect-and-alert mode
    // Full blocking requires Windows Filtering Platform (WFP) + admin privileges

    Err(Error::new(
        napi::Status::GenericFailure,
        "Windows sandbox not yet implemented (detect mode only)".to_string()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_sandbox_placeholder() {
        let result = sandbox_windows("/test/script.bat");
        assert!(result.is_err());
    }
}
