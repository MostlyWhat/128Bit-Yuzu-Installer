//! Natives/platform specific interactions.

/// Basic definition of some running process.
#[derive(Debug)]
pub struct Process {
    pub pid : usize,
    pub name : String
}

#[cfg(windows)]
mod natives {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    const PROCESS_LEN: usize = 10192;

    use std::ffi::CString;

    use logging::LoggingErrors;

    use std::env;
    use std::process::Command;

    use winapi::shared::minwindef::{DWORD, FALSE, MAX_PATH};
    use winapi::um::winnt::{
        HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, PROCESS_VM_READ,
    };
    use winapi::um::processthreadsapi::{OpenProcess};
    use winapi::um::psapi::{
        K32EnumProcesses,
        EnumProcessModulesEx, GetModuleFileNameExW, LIST_MODULES_ALL,
    };

    extern "C" {
        pub fn saveShortcut(
            shortcutPath: *const ::std::os::raw::c_char,
            description: *const ::std::os::raw::c_char,
            path: *const ::std::os::raw::c_char,
            args: *const ::std::os::raw::c_char,
            workingDir: *const ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int;
    }

    // Needed here for Windows interop
    #[allow(unsafe_code)]
    pub fn create_shortcut(
        name: &str,
        description: &str,
        target: &str,
        args: &str,
        working_dir: &str,
    ) -> Result<String, String> {
        let source_file = format!(
            "{}\\Microsoft\\Windows\\Start Menu\\Programs\\{}.lnk",
            env::var("APPDATA").log_expect("APPDATA is bad, apparently"),
            name
        );

        info!("Generating shortcut @ {:?}", source_file);

        let native_target_dir = CString::new(source_file.clone())
            .log_expect("Error while converting to C-style string");
        let native_description =
            CString::new(description).log_expect("Error while converting to C-style string");
        let native_target =
            CString::new(target).log_expect("Error while converting to C-style string");
        let native_args = CString::new(args).log_expect("Error while converting to C-style string");
        let native_working_dir =
            CString::new(working_dir).log_expect("Error while converting to C-style string");

        let shortcutResult = unsafe {
            saveShortcut(
                native_target_dir.as_ptr(),
                native_description.as_ptr(),
                native_target.as_ptr(),
                native_args.as_ptr(),
                native_working_dir.as_ptr(),
            )
        };

        match shortcutResult {
            0 => Ok(source_file),
            _ => Err(format!(
                "Windows gave bad result while creating shortcut: {:?}",
                shortcutResult
            )),
        }
    }

    /// Cleans up the installer
    pub fn burn_on_exit(app_name: &str) {
        let current_exe = env::current_exe().log_expect("Current executable could not be found");
        let path = current_exe
            .parent()
            .log_expect("Parent directory of executable could not be found");

        // Need a cmd workaround here.
        let tool = path.join("maintenancetool.exe");
        let tool = tool
            .to_str()
            .log_expect("Unable to convert tool path to string")
            .replace(" ", "\\ ");

        let log = path.join(format!("{}_installer.log", app_name));
        let log = log
            .to_str()
            .log_expect("Unable to convert log path to string")
            .replace(" ", "\\ ");

        let target_arguments = format!("ping 127.0.0.1 -n 3 > nul && del {} {}", tool, log);

        info!("Launching cmd with {:?}", target_arguments);

        Command::new("C:\\Windows\\system32\\cmd.exe")
            .arg("/C")
            .arg(&target_arguments)
            .spawn()
            .log_expect("Unable to start child process");
    }

    #[allow(unsafe_code)]
    fn get_process_handler(pid: u32) -> Option<HANDLE> {
        if pid == 0 {
            return None;
        }
        let options = PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_TERMINATE;
        let process_handler = unsafe { OpenProcess(options, FALSE, pid as DWORD) };
        if process_handler.is_null() {
            let options = PROCESS_QUERY_INFORMATION | PROCESS_VM_READ;
            let process_handler = unsafe { OpenProcess(options, FALSE, pid as DWORD) };
            if process_handler.is_null() {
                None
            } else {
                Some(process_handler)
            }
        } else {
            Some(process_handler)
        }
    }

    /// Returns a list of running processes
    #[allow(unsafe_code)]
    pub fn get_process_names() -> Vec<super::Process> {
        // Port from https://github.com/GuillaumeGomez/sysinfo/blob/master/src/windows/system.rs
        // I think that 10192 as length will be enough to get all processes at once...
        let mut process_ids = [0 as DWORD; PROCESS_LEN];
        let mut cb_needed = 0;

        let size = ::std::mem::size_of::<DWORD>() * process_ids.len();
        unsafe {
            if K32EnumProcesses(process_ids.as_mut_ptr(),
                                size as DWORD,
                                &mut cb_needed) == 0 {
                return vec![];
            }
        }

        let nb_processes = cb_needed / ::std::mem::size_of::<DWORD>() as DWORD;

        let mut processes = Vec::new();

        for i in 0 .. nb_processes {
            let pid = process_ids[i as usize];

            unsafe {
                if let Some(process_handler) = get_process_handler(pid) {
                    let mut h_mod = ::std::ptr::null_mut();
                    let mut process_name = [0u16; MAX_PATH + 1];
                    let mut cb_needed = 0;

                    if EnumProcessModulesEx(process_handler,
                                            &mut h_mod,
                                            ::std::mem::size_of::<DWORD>() as DWORD,
                                            &mut cb_needed,
                                            LIST_MODULES_ALL) != 0 {
                        GetModuleFileNameExW(process_handler,
                                           h_mod,
                                           process_name.as_mut_ptr(),
                                           MAX_PATH as DWORD + 1);

                        let mut pos = 0;
                        for x in process_name.iter() {
                            if *x == 0 {
                                break
                            }
                            pos += 1;
                        }
                        let name = String::from_utf16_lossy(&process_name[..pos]);

                        processes.push(super::Process {
                            pid: pid as _,
                            name,
                        });
                    }
                }
            }
        }

        processes
    }
}

#[cfg(not(windows))]
mod natives {
    use std::fs::remove_file;

    use std::env;

    use logging::LoggingErrors;

    pub fn create_shortcut(
        name: &str,
        description: &str,
        target: &str,
        args: &str,
        working_dir: &str,
    ) -> Result<String, String> {
        // TODO: no-op
        warn!("create_shortcut is stubbed!");

        Ok("".to_string())
    }

    /// Cleans up the installer
    pub fn burn_on_exit(app_name: &str) {
        let current_exe = env::current_exe().log_expect("Current executable could not be found");

        // Thank god for *nix platforms
        if let Err(e) = remove_file(&current_exe) {
            // No regular logging now.
            eprintln!("Failed to delete maintenancetool: {:?}", e);
        };

        let current_dir = env::current_dir().log_expect("Current directory cannot be found");

        if let Err(e) = remove_file(current_dir.join(format!("{}_installer.log", app_name))) {
            // No regular logging now.
            eprintln!("Failed to delete installer log: {:?}", e);
        };
    }

    /// Returns a list of running processes
    pub fn get_process_names() -> Vec<super::Process> {
        // TODO: no-op
        vec![]
    }
}

pub use self::natives::*;
