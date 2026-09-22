use std::collections::HashSet;

use ::windows::Win32::Foundation::{E_ACCESSDENIED, LPARAM, WPARAM};
use ::windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
use ::windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

use super::top_level_windows::top_level_windows;
use super::OwnedHandle;
use crate::error::{AppError, AppResult};

/// Posts WM_CLOSE to every top-level window of the process, hidden ones included,
/// which is what `taskkill` without `/F` does and what tray-only apps listen to.
pub fn request_close(pid: u32) -> usize {
    top_level_windows(&HashSet::from([pid]))
        .into_iter()
        // SAFETY: posting a message to a window handle has no memory-safety requirements.
        .filter(|&window| {
            unsafe { PostMessageW(Some(window), WM_CLOSE, WPARAM(0), LPARAM(0)) }.is_ok()
        })
        .count()
}

pub fn terminate(pid: u32, process_name: &str) -> AppResult<()> {
    // SAFETY: OpenProcess only reads its arguments; the handle is closed by `OwnedHandle`.
    let handle = unsafe { OpenProcess(PROCESS_TERMINATE, false, pid) }
        .map(OwnedHandle)
        .map_err(|error| map_terminate_error(error, process_name))?;
    // SAFETY: `handle` is a live process handle opened with PROCESS_TERMINATE.
    unsafe { TerminateProcess(handle.0, 1) }
        .map_err(|error| map_terminate_error(error, process_name))
}

fn map_terminate_error(error: ::windows::core::Error, process_name: &str) -> AppError {
    if error.code() == E_ACCESSDENIED {
        return AppError::AccessDenied(process_name.to_owned());
    }
    AppError::Io(std::io::Error::other(error))
}
