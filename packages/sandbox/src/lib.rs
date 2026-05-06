mod linux;

use napi_derive::napi;

#[napi]
pub fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

#[napi]
pub fn sandbox_process(script_path: String) -> napi::Result<()> {
    let platform = std::env::consts::OS;

    match platform {
        "linux" => linux::sandbox_linux(&script_path),
        "macos" => sandbox_macos(&script_path),
        "windows" => sandbox_windows(&script_path),
        _ => Err(napi::Error::new(
            napi::Status::GenericFailure,
            "Unsupported platform".to_string(),
        )),
    }
}

fn sandbox_macos(_script_path: &str) -> napi::Result<()> {
    // Phase 4: macOS Seatbelt implementation
    Err(napi::Error::new(
        napi::Status::GenericFailure,
        "macOS sandbox not yet implemented".to_string(),
    ))
}

fn sandbox_windows(_script_path: &str) -> napi::Result<()> {
    // Phase 4: Windows Job Objects implementation
    Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Windows sandbox not yet implemented".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform() {
        let platform = get_platform();
        assert!(!platform.is_empty());
    }

    #[test]
    fn test_sandbox_process_nonexistent() {
        let result = sandbox_process("/nonexistent/script.sh".to_string());
        assert!(result.is_err());
    }
}
