use serde_json::json;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

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
    let _no_seccomp = args.len() > 3 && args[3] == "--no-seccomp";

    #[cfg(target_os = "windows")]
    {
        run_windows(script_path, package_name);
    }

    #[cfg(target_os = "linux")]
    {
        run_linux(script_path, package_name, _no_seccomp);
    }

    #[cfg(target_os = "macos")]
    {
        run_macos(script_path, package_name);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        eprintln!("Unsupported platform: {}", std::env::consts::OS);
        process::exit(1);
    }
}

#[cfg(target_os = "windows")]
fn run_windows(script_path: &str, package_name: &str) {
    eprintln!("Running script: {}", script_path);
    eprintln!("Package: {}", package_name);

    match guardinstall_sandbox::windows::job_objects::run_script_in_sandbox(script_path) {
        Ok(exit_code) => {
            let event = json!({
                "action": "allowed",
                "event": "script_completed",
                "package": package_name,
                "exit_code": exit_code,
                "timestamp_ns": get_timestamp_ns()
            });
            eprintln!("{}", serde_json::to_string(&event).unwrap());
            process::exit(exit_code);
        }
        Err(e) => {
            let event = json!({
                "action": "blocked",
                "event": "script_blocked",
                "package": package_name,
                "error": e,
                "timestamp_ns": get_timestamp_ns()
            });
            eprintln!("{}", serde_json::to_string(&event).unwrap());
            process::exit(1);
        }
    }
}

#[cfg(target_os = "linux")]
fn run_linux(script_path: &str, package_name: &str, no_seccomp: bool) {
    use libc::{self, SYS_execve};

    eprintln!("Running script: {}", script_path);
    eprintln!("Package: {}", package_name);
    if no_seccomp {
        eprintln!("Seccomp disabled (relaxed mode for verified packages)");
    }

    let pid = unsafe { libc::fork() };

    if pid < 0 {
        eprintln!("Fork failed: {}", std::io::Error::last_os_error());
        process::exit(1);
    }

    if pid == 0 {
        landlock::apply_land_lock(script_path).unwrap_or_else(|e| {
            eprintln!("Warning: Landlock not applied: {}", e);
        });

        if !no_seccomp {
            apply_seccomp();
        }

        let bash_path = "/bin/bash\0".as_ptr() as *const i8;
        let script_path_c = format!("{}\0", script_path);
        let script_ptr = script_path_c.as_ptr() as *const i8;
        let argv: [*const i8; 3] = [bash_path, script_ptr, std::ptr::null()];
        let envp: [*const i8; 1] = [std::ptr::null()];

        let _ret = unsafe { libc::syscall(SYS_execve, bash_path, argv.as_ptr(), envp.as_ptr()) };

        eprintln!("execve failed: {}", std::io::Error::last_os_error());
        process::exit(1);
    } else {
        let mut status = 0;
        unsafe {
            libc::waitpid(pid, &mut status, 0);
        }

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

#[cfg(target_os = "linux")]
fn apply_seccomp() {
    eprintln!("Applying seccomp-BPF filter (kills on socket() syscall)...");

    const BPF_INSTRUCTIONS: [libc::sock_filter; 6] = [
        libc::sock_filter { code: 0x20, jt: 0, jf: 0, k: 0x00000004 },
        libc::sock_filter { code: 0x15, jt: 0, jf: 3, k: 0xc000003e },
        libc::sock_filter { code: 0x20, jt: 0, jf: 0, k: 0x00000000 },
        libc::sock_filter { code: 0x15, jt: 1, jf: 0, k: 41 },
        libc::sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x7fff0000 },
        libc::sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x00000000 },
    ];

    let prog = libc::sock_fprog {
        len: BPF_INSTRUCTIONS.len() as u16,
        filter: BPF_INSTRUCTIONS.as_ptr() as *mut libc::sock_filter,
    };

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

#[cfg(target_os = "macos")]
fn run_macos(script_path: &str, package_name: &str) {
    use std::process::Command;
    use std::fs;

    eprintln!("Running script: {}", script_path);
    eprintln!("Package: {}", package_name);

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
        script_path
    );

    let temp_profile = "/tmp/guardinstall-seatbelt.sb";
    if let Err(e) = fs::write(temp_profile, &profile) {
        eprintln!("Failed to write seatbelt profile: {}", e);
        process::exit(1);
    }

    let output = Command::new("sandbox-exec")
        .arg("-f")
        .arg(temp_profile)
        .arg(script_path)
        .output();

    let _ = fs::remove_file(temp_profile);

    match output {
        Ok(o) => {
            let exit_code = o.status.code().unwrap_or(1);
            if o.status.success() {
                let event = json!({
                    "action": "allowed",
                    "event": "script_completed",
                    "package": package_name,
                    "exit_code": exit_code,
                    "timestamp_ns": get_timestamp_ns()
                });
                eprintln!("{}", serde_json::to_string(&event).unwrap());
                process::exit(exit_code);
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let event = json!({
                    "action": "blocked",
                    "event": "script_blocked",
                    "package": package_name,
                    "error": stderr.to_string(),
                    "timestamp_ns": get_timestamp_ns()
                });
                eprintln!("{}", serde_json::to_string(&event).unwrap());
                process::exit(exit_code);
            }
        }
        Err(e) => {
            let event = json!({
                "action": "blocked",
                "event": "script_blocked",
                "package": package_name,
                "error": e.to_string(),
                "timestamp_ns": get_timestamp_ns()
            });
            eprintln!("{}", serde_json::to_string(&event).unwrap());
            process::exit(1);
        }
    }
}

fn get_timestamp_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
