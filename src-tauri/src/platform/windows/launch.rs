use std::mem::size_of;
use std::path::Path;

use ::windows::core::{w, HSTRING, PCWSTR};
use ::windows::Win32::Foundation::ERROR_CANCELLED;
use ::windows::Win32::System::Threading::GetProcessId;
use ::windows::Win32::UI::Shell::{
    ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
};
use ::windows::Win32::UI::WindowsAndMessaging::{SW_SHOWMINNOACTIVE, SW_SHOWNORMAL};

use super::OwnedHandle;
use crate::error::{AppError, AppResult};
use crate::platform::LaunchOptions;

/// Returns the PID of the started process, or `None` when the shell handed the request to an
/// already running instance and no new process exists.
pub fn launch(
    exe_path: &Path,
    args: Option<&str>,
    working_dir: &Path,
    options: LaunchOptions,
) -> AppResult<Option<u32>> {
    let file = HSTRING::from(exe_path.as_os_str());
    // Passing the user's arguments as a single string keeps their quoting untouched.
    let parameters = args.map(HSTRING::from);
    let directory = HSTRING::from(working_dir.as_os_str());
    // SW_SHOWMINNOACTIVE also keeps the focus on the simulator instead of the new window.
    let show = if options.minimized {
        SW_SHOWMINNOACTIVE
    } else {
        SW_SHOWNORMAL
    };

    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: if options.elevated {
            w!("runas")
        } else {
            PCWSTR::null()
        },
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: parameters
            .as_ref()
            .map_or(PCWSTR::null(), |value| PCWSTR(value.as_ptr())),
        lpDirectory: PCWSTR(directory.as_ptr()),
        nShow: show.0,
        ..Default::default()
    };

    // SAFETY: every string pointer in `info` borrows an HSTRING that lives until this call returns.
    unsafe { ShellExecuteExW(&mut info) }.map_err(|error| map_launch_error(&error, exe_path))?;

    if info.hProcess.is_invalid() {
        return Ok(None);
    }
    let process = OwnedHandle(info.hProcess);
    // SAFETY: `process` holds the live handle requested with SEE_MASK_NOCLOSEPROCESS.
    let pid = unsafe { GetProcessId(process.0) };
    Ok((pid != 0).then_some(pid))
}

/// Hands a web address, a Steam link or a `shell:` path to the program Windows associates with it.
pub fn open_link(link: &str) -> AppResult<()> {
    let file = HSTRING::from(link);
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOASYNC,
        lpFile: PCWSTR(file.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    // SAFETY: `lpFile` borrows an HSTRING that lives until this call returns.
    unsafe { ShellExecuteExW(&mut info) }.map_err(|error| AppError::LaunchFailed {
        name: link.to_owned(),
        reason: error.message(),
    })
}

fn map_launch_error(error: &::windows::core::Error, exe_path: &Path) -> AppError {
    if error.code() == ERROR_CANCELLED.to_hresult() {
        return AppError::ElevationDenied;
    }
    AppError::LaunchFailed {
        name: exe_path.display().to_string(),
        reason: error.message(),
    }
}
