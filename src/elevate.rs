use crate::startup::{run_task, task_exists};
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::{HRESULT, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, SetLastError, ERROR_ALREADY_EXISTS, HANDLE, INVALID_HANDLE_VALUE,
    WIN32_ERROR,
};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{CreateMutexW, GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONINFORMATION, MB_OK, SW_SHOWNORMAL,
};

const MUTEX_NAME: &str = "Global\\NakornCode.CapsLang";

pub struct InstanceMutex(HANDLE);

impl Drop for InstanceMutex {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some((&mut elevation as *mut TOKEN_ELEVATION).cast()),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        );
        let _ = CloseHandle(token);
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

pub fn take_single_instance() -> Option<InstanceMutex> {
    let name = wide(MUTEX_NAME);
    unsafe {
        SetLastError(WIN32_ERROR(0));
        match CreateMutexW(None, true, PCWSTR(name.as_ptr())) {
            Ok(handle) if handle != INVALID_HANDLE_VALUE => {
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    let _ = CloseHandle(handle);
                    None
                } else {
                    Some(InstanceMutex(handle))
                }
            }
            _ => None,
        }
    }
}

pub fn hand_off_to_elevated_instance() {
    if task_exists() && run_task().is_ok() {
        return;
    }

    if relaunch_elevated().is_ok() {
        return;
    }

    show_uac_cancelled();
}

fn relaunch_elevated() -> windows::core::Result<()> {
    let exe = std::env::current_exe().map_err(|_| HRESULT(0x80004005u32 as i32))?;
    let exe_wide = wide_path(&exe);
    let verb = wide("runas");
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(exe_wide.as_ptr()),
        nShow: SW_SHOWNORMAL.0 as i32,
        ..Default::default()
    };

    unsafe {
        ShellExecuteExW(&mut info)?;
    }
    Ok(())
}

fn show_uac_cancelled() {
    let title = wide("CapsLang");
    let body = wide(
        "CapsLang needs one administrator approval so it can run elevated.\r\n\r\n\
         That lets CapsLock switch languages inside elevated apps such as\r\n\
         Task Manager, and lets Windows start CapsLang later without more UAC prompts.\r\n\r\n\
         Run CapsLang again and choose Yes.",
    );
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(body.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
