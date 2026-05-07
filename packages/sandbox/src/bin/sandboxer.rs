//! Standalone sandboxer binary - WORKING VERSION
//! Uses seccomp-BPF (allow all) and fork+exec

use std::env;
use std::io::{self, Write};
use std::process;
use serde_json::json;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <script_path> [package_name]", args[0]);
        process::exit(1);
    }

    let script_path = &args[1];
    let package_name = if args.len() > 2 { &args[2] } else { "unknown" };

    eprintln!("Running script: {}", script_path);

    let pid = unsafe { libc::fork() };
    
    if pid < 0 {
        eprintln!("Fork failed: {}", std::io::Error::last_os_error());
        emit_event("script_error", package_name, "error");
        process::exit(1);
    } else if pid == 0 {
        // Child process
        
        // Apply seccomp filter (allow all for now)
        #[cfg(target_os = "linux")]
        {
            if let Err(e) = apply_seccomp_allow_all() {
                eprintln!("Failed to apply seccomp: {}", e);
                process::exit(1);
            }
        }
        
        // Exec the script
        let script_cstr = std::ffi::CString::new(script_path.as_bytes()).unwrap();
        unsafe {
            libc::execl(
                "/bin/sh\0".as_ptr() as *const libc::c_char,
                "sh\0".as_ptr() as *const libc::c_char,
                script_cstr.as_ptr(),
                0 as *const libc::c_char
            );
            eprintln!("execl failed: {}", std::io::Error::last_os_error());
            process::exit(1);
        }
    } else {
        // Parent process
        let mut status: i32 = 0;
        let ret = unsafe { libc::waitpid(pid, &mut status as *mut i32, 0) };
        
        if ret < 0 {
            eprintln!("waitpid failed: {}", std::io::Error::last_os_error());
            emit_event("script_error", package_name, "error");
            process::exit(1);
        }
        
        if libc::WIFEXITED(status) {
            let exit_code = libc::WEXITSTATUS(status);
            if exit_code == 0 {
                emit_event("script_completed", package_name, "allowed");
                process::exit(0);
            } else {
                eprintln!("Script exited with code: {}", exit_code);
                emit_event("script_failed", package_name, "blocked");
                process::exit(exit_code);
            }
        } else if libc::WIFSIGNALED(status) {
            let sig = libc::WTERMSIG(status);
            eprintln!("Script terminated by signal: {}", sig);
            emit_event("script_failed", package_name, "blocked");
            process::exit(1);
        } else {
            emit_event("script_failed", package_name, "blocked");
            process::exit(1);
        }
    }
}

#[cfg(target_os = "linux")]
fn apply_seccomp_allow_all() -> Result<(), Box<dyn std::error::Error>> {
    // BPF program that allows all syscalls
    let bpf_code: [libc::sock_filter; 1] = [
        libc::sock_filter {
            code: (libc::BPF_RET | libc::BPF_K) as u16,
            jt: 0,
            jf: 0,
            k: libc::SECCOMP_RET_ALLOW,
        },
    ];

    let prog = libc::sock_fprog {
        len: bpf_code.len() as u16,
        filter: bpf_code.as_ptr() as *mut libc::sock_filter,
    };

    // Set no_new_privs
    let ret = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if ret != 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    // Apply seccomp filter
    let ret = unsafe { 
        libc::prctl(
            libc::PR_SET_SECCOMP, 
            libc::SECCOMP_MODE_FILTER, 
            &prog as *const _ as usize, 
            0, 
            0
        ) 
    };
    if ret != 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    eprintln!("Seccomp filter applied (allow all)");
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn apply_seccomp_allow_all() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn emit_event(event_type: &str, package: &str, action: &str) {
    let event = json!({
        "event": event_type,
        "package": package,
        "action": action,
        "timestamp_ns": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    });
    let _ = writeln!(io::stderr(), "{}", serde_json::to_string(&event).unwrap());
}
