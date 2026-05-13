use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
    JobObjectExtendedLimitInformation,
    JOBOBJECT_BASIC_LIMIT_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, GetExitCodeProcess, ResumeThread, PROCESS_INFORMATION,
    STARTUPINFOW, WaitForSingleObject, CREATE_NO_WINDOW,
    CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT,
};

fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

pub fn create_job_object() -> Result<HANDLE, String> {
    let job = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };

    if job == 0 {
        return Err(format!(
            "Failed to create Job Object: {}",
            std::io::Error::last_os_error()
        ));
    }

    let extended_limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
        BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION {
            PerProcessUserTimeLimit: 0,
            PerJobUserTimeLimit: 0,
            LimitFlags: JOB_OBJECT_LIMIT_ACTIVE_PROCESS
                | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            MinimumWorkingSetSize: 0,
            MaximumWorkingSetSize: 0,
            ActiveProcessLimit: 3,
            Affinity: 0,
            PriorityClass: 0,
            SchedulingClass: 0,
        },
        IoInfo: unsafe { std::mem::zeroed() },
        ProcessMemoryLimit: 0,
        JobMemoryLimit: 0,
        PeakProcessMemoryUsed: 0,
        PeakJobMemoryUsed: 0,
    };

    let result = unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &extended_limits as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };

    if result == FALSE {
        unsafe { CloseHandle(job) };
        return Err(format!(
            "Failed to set Job Object limits: {}",
            std::io::Error::last_os_error()
        ));
    }

    Ok(job)
}

pub fn assign_process_to_job(job: HANDLE, process_handle: HANDLE) -> Result<(), String> {
    let result = unsafe { AssignProcessToJobObject(job, process_handle) };

    if result == FALSE {
        return Err(format!(
            "Failed to assign process to Job Object: {}",
            std::io::Error::last_os_error()
        ));
    }

    Ok(())
}

pub fn run_script_in_sandbox(script_path: &str) -> Result<i32, String> {
    let path = Path::new(script_path);
    if !path.exists() {
        return Err(format!("Script not found: {}", script_path));
    }

    let job = create_job_object()?;

    let cmd_line = format!("cmd.exe /c \"{}\"", script_path);
    let cmd_wstr = to_wstring(&cmd_line);

    let mut si = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        lpReserved: ptr::null_mut(),
        lpDesktop: ptr::null_mut(),
        lpTitle: ptr::null_mut(),
        dwX: 0,
        dwY: 0,
        dwXSize: 0,
        dwYSize: 0,
        dwXCountChars: 0,
        dwYCountChars: 0,
        dwFillAttribute: 0,
        dwFlags: 0,
        wShowWindow: 0,
        cbReserved2: 0,
        lpReserved2: ptr::null_mut(),
        hStdInput: 0,
        hStdOutput: 0,
        hStdError: 0,
    };

    let mut pi = PROCESS_INFORMATION {
        hProcess: 0,
        hThread: 0,
        dwProcessId: 0,
        dwThreadId: 0,
    };

    let result = unsafe {
        CreateProcessW(
            ptr::null(),
            cmd_wstr.as_ptr() as *mut u16,
            ptr::null_mut(),
            ptr::null_mut(),
            FALSE,
            CREATE_NO_WINDOW | CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT,
            ptr::null_mut(),
            ptr::null(),
            &mut si,
            &mut pi,
        )
    };

    if result == FALSE {
        unsafe { CloseHandle(job) };
        return Err(format!(
            "Failed to create process: {}",
            std::io::Error::last_os_error()
        ));
    }

    if let Err(e) = assign_process_to_job(job, pi.hProcess) {
        unsafe {
            CloseHandle(pi.hThread);
            CloseHandle(pi.hProcess);
            CloseHandle(job);
        }
        return Err(e);
    }

    let resume_result = unsafe { ResumeThread(pi.hThread) };

    unsafe { CloseHandle(pi.hThread) };

    if resume_result == u32::MAX {
        unsafe {
            CloseHandle(pi.hProcess);
            CloseHandle(job);
        }
        return Err(format!(
            "Failed to resume thread: {}",
            std::io::Error::last_os_error()
        ));
    }

    let wait_result = unsafe { WaitForSingleObject(pi.hProcess, 30000) };

    if wait_result != 0 {
        unsafe {
            CloseHandle(pi.hProcess);
            CloseHandle(job);
        }
        return Err(format!(
            "Process wait failed or timed out: result={}",
            wait_result
        ));
    }

    let mut exit_code: u32 = 0;
    let result = unsafe { GetExitCodeProcess(pi.hProcess, &mut exit_code) };

    unsafe {
        CloseHandle(pi.hProcess);
        CloseHandle(job);
    }

    if result == FALSE {
        return Err(format!(
            "Failed to get exit code: {}",
            std::io::Error::last_os_error()
        ));
    }

    Ok(exit_code as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_job_object() {
        let job = create_job_object();
        assert!(job.is_ok());
        if let Ok(j) = job {
            unsafe { CloseHandle(j) };
        }
    }

    #[test]
    fn test_run_nonexistent_script() {
        let result = run_script_in_sandbox("C:\\nonexistent\\script.bat");
        assert!(result.is_err());
    }
}
