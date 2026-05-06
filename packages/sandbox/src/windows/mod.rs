//! Windows sandbox placeholder
//! Real implementation requires winapi crate + WFP

use napi::Error;

pub fn sandbox_windows(_script_path: &str) -> napi::Result<()> {
    // Phase 4: Windows requires:
    // 1. Job Objects for process isolation
    // 2. Windows Filtering Platform (WFP) for network isolation (admin only)
    // For now: detect-and-alert mode
    Err(Error::new(
        napi::Status::GenericFailure,
        "Windows sandbox not yet implemented (detect mode only)".to_string()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_placeholder() {
        let result = sandbox_windows("/test/script.bat");
        assert!(result.is_err());
    }
}
