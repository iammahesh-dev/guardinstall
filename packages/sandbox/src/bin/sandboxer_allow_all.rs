//! Sandboxer that allows ALL syscalls (BPF: just return ALLOW)
use libc::{self, c_ulong};
use std::os::unix::process::CommandExt;
use std::process::Command;
use serde_json::{json};
use std::io::Write;

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
        // Child process - apply seccomp (allow all) and run script
        if let Err(e) = apply_seccomp_allow_all() {
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

/// Apply seccomp filter that allows ALL syscalls (for testing)
fn apply_seccomp_allow_all() -> Result<(), Box<dyn std::error::Error>> {
    // First set no_new_privs
    let res = unsafe {
        libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0)
    };
    
    if res != 0 {
        return Err(format!("prctl NO_NEW_PRIVS failed: {}", std::io::Error::last_os_error()).into());
    }
    
    eprintln!("NO_NEW_PRIVS set successfully");

    // BPF program: just return ALLOW
    #[repr(C)]
    struct sock_filter {
        code: u16,
        jt: u8,
        jf: u8,
        k: u32,
    }
    
    #[repr(C)]
    struct sock_fprog {
        len: u16,
        filter: *const sock_filter,
    }
    
    let bpf_code = [
        sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x7fff0000 }, // RET: ALLOW
    ];
    
    let prog = sock_fprog {
        len: bpf_code.len() as u16,
        filter: bpf_code.as_ptr(),
    };
    
    eprintln!("Applying BPF filter (ALLOW ALL)...");
    
    let res = unsafe {
        libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &prog as *const _ as c_ulong, 0, 0)
    };
    
    if res != 0 {
        return Err(format!("prctl SECCOMP_MODE_FILTER failed: {}", std::io::Error::last_os_error()).into());
    }
    
    eprintln!("Seccomp filter applied successfully (ALLOW ALL)");
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
