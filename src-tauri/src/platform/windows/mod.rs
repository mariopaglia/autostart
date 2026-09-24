mod elevation;
mod exe_info;
mod launch;
mod process_exit;
mod shortcuts;
mod simconnect;
mod top_level_windows;
mod windows_close;
mod windows_minimize;

use ::windows::Win32::Foundation::{CloseHandle, HANDLE};

pub use elevation::is_elevated;
pub use exe_info::{icon_png_base64, product_name};
pub use launch::{launch, open_link};
pub use process_exit::ExitWatcher;
pub use shortcuts::{shortcut_folders, ShortcutResolver};
pub use simconnect::is_simconnect_available;
pub use top_level_windows::pids_with_visible_windows;
pub use windows_close::{request_close, terminate};
pub use windows_minimize::minimize_new_windows;

struct OwnedHandle(HANDLE);

// SAFETY: kernel object handles are process-wide and may be used and closed from any thread;
// OwnedHandle is never shared (no Sync), so moving it between async tasks' threads is sound.
unsafe impl Send for OwnedHandle {}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: every OwnedHandle wraps a live handle returned by the WinAPI and is closed exactly once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

#[cfg(test)]
mod tests;
