//! Event emission from sandbox to parent process
//! Uses Unix socket for structured JSON event streaming

use napi::{Error, Status};
use serde_json::json;

#[derive(Debug, Clone)]
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

/// Emit event to parent process via Unix socket
pub fn emit_event(event: &SandboxEvent) -> Result<(), Error> {
    // Placeholder: will use Unix socket to stream JSON to parent
    println!("{}", serde_json::to_string(event).unwrap());
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
}
