//! Sandboxer that uses seccompiler correctly
use libc::{self, c_ulong};
use std::os::unix::process::CommandExt;
use std::process::Command;
use serde_json::{json};
use std::io::Write;
use std::collections::BTreeMap;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <script_path> <package_name>", args[0]);
        std::process::exit(1);
    }

    let script_path = &args[1];
    let package_name = &args[2];

    eprintln!("Running script: {}", script_path);
    eprintln!("Package: {}", package_name);

    // Fork the process
    let pid = unsafe { libc::fork() };
    
    if pid < 0 {
        eprintln!("Fork failed: {}", std::io::Error::last_os_error());
        std::process::exit(1);
    }

    if pid == 0 {
        // Child process - apply seccomp and run script
        if let Err(e) = apply_seccomp_filter() {
            eprintln!("Failed to apply seccomp: {}", e);
            std::process::exit(1);
        }

        // Run the script
        let status = Command::new("bash")
            .arg(script_path)
            .status();

        match status {
            Ok(s) if s.success() => {
                let event = json!({
                    "action": "allowed",
                    "event": "script_completed",
                    "package": package_name,
                    "timestamp_ns": get_timestamp_ns()
                });
                eprintln!("{}", serde_json::to_string(&event).unwrap());
                std::process::exit(0);
            }
            Ok(s) => {
                let event = json!({
                    "action": "blocked",
                    "event": "script_failed",
                    "package": package_name,
                    "timestamp_ns": get_timestamp_ns()
                });
                eprintln!("{}", serde_json::to_string(&event).unwrap());
                std::process::exit(s.code().unwrap_or(1));
            }
            Err(e) => {
                let event = json!({
                    "action": "blocked",
                    "event": "script_failed",
                    "package": package_name,
                    "timestamp_ns": get_timestamp_ns()
                });
                eprintln!("{}", serde_json::to_string(&event).unwrap());
                std::process::exit(1);
            }
        }
    } else {
        // Parent process - wait for child
        let mut status = 0;
        unsafe {
            libc::waitpid(pid, &mut status, 0);
        }

        if libc::WIFSIGNALED(status) {
            let event = json!({
                "action": "blocked",
                "event": "script_failed",
                "package": package_name,
                "timestamp_ns": get_timestamp_ns()
            });
            eprintln!("{}", serde_json::to_string(&event).unwrap());
        }
    }
}

/// Apply seccomp filter using seccompiler with correct API
fn apply_seccomp_filter() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Building seccomp filter...");
    
    use seccompiler::{SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition, SeccompFilter, SeccompRule, BpfProgram};
    
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();

    // Block execve (spawning new processes) - unconditional block
    rules.insert(libc::SYS_execve as i64, vec![]);
    eprintln!("Added rule: block execve");

    // Block execveat - unconditional block
    rules.insert(libc::SYS_execveat as i64, vec![]);
    eprintln!("Added rule: block execveat");

    // Block ptrace (anti-sandbox detection) - unconditional block
    rules.insert(libc::SYS_ptrace as i64, vec![]);
    eprintln!("Added rule: block ptrace");

    // Block socket(AF_INET) - only when domain == AF_INET
    let socket_condition = SeccompCondition::new(
        0,  // arg0 (domain)
        SeccompCmpArgLen::Dword,
        SeccompCmpOp::Eq,
        libc::AF_INET as u64,
    ).map_err(|e| format!("Failed to create condition: {}", e))?;
    
    let socket_rule = SeccompRule::new(vec![socket_condition])
        .map_err(|e| format!("Failed to create rule: {}", e))?;
    
    rules.insert(libc::SYS_socket as i64, vec![socket_rule]);
    eprintln!("Added rule: block socket(AF_INET)");

    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Errno(libc::EPERM as u32),  // Action when rule matches
        SeccompAction::Allow,                       // Default action (allow others)
        seccompiler::TargetArch::x86_64,          // Target architecture
    ).map_err(|e| format!("Failed to create filter: {}", e))?;

    let bpf_program: BpfProgram = filter.try_into()
        .map_err(|e| format!("Failed to convert to BPF: {}", e))?;
    
    eprintln!("Applying BPF filter...");
    
    seccompiler::apply_filter(&bpf_program)
        .map_err(|e| format!("Failed to apply filter: {}", e))?;
    
    eprintln!("Seccomp filter applied successfully (blocks execve, execveat, ptrace, socket)");
    Ok(())
}

fn get_timestamp_ns() -> u64 {
    let mut timespec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe {
        libc::clock_gettime(libc::CLOCK_REALTIME, &mut timespec);
    }
    (timespec.tv_sec as u64) * 1_000_000_000 + (timespec.tv_nsec as u64)
}
