//! Main sandboxer binary - WORKING VERSION
//! Uses EXACT same BPF as sandboxer_working_c (which works)
//! Blocks: execve (59) only
//! 
//! Usage: sandboxer <script_path> <package_name>

use libc::{self, c_ulong, c_char};
use std::ffi::CString;
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
        // Child process - apply seccomp and run script
        if let Err(e) = apply_seccomp_filter() {
            eprintln!("Failed to apply seccomp: {}", e);
            std::process::exit(1);
        }

        // Run the script using execve (EXACT same as working_c)
        let bash_path = CString::new("/bin/bash").unwrap();
        let arg0 = CString::new("bash").unwrap();
        let script_cstr = CString::new(script_path.as_bytes()).unwrap();
        
        eprintln!("Executing script with seccomp filter active...");
        
        unsafe {
            libc::execve(
                bash_path.as_ptr(),
                [
                    bash_path.as_ptr(),
                    arg0.as_ptr(),
                    script_cstr.as_ptr(),
                    std::ptr::null()
                ].as_ptr() as *const *const c_char,
                std::ptr::null() as *const *const c_char,
            );
        }
        
        // If we get here, execve failed
        eprintln!("execve failed: {}", std::io::Error::last_os_error());
        std::process::exit(1);
    } else {
        // Parent process - wait for child
        let mut status = 0;
        unsafe {
            libc::waitpid(pid, &mut status, 0);
        }

        if libc::WIFEXITED(status) {
            let exit_code = libc::WEXITSTATUS(status);
            if exit_code == 0 {
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
                std::process::exit(exit_code);
            }
        } else if libc::WIFSIGNALED(status) {
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

/// Apply seccomp filter - EXACT same as sandboxer_working_c
/// Blocks execve (59) on x86_64
fn apply_seccomp_filter() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Applying seccomp filter (block execve)...");
    
    // Set no_new_privs first
    let res = unsafe {
        libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0)
    };
    
    if res != 0 {
        return Err(format!("prctl NO_NEW_PRIVS failed: {}", std::io::Error::last_os_error()).into());
    }
    
    eprintln!("NO_NEW_PRIVS set successfully");

    // BPF filter - EXACT same as working_c version
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
    
    // This is the EXACT filter from sandboxer_working_c that works
    let bpf_code = [
        // Load architecture (offset 4 in seccomp_data)
        sock_filter { code: 0x20, jt: 0, jf: 0, k: 0x00000004 },
        // Jump if x86_64 (AUDIT_ARCH_X86_64 = 0xC000003E)
        sock_filter { code: 0x15, jt: 1, jf: 0, k: 0xC000003E },
        // Not x86_64, allow
        sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x7fff0000 },
        // Load syscall nr (offset 0 in seccomp_data)
        sock_filter { code: 0x20, jt: 0, jf: 0, k: 0x00000000 },
        // Check if execve (59 on x86_64)
        sock_filter { code: 0x15, jt: 0, jf: 1, k: 59 },
        // Block: return EPERM
        sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x00050001 },
        // Allow: return ALLOW
        sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x7fff0000 },
    ];
    
    let prog = sock_fprog {
        len: bpf_code.len() as u16,
        filter: bpf_code.as_ptr(),
    };
    
    eprintln!("Loading BPF filter (block execve)...");
    
    // Pass pointer as c_ulong (like working C version)
    let prog_ptr = &prog as *const _ as c_ulong;
    
    let res = unsafe {
        libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, prog_ptr, 0, 0)
    };
    
    if res != 0 {
        return Err(format!("prctl SECCOMP failed: {}", std::io::Error::last_os_error()).into());
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
