#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;
pub mod benchmark;

use napi_derive::napi;

#[napi]
pub fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

#[napi]
pub fn sandbox_process(script_path: String) -> napi::Result<()> {
    sandbox_process_impl(&script_path)
}

#[cfg(target_os = "linux")]
fn sandbox_process_impl(script_path: &str) -> napi::Result<()> {
    linux::sandbox_linux(script_path)
}

#[cfg(target_os = "windows")]
fn sandbox_process_impl(script_path: &str) -> napi::Result<()> {
    windows::sandbox_windows(script_path)
}

#[cfg(target_os = "macos")]
fn sandbox_process_impl(script_path: &str) -> napi::Result<()> {
    macos::sandbox_macos(script_path)
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn sandbox_process_impl(_script_path: &str) -> napi::Result<()> {
    Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Unsupported platform".to_string(),
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
