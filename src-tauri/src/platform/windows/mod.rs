mod elevation;
mod exe_info;
mod launch;
mod simconnect;
mod top_level_windows;
mod windows_close;
mod windows_minimize;

use ::windows::Win32::Foundation::{CloseHandle, HANDLE};

pub use elevation::is_elevated;
pub use exe_info::{icon_png_base64, product_name};
pub use launch::launch;
pub use simconnect::is_simconnect_available;
pub use windows_close::{request_close, terminate};
pub use windows_minimize::minimize_new_windows;

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: every OwnedHandle wraps a live handle returned by the WinAPI and is closed exactly once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}
