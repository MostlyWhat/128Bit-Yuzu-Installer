//! Natives/platform specific interactions.

/// Basic definition of some running process.
#[derive(Debug)]
pub struct Process {
    pub pid: usize,
    pub name: String,
}

#[cfg(windows)]
mod natives {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    const PROCESS_LEN: usize = 10192;
    const WV2_INSTALLER_DATA: &[u8] = include_bytes!("../../MicrosoftEdgeWebview2Setup.exe");

    use crate::logging::LoggingErrors;

    use std::env;
    use std::io::Write;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    use winapi::shared::minwindef::{DWORD, FALSE, MAX_PATH};

    use winapi::shared::winerror::HRESULT;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::psapi::{
        EnumProcessModulesEx, GetModuleFileNameExW, K32EnumProcesses, LIST_MODULES_ALL,
    };
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::winnt::{
        HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, PROCESS_VM_READ,
    };
    use winapi::um::winuser::SW_SHOWDEFAULT;

    use std::process::Command;
    use tempfile::Builder;
    use tinyfiledialogs::{message_box_yes_no, MessageBoxIcon, YesNo};
    use webview2::EnvironmentBuilder;
    use widestring::U16CString;

    extern "C" {
        pub fn saveShortcut(
            shortcutPath: *const winapi::ctypes::wchar_t,
            description: *const winapi::ctypes::wchar_t,
            path: *const winapi::ctypes::wchar_t,
            args: *const winapi::ctypes::wchar_t,
            workingDir: *const winapi::ctypes::wchar_t,
            exePath: *const winapi::ctypes::wchar_t,
        ) -> ::std::os::raw::c_int;

        pub fn isDarkThemeActive() -> ::std::os::raw::c_uint;

        pub fn spawnDetached(
            app: *const winapi::ctypes::wchar_t,
            cmdline: *const winapi::ctypes::wchar_t,
        ) -> ::std::os::raw::c_int;

        pub fn getSystemFolder(out_path: *mut ::std::os::raw::c_ushort) -> HRESULT;

        pub fn getDesktopFolder(out_path: *mut ::std::os::raw::c_ushort) -> HRESULT;
    }

    pub fn prepare_install_webview2(name: &str) -> Result<(), String> {
        if EnvironmentBuilder::default()
            .get_available_browser_version_string()
            .is_ok()
        {
            return Ok(());
        }
        if message_box_yes_no(&format!("{} installer", name), &format!("{} installer now requires Webview2 runtime to function properly.\nDo you wish to install it now?", name), MessageBoxIcon::Question, YesNo::Yes) == YesNo::No {
            std::process::exit(1);
        }
        let mut installer_file = Builder::new()
            .suffix(".exe")
            .tempfile()
            .log_expect("Unable to open the webview2 installer file");
        installer_file
            .write_all(&WV2_INSTALLER_DATA)
            .log_expect("Unable to write the webview2 installer file");
        let path = installer_file.path().to_owned();
        installer_file.keep().log_unwrap();
        Command::new(&path)
            .arg("/install")
            .spawn()
            .log_expect("Unable to run the webview2 installer")
            .wait()
            .log_unwrap();
        Ok(())
    }

    // Needed here for Windows interop
    #[allow(unsafe_code)]
    pub fn create_shortcut(
        name: &str,
        description: &str,
        target: &str,
        args: &str,
        working_dir: &str,
        exe_path: &str,
    ) -> Result<String, String> {
        let source_file = format!(
            "{}\\Microsoft\\Windows\\Start Menu\\Programs\\{}.lnk",
            env::var("APPDATA").log_expect("APPDATA is bad, apparently"),
            name
        );
        create_shortcut_inner(
            source_file,
            name,
            description,
            target,
            args,
            working_dir,
            exe_path,
        )
    }

    // Needed here for Windows interop
    #[allow(unsafe_code)]
    pub fn create_desktop_shortcut(
        name: &str,
        description: &str,
        target: &str,
        args: &str,
        working_dir: &str,
        exe_path: &str,
    ) -> Result<String, String> {
        let mut cmd_path = [0u16; MAX_PATH + 1];
        let _result = unsafe { getDesktopFolder(cmd_path.as_mut_ptr()) };
        let source_path = format!(
            "{}\\{}.lnk",
            String::from_utf16_lossy(&cmd_path[..count_u16(&cmd_path)]).as_str(),
            name
        );
        create_shortcut_inner(
            source_path,
            name,
            description,
            target,
            args,
            working_dir,
            exe_path,
        )
    }

