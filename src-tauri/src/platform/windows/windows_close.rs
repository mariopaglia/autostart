use ::windows::core::BOOL;
use ::windows::Win32::Foundation::{CloseHandle, E_ACCESSDENIED, HANDLE, HWND, LPARAM, WPARAM};
use ::windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
use ::windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE,
};

use crate::error::{AppError, AppResult};

struct WindowSearch {
    pid: u32,
    windows: Vec<HWND>,
}

/// Posts WM_CLOSE to every top-level window of the process, hidden ones included,
/// which is what `taskkill` without `/F` does and what tray-only apps listen to.
pub fn request_close(pid: u32) -> usize {
    let mut search = WindowSearch {
        pid,
        windows: Vec::new(),
    };
    // SAFETY: `search` outlives the synchronous EnumWindows call that receives its address.
    let enumerated =
        unsafe { EnumWindows(Some(collect_window), LPARAM(&mut search as *mut _ as isize)) };
    if let Err(error) = enumerated {
        log::warn!("EnumWindows failed for pid {pid}: {error}");
    }

    search
        .windows
        .into_iter()
        // SAFETY: posting a message to a window handle has no memory-safety requirements.
        .filter(|&window| {
            unsafe { PostMessageW(Some(window), WM_CLOSE, WPARAM(0), LPARAM(0)) }.is_ok()
        })
        .count()
}

unsafe extern "system" fn collect_window(window: HWND, context: LPARAM) -> BOOL {
    // SAFETY: `context` is the `WindowSearch` pointer passed by `request_close`.
    let search = unsafe { &mut *(context.0 as *mut WindowSearch) };
    let mut owner_pid = 0u32;
    // SAFETY: `owner_pid` is a valid out-pointer for the duration of the call.
    unsafe { GetWindowThreadProcessId(window, Some(&mut owner_pid)) };
    if owner_pid == search.pid {
        search.windows.push(window);
    }
    true.into()
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

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: the handle was returned by OpenProcess and is closed exactly once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}
