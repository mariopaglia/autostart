use std::ffi::c_void;
use std::mem::size_of;
use std::path::Path;

use ::windows::core::{w, HSTRING, PCWSTR};
use ::windows::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, HANDLE};
use ::windows::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use ::windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use ::windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW};
use ::windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use crate::error::{AppError, AppResult};

pub fn is_elevated() -> bool {
    let mut token = HANDLE::default();
    // SAFETY: GetCurrentProcess returns a pseudo-handle; `token` is a valid out-pointer.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }.is_err() {
        return false;
    }

    let mut elevation = TOKEN_ELEVATION::default();
    let mut returned_length = 0u32;
    // SAFETY: the buffer is a TOKEN_ELEVATION whose exact size is passed alongside it.
    let queried = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut c_void),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned_length,
        )
    };
    // SAFETY: `token` was opened above and is closed exactly once.
    let _ = unsafe { CloseHandle(token) };

    queried.is_ok() && elevation.TokenIsElevated != 0
}

pub fn launch_elevated(exe_path: &Path, args: Option<&str>, working_dir: &Path) -> AppResult<()> {
    let file = HSTRING::from(exe_path.as_os_str());
    let parameters = args.map(HSTRING::from);
    let directory = HSTRING::from(working_dir.as_os_str());

    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOASYNC,
        lpVerb: w!("runas"),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: parameters
            .as_ref()
            .map_or(PCWSTR::null(), |value| PCWSTR(value.as_ptr())),
        lpDirectory: PCWSTR(directory.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    // SAFETY: every string pointer in `info` borrows an HSTRING that lives until this call returns.
    unsafe { ShellExecuteExW(&mut info) }.map_err(|error| {
        if error.code() == ERROR_CANCELLED.to_hresult() {
            AppError::ElevationDenied
        } else {
            AppError::LaunchFailed {
                name: exe_path.display().to_string(),
                reason: error.message(),
            }
        }
    })
}
