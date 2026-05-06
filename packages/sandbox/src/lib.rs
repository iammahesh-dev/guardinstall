pub mod linux;
pub mod macos;
pub mod windows;
pub mod benchmark;

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
        "macos" => macos::sandbox_macos(&script_path),
        "windows" => windows::sandbox_windows(&script_path),
        _ => Err(napi::Error::new(
            napi::Status::GenericFailure,
            "Unsupported platform".to_string(),
        )),
    }
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
