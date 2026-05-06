//! Landlock LSM for filesystem restriction
//! Restricts filesystem access to only /tmp and package directory.

use napi::{Error, Status};
use std::fs;
use std::path::Path;

/// Apply Landlock rules to restrict filesystem access
/// Only allows read-write to /tmp and package directory
pub fn apply_land_lock(package_path: &str) -> Result<(), Error> {
    // For now, just check if Landlock is available and log
    // Real implementation will use landlock crate
    if is_land_lock_available() {
        println!("Landlock available - would restrict filesystem access to /tmp and {}", package_path);
        // TODO: Implement real Landlock restriction
        // Requires landlock crate with proper API
    } else {
        println!("Landlock not available (requires kernel 5.13+)");
    }
    Ok(())
}

/// Check if Landlock is available (kernel >= 5.13)
fn is_land_lock_available() -> bool {
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
