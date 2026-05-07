//! Windows sandbox - Job Objects implementation
//! Real implementation requires winapi crate + WFP

use napi::Error;

pub fn sandbox_windows(script_path: &str) -> napi::Result<()> {
    // Use Job Objects for process isolation
    job_objects::sandbox_windows(script_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_placeholder() {
        // This will fail on non-Windows, which is expected
        let _ = sandbox_windows("/test/script.bat");
    }
}
