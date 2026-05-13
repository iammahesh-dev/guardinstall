pub mod job_objects;

use napi::{Error, Status};

pub fn sandbox_windows(script_path: &str) -> napi::Result<()> {
    job_objects::run_script_in_sandbox(script_path).map_err(|e| {
        Error::new(Status::GenericFailure, e)
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_nonexistent() {
        let result = sandbox_windows("C:\\nonexistent\\script.bat");
        assert!(result.is_err());
    }
}
