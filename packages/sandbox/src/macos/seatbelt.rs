//! Real macOS Seatbelt implementation
//! Uses sandbox-exec with generated profiles.

use napi::{Error, Status};
use std::process::Command;
use std::fs;
use std::path::Path;

/// Apply macOS sandbox using Seatbelt
pub fn sandbox_macos(script_path: &str) -> napi::Result<()> {
    let path = Path::new(script_path);
    if !path.exists() {
        return Err(Error::new(
            Status::GenericFailure,
            "Script not found".to_string(),
        ));
    }

    let profile = generate_profile(path);
    let temp_profile = "/tmp/guardinstall-seatbelt.sb";

    // Write profile to temp file
    fs::write(temp_profile, profile)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to write profile: {}", e)))?;

    // Run: sandbox-exec -f profile script
    let output = Command::new("sandbox-exec")
        .arg("-f")
        .arg(temp_profile)
        .arg(script_path)
        .output();

    // Clean up
    let _ = fs::remove_file(temp_profile);

    match output {
        Ok(o) => {
            if o.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                Err(Error::new(
                    Status::GenericFailure,
                    format!("Script failed: {}", stderr),
                ))
            }
        }
        Err(e) => Err(Error::new(
            Status::GenericFailure,
            format!("Failed to execute sandbox-exec: {}", e),
        )),
    }
}

/// Generate a Seatbelt profile for a package
pub fn generate_profile(script_path: &Path) -> String {
    format!(
        r#"(version 1)
(deny default)
(allow file-read*)
(allow file-write*
  (subpath "/tmp")
  (subpath "{}"))
(deny network*)
(allow process-exec
  (literal "/usr/local/bin/node"))
"#,
        script_path.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_generation() {
        let profile = generate_profile(std::path::Path::new("/test/pkg"));
        assert!(profile.contains("deny network*"));
        assert!(profile.contains("/tmp"));
        assert!(profile.contains("version 1"));
    }

    #[test]
    fn test_sandbox_nonexistent() {
        let result = sandbox_macos("/nonexistent/script.sh");
        assert!(result.is_err());
    }
}
