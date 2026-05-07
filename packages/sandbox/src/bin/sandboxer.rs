//! Main sandboxer binary - WORKING VERSION
//! Uses seccomp-BPF to KILL on socket() syscall (network access)
//! Uses Landlock to block filesystem access to sensitive files
//! Does NOT block execve - lets bash run scripts
//! 
//! Usage: sandboxer <script_path> <package_name>

use libc::{self, SYS_execve};
use serde_json::{json};
use std::process;

/// BPF instructions for seccomp filter
/// KILLS process on ANY socket syscall (41) - blocks ALL socket families
/// Allows everything else including execve (59)
const BPF_INSTRUCTIONS: [libc::sock_filter; 6] = [
    // Load architecture (A = seccomp_data.arch)
    libc::sock_filter { code: 0x20, jt: 0, jf: 0, k: 0x00000004 },
    // Check if x86-64 architecture (A == 0xc000003e)
    libc::sock_filter { code: 0x15, jt: 0, jf: 3, k: 0xc000003e },
    // Load syscall number (A = seccomp_data.nr)
    libc::sock_filter { code: 0x20, jt: 0, jf: 0, k: 0x00000000 },
    // Check if syscall == socket (41), if true jump to block
    libc::sock_filter { code: 0x15, jt: 1, jf: 0, k: 41 },
    // Not socket, allow (SECCOMP_RET_ALLOW = 0x7fff0000)
    libc::sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x7fff0000 },
    // Is socket, KILL process (SECCOMP_RET_KILL = 0x00000000)
    libc::sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x00000000 },
];

// Declare landlock module (from bin/landlock.rs)
#[cfg(target_os = "linux")]
mod landlock;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <script_path> <package_name> [--no-seccomp]", args[0]);
        process::exit(1);
    }

    let script_path = &args[1];
    let package_name = &args[2];
    let no_seccomp = args.len() > 3 && args[3] == "--no-seccomp";

    eprintln!("Running script: {}", script_path);
    eprintln!("Package: {}", package_name);
    if no_seccomp {
        eprintln!("Seccomp disabled (relaxed mode for verified packages)");
    }

    // Fork the process
    let pid = unsafe { libc::fork() };
    
    if pid < 0 {
        eprintln!("Fork failed: {}", std::io::Error::last_os_error());
        process::exit(1);
    }

    if pid == 0 {
        // Child process - apply restrictions and run script
        #[cfg(target_os = "linux")]
        {
            landlock::apply_land_lock(script_path).unwrap_or_else(|e| {
                eprintln!("Warning: Landlock not applied: {}", e);
            });
        }
        
        // Only apply seccomp if not in relaxed mode
        if !no_seccomp {
            apply_seccomp();
        }
        
        // Use syscall(SYS_execve) directly (not Command::new())
        let bash_path = "/bin/bash\0".as_ptr() as *const i8;
        let script_path_c = format!("{}\0", script_path);
        let script_ptr = script_path_c.as_ptr() as *const i8;
        
        // execve("/bin/bash", ["/bin/bash", script_path, NULL], environ)
        let argv: [*const i8; 3] = [bash_path, script_ptr, std::ptr::null()];
        let envp: [*const i8; 1] = [std::ptr::null()];
        
        let _ret = unsafe { libc::syscall(SYS_execve, bash_path, argv.as_ptr(), envp.as_ptr()) };
        
        // If we get here, execve failed
        eprintln!("execve failed: {}", std::io::Error::last_os_error());
        process::exit(1);
    } else {
        // Parent process - wait for child
        let mut status = 0;
        unsafe {
            libc::waitpid(pid, &mut status, 0);
        }

        // Check if process was killed (by seccomp)
        if libc::WIFSIGNALED(status) {
            let sig = libc::WTERMSIG(status);
            let event = json!({
                "action": "blocked",
                "event": "script_blocked",
                "package": package_name,
                "signal": sig,
                "timestamp_ns": get_timestamp_ns()
            });
            eprintln!("{}", serde_json::to_string(&event).unwrap());
        } else if libc::WIFEXITED(status) {
            let exit_code = libc::WEXITSTATUS(status);
            let event = json!({
                "action": "allowed",
                "event": "script_completed",
                "package": package_name,
                "exit_code": exit_code,
                "timestamp_ns": get_timestamp_ns()
            });
            eprintln!("{}", serde_json::to_string(&event).unwrap());
        }
    }
}

/// Apply seccomp-BPF filter to KILL on socket() syscall
fn apply_seccomp() {
    eprintln!("Applying seccomp-BPF filter (kills on socket() syscall)...");
    
    let prog = libc::sock_fprog {
        len: BPF_INSTRUCTIONS.len() as u16,
        filter: BPF_INSTRUCTIONS.as_ptr() as *mut libc::sock_filter,
    };

    // Load the BPF program into the kernel
    let ret = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if ret != 0 {
        eprintln!("prctl(PR_SET_NO_NEW_PRIVS) failed: {}", std::io::Error::last_os_error());
        process::exit(1);
    }

    let ret = unsafe { libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &prog) };
    if ret != 0 {
        eprintln!("prctl(PR_SET_SECCOMP) failed: {}", std::io::Error::last_os_error());
        process::exit(1);
    }
    
    eprintln!("Seccomp-BPF filter applied (kills on socket() syscall)");
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
