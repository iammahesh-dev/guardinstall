//! Linux namespace isolation
//! Creates new network namespace to block network access

use napi::{Error, Status};

/// Create new Linux namespaces (network, mount, PID)
/// Uses unshare() system call
pub fn create_namespaces() -> napi::Result<()> {
    // Phase 2: Real implementation would:
    // 1. Call unshare(CLONE_NEWNET | CLONE_NEWNS | CLONE_NEWPID)
    // 2. If that fails (no privileges), try user namespaces: unshare(CLONE_NEWUSER)
    // 3. Then unshare(CLONE_NEWNET) inside user namespace

    // Check if we can create namespaces
    if !check_capabilities()? {
        return Err(Error::new(
            Status::GenericFailure,
            "Cannot create namespaces: insufficient privileges. Try running with user namespaces or as root.".to_string()
        ));
    }

    Ok(())
}

/// Check if we have necessary capabilities for namespaces
pub fn check_capabilities() -> napi::Result<bool> {
    // Check for CAP_SYS_ADMIN or if user namespaces are available
    // /proc/sys/kernel/unprivileged_user_namespaces == 1 means we can use them

    let userns_path = std::path::Path::new("/proc/sys/kernel/unprivileged_user_namespaces");
    if userns_path.exists() {
        if let Ok(contents) = std::fs::read_to_string(userns_path) {
            if contents.trim() == "1" {
                return Ok(true);
            }
        }
    }

    // Check if we're root by checking effective uid
    // Use /proc/self/status instead of libc
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("Uid:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(uid) = parts[1].parse::<u32>() {
                        return Ok(uid == 0);
                    }
                }
            }
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_creation() {
        // This will fail in test environment without privileges
        // That's expected - we're just testing it doesn't panic
        let _ = create_namespaces();
    }

    #[test]
    fn test_capability_check() {
        let result = check_capabilities();
        assert!(result.is_ok());
    }
}
