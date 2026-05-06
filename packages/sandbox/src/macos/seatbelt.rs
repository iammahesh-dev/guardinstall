//! Real macOS Seatbelt (sandbox-exec) implementation
//! Generates sandbox profiles and executes scripts under Seatbelt

use napi::{Error, Status};
use std::path::Path;
use std::process::Command;

/// Generate a Seatbelt profile for a package
pub fn generate_profile(script_path: &Path) -> Result<String, Error> {
    let profile = format!(
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
    );

    Ok(profile)
}

/// Run a command under Seatbelt sandbox
pub fn run_sandboxed(script_path: &str, profile: &str) -> Result<(), Error> {
    // Write profile to temp file
    let temp_profile = "/tmp/guardinstall-seatbelt-profile.sb";
    if let Err(e) = std::fs::write(temp_profile, profile) {
        return Err(Error::new(Status::GenericFailure, format!("Failed to write profile: {}", e)));
    }

    // Run: sandbox-exec -f profile script
    let output = Command::new("sandbox-exec")
        .arg("-f")
        .arg(temp_profile)
        .arg(script_path)
        .output();

    // Clean up temp file
    let _ = std::fs::remove_file(temp_profile);

    match output {
        Ok(o) => {
            if o.status.success() {
                Ok(())
            } else {
                Err(Error::new(
                    Status::GenericFailure,
                    format!("Script failed: {}", String::from_utf8_lossy(&o.stderr))
                ))
            }
        }
        Err(e) => Err(Error::new(
            Status::GenericFailure,
            format!("Failed to execute sandbox-exec: {}", e)
        )),
    }
}

/// Apply macOS sandbox to current process
pub fn sandbox_macos(script_path: &str) -> napi::Result<()> {
    let path = Path::new(script_path);
    if !path.exists() {
        return Err(Error::new(
            napi::Status::GenericFailure,
            "Script not found".to_string(),
        ));
    }

    let profile = generate_profile(path)?;
    run_sandboxed(script_path, &profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_generation() {
        let profile = generate_profile(Path::new("/test/package"));
        assert!(profile.is_ok());
        let p = profile.unwrap();
        assert!(p.contains("deny network*"));
        assert!(p.contains("/tmp"));
    }

    #[test]
    fn test_sandbox_nonexistent() {
        let result = sandbox_macos("/nonexistent/script.sh");
        assert!(result.is_err());
    }
}
