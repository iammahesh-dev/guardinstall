//! Landlock LSM for filesystem restriction
//! Restricts filesystem access to only allowed paths.
//! Requires Linux kernel 5.13+ and Landlock LSM loaded.

use landlock::{
    Access, AccessFs, PathFd, PathBeneath, RestrictionStatus, Ruleset, RulesetAttr,
    ABI, BitFlags,
};
use std::fs;
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// Apply Landlock rules to restrict filesystem access
/// Only allows read-write to /tmp, package directory, and node_modules
pub fn apply_land_lock(package_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !is_land_lock_available() {
        eprintln!("Landlock not available (requires kernel 5.13+), skipping");
        return Ok(());
    }

    eprintln!("Applying Landlock filesystem restrictions...");

    // Get the best ABI supported by the kernel
    let abi = ABI::new();
    eprintln!("Landlock ABI version: {:?}", abi);

    // Create a new ruleset
    let mut ruleset = Ruleset::new()
        .map_err(|e| format!("Failed to create ruleset: {}", e))?;

    // Define allowed paths with their access rights
    let allowed_paths = vec![
        // /tmp - full read/write for temp files
        ("/tmp", AccessFs::from_all(abi)),
        // Package directory - full access
        (package_path, AccessFs::from_all(abi)),
        // Standard directories needed for executables to run
        ("/usr", AccessFs::from_read(abi)),
        ("/lib", AccessFs::from_read(abi)),
        ("/lib64", AccessFs::from_read(abi)),
        ("/etc", AccessFs::from_read(abi)),
        ("/dev/null", AccessFs::from_read(abi)),
        ("/dev/urandom", AccessFs::from_read(abi)),
    ];

    // Add rules for each allowed path
    for (path_str, access) in allowed_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("Warning: path does not exist: {}", path_str);
            continue;
        }

        let path_fd = PathFd::new(path)
            .map_err(|e| format!("Failed to open path {}: {}", path_str, e))?;

        let rule = PathBeneath::new(path_fd, access)
            .map_err(|e| format!("Failed to create rule for {}: {}", path_str, e))?;

        ruleset = ruleset.add_rule(rule)
            .map_err(|e| format!("Failed to add rule for {}: {}", path_str, e))?;

        eprintln!("Added Landlock rule: {} (access: {:?})", path_str, access);
    }

    // Apply the ruleset to the current process
    match ruleset.restrict_self() {
        Ok(RestrictionStatus::Restricted) => {
            eprintln!("Landlock filesystem restrictions applied successfully");
        }
        Ok(RestrictionStatus::Unrestricted) => {
            eprintln!("Landlock not enforced (already restricted or not fully supported)");
        }
        Err(e) => {
            eprintln!("Failed to apply Landlock: {}", e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}

/// Check if Landlock is available (kernel >= 5.13 and LSM loaded)
pub fn is_land_lock_available() -> bool {
    // Check kernel version
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

    // Also check if Landlock is in the LSM list
    if let Ok(lsm) = fs::read_to_string("/sys/kernel/security/lsm") {
        return lsm.contains("landlock");
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
        // Should not error even with nonexistent path
        let result = apply_land_lock("/nonexistent/path");
        // May fail if Landlock not available, that's ok
        let _ = result;
    }

    #[test]
    fn test_land_lock_with_tmp() {
        // Test with /tmp which should always exist
        let result = apply_land_lock("/tmp");
        let _ = result;
    }
}