    // Needed here for Windows interop
    #[allow(unsafe_code)]
    fn create_shortcut_inner(
        source_file: String,
        _name: &str,
        description: &str,
        target: &str,
        args: &str,
        working_dir: &str,
        exe_path: &str,
    ) -> Result<String, String> {
        info!("Generating shortcut @ {:?}", source_file);

        let native_target_dir = U16CString::from_str(source_file.clone())
            .log_expect("Error while converting to wchar_t");
        let native_description =
            U16CString::from_str(description).log_expect("Error while converting to wchar_t");
        let native_target =
            U16CString::from_str(target).log_expect("Error while converting to wchar_t");
        let native_args =
            U16CString::from_str(args).log_expect("Error while converting to wchar_t");
        let native_working_dir =
            U16CString::from_str(working_dir).log_expect("Error while converting to wchar_t");
        let native_exe_path =
            U16CString::from_str(exe_path).log_expect("Error while converting to wchar_t");

        let shortcutResult = unsafe {
            saveShortcut(
                native_target_dir.as_ptr(),
                native_description.as_ptr(),
                native_target.as_ptr(),
                native_args.as_ptr(),
                native_working_dir.as_ptr(),
                native_exe_path.as_ptr(),
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

    // Needed to call unsafe function `ShellExecuteW` from `winapi` crate
    #[allow(unsafe_code)]
    pub fn open_in_shell(path: &Path) {
        let native_verb = U16CString::from_str("open").unwrap();
        // https://doc.rust-lang.org/std/os/windows/ffi/trait.OsStrExt.html#tymethod.encode_wide
        let mut native_path: Vec<u16> = path.as_os_str().encode_wide().collect();
        native_path.push(0); // NULL terminator
        unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                native_verb.as_ptr(),
                native_path.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                SW_SHOWDEFAULT,
            );
        }
    }

    #[inline]
    fn count_u16(u16str: &[u16]) -> usize {
        let mut pos = 0;
        for x in u16str.iter() {
            if *x == 0 {
                break;
            }
            pos += 1;
        }
        pos
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
        let tool_wv = format!("{}.WebView2", tool);

        let log = path.join(format!("{}_installer.log", app_name));
        let log = log
            .to_str()
            .log_expect("Unable to convert log path to string")
            .replace(" ", "\\ ");

        let install_path = path
            .to_str()
            .log_expect("Unable to convert path to string")
            .replace(" ", "\\ ");

        let target_arguments = format!(
            "/C choice /C Y /N /D Y /T 2 & del {} {} & rmdir /Q /S {} & rmdir {}",
            tool, log, tool_wv, install_path
        );

        info!("Launching cmd with {:?}", target_arguments);

        // Needs to use `spawnDetached` which is an unsafe C/C++ function from interop.cpp
        #[allow(unsafe_code)]
        let spawn_result: i32 = unsafe {
            let mut cmd_path = [0u16; MAX_PATH + 1];
            let result = getSystemFolder(cmd_path.as_mut_ptr());
            let mut pos = 0;
            for x in cmd_path.iter() {
                if *x == 0 {
                    break;
                }
                pos += 1;
            }
            if result != winapi::shared::winerror::S_OK {
                return;
            }

            spawnDetached(
                U16CString::from_str(
                    format!("{}\\cmd.exe", String::from_utf16_lossy(&cmd_path[..pos])).as_str(),
                )
                .log_expect("Unable to convert string to wchar_t")
                .as_ptr(),
                U16CString::from_str(target_arguments.as_str())
                    .log_expect("Unable to convert string to wchar_t")
                    .as_ptr(),
            )
        };

        if spawn_result != 0 {
            warn!("Unable to start child process");
        }
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
            if K32EnumProcesses(process_ids.as_mut_ptr(), size as DWORD, &mut cb_needed) == 0 {
                return vec![];
            }
        }

        let nb_processes = cb_needed / ::std::mem::size_of::<DWORD>() as DWORD;

        let mut processes = Vec::new();

        for i in 0..nb_processes {
            let pid = process_ids[i as usize];

            unsafe {
                if let Some(process_handler) = get_process_handler(pid) {
                    let mut h_mod = ::std::ptr::null_mut();
                    let mut process_name = [0u16; MAX_PATH + 1];
                    let mut cb_needed = 0;

                    if EnumProcessModulesEx(
                        process_handler,
                        &mut h_mod,
                        ::std::mem::size_of::<DWORD>() as DWORD,
                        &mut cb_needed,
                        LIST_MODULES_ALL,
                    ) != 0
                    {
                        GetModuleFileNameExW(
                            process_handler,
                            h_mod,
                            process_name.as_mut_ptr(),
                            MAX_PATH as DWORD + 1,
                        );

                        let mut pos = 0;
                        for x in process_name.iter() {
                            if *x == 0 {
                                break;
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

    // Needed here for Windows interop
    #[allow(unsafe_code)]
    pub fn is_dark_mode_active() -> bool {
        unsafe { isDarkThemeActive() == 1 }
    }
}

#[cfg(not(windows))]
mod natives {
    use std::fs::{remove_dir, remove_file};

    use std::env;

    use crate::logging::LoggingErrors;

    use sysinfo::{PidExt, ProcessExt, SystemExt};

    use dirs;

    use slug::slugify;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use std::path::Path;
    use std::process::Command;

    #[cfg(target_os = "linux")]
    pub fn create_shortcut(
        name: &str,
        description: &str,
        target: &str,
        args: &str,
        working_dir: &str,
        exe_path: &str,
    ) -> Result<String, String> {
        // FIXME: no icon will be shown since no icon is provided
        let data_local_dir = dirs::data_local_dir();
        match data_local_dir {
            Some(x) => {
                let mut path = x;
                path.push("applications");
                match create_dir_all(path.to_path_buf()) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(format!(
                            "Local data directory does not exist and cannot be created: {}",
                            e
                        ));
                    }
                };
                path.push(format!("yuzu-maintenance-tool_{}.desktop", slugify(name))); // file name
                let desktop_file = format!(
                "[Desktop Entry]\nType=Application\nName={}\nExec=\"{}\" {}\nComment={}\nPath={}\nIcon=yuzu\n",
                name, target, args, description, working_dir
                );
                let desktop_f = File::create(path);
                let mut desktop_f = match desktop_f {
                    Ok(file) => file,
                    Err(e) => return Err(format!("Unable to create desktop file: {}", e)),
                };
                let desktop_f = desktop_f.write_all(desktop_file.as_bytes());
                match desktop_f {
                    Ok(_) => Ok("".to_string()),
                    Err(e) => Err(format!("Unable to write desktop file: {}", e)),
                }
            }
            // return error when failed to acquire local data directory
            None => Err("Unable to determine local data directory".to_string()),
        }
    }

    #[cfg(target_os = "macos")]
    pub fn create_shortcut(
        name: &str,
        description: &str,
        target: &str,
        args: &str,
        working_dir: &str,
        _exe_path: &str,
    ) -> Result<String, String> {
        warn!("STUB! Creating shortcut is not implemented on macOS");
        Ok("".to_string())
    }

    pub fn open_in_shell(path: &Path) {
        let shell: &str;
        if cfg!(target_os = "linux") {
            shell = "xdg-open";
        } else if cfg!(target_os = "macos") {
            shell = "open";
        } else {
            warn!("Unsupported platform");
            return;
        }
        Command::new(shell).arg(path).spawn().ok();
    }

    /// Cleans up the installer
    pub fn burn_on_exit(app_name: &str) {
        let current_exe = env::current_exe().log_expect("Current executable could not be found");
        let exe_dir = current_exe
            .parent()
            .log_expect("Current executable directory cannot be found");

        if let Err(e) = remove_file(exe_dir.join(format!("{}_installer.log", app_name))) {
            // No regular logging now.
            eprintln!("Failed to delete maintenance log: {:?}", e);
        };

        // Thank god for *nix platforms
        if let Err(e) = remove_file(&current_exe) {
            // No regular logging now.
            eprintln!("Failed to delete maintenancetool: {:?}", e);
        };
        // delete the directory if not empty and ignore errors (since we can't handle errors anymore)
        remove_dir(exe_dir).ok();
    }

    /// Returns a list of running processes
    pub fn get_process_names() -> Vec<super::Process> {
        // a platform-independent implementation using sysinfo crate
        let mut processes: Vec<super::Process> = Vec::new();
        let mut system = sysinfo::System::new();
        system.refresh_all();
        for (pid, procs) in system.processes() {
            processes.push(super::Process {
                pid: pid.as_u32() as usize,
                name: procs.name().to_string(),
            });
        }
        processes // return running processes
    }

    /// Returns if dark mode is active on this system.
    pub fn is_dark_mode_active() -> bool {
        // No-op
        false
    }
}

pub use self::natives::*;
