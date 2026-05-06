pub mod seatbelt;

use napi::Error;

/// Apply macOS sandbox using Seatbelt (sandbox-exec)
pub fn sandbox_macos(script_path: &str) -> napi::Result<()> {
    // Phase 4: macOS implementation
    // 1. Generate Seatbelt profile
    // 2. Run: sandbox-exec -f profile script_path

    let path = std::path::Path::new(script_path);
    let profile = seatbelt::generate_profile(path);

    // In production, write profile to temp file and execute
    // For now, return placeholder
    Err(Error::new(
        napi::Status::GenericFailure,
        "macOS Seatbelt not yet implemented".to_string()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_sandbox_placeholder() {
        let result = sandbox_macos("/test/script.sh");
        assert!(result.is_err());
    }
}
