//! Event emission from sandbox to parent process
//! Uses Unix socket (via stdout) for structured JSON event streaming

use napi::{Error, Status};
use serde::Serialize;
use serde_json::json;
use std::io::{self, Write};

#[derive(Debug, Clone, Serialize)]
pub struct SandboxEvent {
    pub event_type: String,
    pub package: String,
    pub syscall: Option<String>,
    pub args: Option<serde_json::Value>,
    pub path: Option<String>,
    pub action: String,
    pub timestamp_ns: u64,
}

impl SandboxEvent {
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "event": self.event_type,
            "package": self.package,
            "syscall": self.syscall,
            "args": self.args,
            "path": self.path,
            "action": self.action,
            "timestamp_ns": self.timestamp_ns
        })
    }
}

/// Emit event to parent process via stdout (JSON lines)
pub fn emit_event(event: &SandboxEvent) -> Result<(), Error> {
    let json_str = serde_json::to_string(event).map_err(|e| {
        Error::new(Status::GenericFailure, format!("JSON serialization failed: {}", e))
    })?;

    io::stdout().write_all(json_str.as_bytes()).map_err(|e| {
        Error::new(Status::GenericFailure, format!("Failed to write event: {}", e))
    })?;
    io::stdout().write_all(b"\n").map_err(|e| {
        Error::new(Status::GenericFailure, format!("Failed to write newline: {}", e))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = SandboxEvent {
            event_type: "syscall_intercepted".to_string(),
            package: "test@1.0.0".to_string(),
            syscall: Some("execve".to_string()),
            args: Some(serde_json::json!(["/bin/sh"])),
            path: None,
            action: "blocked".to_string(),
            timestamp_ns: 1746547200000,
        };

        let json = event.to_json();
        assert_eq!(json["event"], "syscall_intercepted");
        assert_eq!(json["package"], "test@1.0.0");
    }

    #[test]
    fn test_emit_event() {
        let event = SandboxEvent {
            event_type: "test".to_string(),
            package: "test@1.0.0".to_string(),
            syscall: None,
            args: None,
            path: None,
            action: "logged".to_string(),
            timestamp_ns: 1746547200000,
        };

        // Should not panic
        let result = emit_event(&event);
        assert!(result.is_ok());
    }
}
