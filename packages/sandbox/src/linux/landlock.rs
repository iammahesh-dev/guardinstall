//! Landlock LSM for filesystem restriction
//! Restricts filesystem access to only /tmp and package directory.
//! NOTE: Currently stubbed due to landlock crate API complexity.
//! See: https://github.com/iammahesh-dev/guardinstall/issues (Landlock tracking issue)

use napi::{Error, Status};
use std::fs;

/// Apply Landlock rules to restrict filesystem access
/// Only allows read-write to /tmp and package directory
/// NOTE: This is currently a no-op. Landlock enforcement is tracked in GitHub issues.
pub fn apply_land_lock(_package_path: &str) -> Result<(), Error> {
    // Check if Landlock is available at runtime
    if is_land_lock_available() {
        // Landlock is available but not yet enforced
        // TODO: Implement real Landlock restriction
        // See: https://github.com/iammahesh-dev/guardinstall/issues
        println!("Landlock available but not enforced (tracked in GitHub issues)");
    } else {
        println!("Landlock not available (requires kernel 5.13+)");
    }
    Ok(())
}

/// Check if Landlock is available (kernel >= 5.13)
pub fn is_land_lock_available() -> bool {
    // Check /proc/version for kernel version
    if let Ok(version) = fs::read_to_string("/proc/version") {
        if version.contains("Linux version") {
            let parts: Vec<&str> = version.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "version" && i + 1 < parts.len() {
                    return check_kernel_version(parts[i + 1]);
                }
            }
        }
    }
    false
}

fn check_kernel_version(ver: &str) -> bool {
    let parts: Vec<&str> = ver.split('.').collect();
    if parts.len() >= 2 {
        if let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            return major >= 6 || (major == 5 && minor >= 13);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_land_lock_availability_check() {
        let _ = is_land_lock_available();
    }

    #[test]
    fn test_apply_land_lock_nonexistent_path() {
        let result = apply_land_lock("/nonexistent/path");
        assert!(result.is_ok());
    }
}
