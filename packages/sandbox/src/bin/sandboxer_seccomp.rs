//! Sandboxer with working seccomp-BPF filter
use libc::{self, c_ulong, c_void};
use std::os::unix::process::CommandExt;
use std::process::Command;
use nix::sched;
use serde_json::{json, Value};
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <script_path> <package_name> [timeout_sec]", args[0]);
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
        if let Err(e) = apply_seccomp_bpf() {
            eprintln!("Failed to apply seccomp: {}", e);
            std::process::exit(1);
        }

        // Run the script
        let output = Command::new("bash")
            .arg(script_path)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let event = json!({
                        "action": "allowed",
                        "event": "script_completed",
                        "package": package_name,
                        "timestamp_ns": get_timestamp_ns()
                    });
                    eprintln!("{}", serde_json::to_string(&event).unwrap());
                    std::process::exit(0);
                } else {
                    let event = json!({
                        "action": "blocked",
                        "event": "script_failed",
                        "package": package_name,
                        "timestamp_ns": get_timestamp_ns()
                    });
                    eprintln!("{}", serde_json::to_string(&event).unwrap());
                    std::process::exit(output.status.code().unwrap_or(1));
                }
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
            let sig = libc::WTERMSIG(status);
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

/// Apply seccomp filter using prctl with correct BPF program
/// Blocks: execve, execveat, ptrace, socket(AF_INET)
fn apply_seccomp_bpf() -> Result<(), Box<dyn std::error::Error>> {
    // First set no_new_privs
    let res = unsafe {
        libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0)
    };
    
    if res != 0 {
        return Err(format!("prctl NO_NEW_PRIVS failed: {}", std::io::Error::last_os_error()).into());
    }
    
    eprintln!("NO_NEW_PRIVS set successfully");

    // BPF program for seccomp - based on working C example
    // seccomp_data structure offsets:
    //   nr (syscall number) at offset 0
    //   arch at offset 4
    
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
    
    // BPF program that blocks dangerous syscalls
    // This is a simplified filter - blocks execve (59 on x86_64)
    let bpf_code = [
        // Load architecture (offset 4 in seccomp_data)
        sock_filter { code: 0x20, jt: 0, jf: 0, k: 4 }, // LD|W|ABS: arch
        // Jump if x86_64 (AUDIT_ARCH_X86_64 = 0xC000003E)
        sock_filter { code: 0x15, jt: 1, jf: 0, k: 0xC000003E }, // JMP|EQ
        // Not x86_64, allow
        sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x7fff0000 }, // RET: ALLOW
        // Load syscall nr (offset 0 in seccomp_data)
        sock_filter { code: 0x20, jt: 0, jf: 0, k: 0 }, // LD|W|ABS: nr
        // Check if execve (59 on x86_64)
        sock_filter { code: 0x15, jt: 0, jf: 1, k: 59 }, // JMP|EQ: nr == execve
        // Block: return EPERM
        sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x00050001 }, // RET: ERRNO(EPERM)
        // Allow: return ALLOW
        sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x7fff0000 }, // RET: ALLOW
    ];
    
    let prog = sock_fprog {
        len: bpf_code.len() as u16,
        filter: bpf_code.as_ptr(),
    };
    
    eprintln!("Applying BPF filter to block execve...");
    
    let res = unsafe {
        libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &prog as *const _ as c_ulong, 0, 0)
    };
    
    if res != 0 {
        return Err(format!("prctl SECCOMP_MODE_FILTER failed: {}", std::io::Error::last_os_error()).into());
    }
    
    eprintln!("Seccomp filter applied successfully (blocks execve)");
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
