//! EXACT same program as working C version
//! This should work if the BPF and prctl usage is correct
use libc::{self, c_ulong};
use std::ffi::CString;

fn main() {
    // Set no_new_privs
    let res = unsafe {
        libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0)
    };
    if res != 0 {
        eprintln!("NO_NEW_PRIVS failed: {}", std::io::Error::last_os_error());
        return;
    }
    eprintln!("NO_NEW_PRIVS set");
    
    // BPF filter - EXACT same as C version
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
        // Load architecture (offset 4)
        sock_filter { code: 0x20, jt: 0, jf: 0, k: 0x00000004 },
        // Jump if x86_64
        sock_filter { code: 0x15, jt: 1, jf: 0, k: 0xC000003E },
        // Not x86_64, allow
        sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x7fff0000 },
        // Load syscall nr (offset 0)
        sock_filter { code: 0x20, jt: 0, jf: 0, k: 0x00000000 },
        // Check if execve (59)
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
    
    eprintln!("Applying BPF filter (block execve)...");
    
    // Try multiple ways to pass the pointer to prctl
    // The C version does: prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog, 0, 0)
    // where &prog is a pointer to sock_fprog
    
    // Method 1: Cast to c_ulong (like C's implicit pointer to integer)
    let prog_ptr = &prog as *const _ as c_ulong;
    eprintln!("prog_ptr (c_ulong) = 0x{:x}", prog_ptr);
    
    let res = unsafe {
        libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, 
                   prog_ptr, 0, 0)
    };
    
    if res != 0 {
        eprintln!("Method 1 failed: {}", std::io::Error::last_os_error());
        
        // Method 2: Cast to i64
        let prog_ptr2 = &prog as *const _ as i64;
        eprintln!("Trying method 2 (i64)...");
        
        let res2 = unsafe {
            libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, 
                       prog_ptr2, 0, 0)
        };
        
        if res2 != 0 {
            eprintln!("Method 2 failed: {}", std::io::Error::last_os_error());
            return;
        }
    }
    
    eprintln!("Seccomp filter applied. Testing execve...");
    
    // Try execve using syscall (like C version)
    let echo_path = CString::new("/bin/echo").unwrap();
    let arg0 = CString::new("echo").unwrap();
    let arg1 = CString::new("Hello from Rust!").unwrap();
    
    unsafe {
        libc::syscall(
            libc::SYS_execve,
            echo_path.as_ptr(),
            [echo_path.as_ptr(), arg0.as_ptr(), arg1.as_ptr(), std::ptr::null()].as_ptr(),
            std::ptr::null() as *const *const libc::c_char,
        );
    }
    
    eprintln!("execve failed (should be blocked): {}", std::io::Error::last_os_error());
}
