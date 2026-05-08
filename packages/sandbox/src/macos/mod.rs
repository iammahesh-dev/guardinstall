pub mod seatbelt;

use napi::Error;

/// Apply macOS sandbox using Seatbelt (sandbox-exec)
pub fn sandbox_macos(script_path: &str) -> napi::Result<()> {
    seatbelt::sandbox_macos(script_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_sandbox_placeholder() {
        // This will fail on non-macOS, which is expected
        let _ = sandbox_macos("/test/script.sh");
    }
}
