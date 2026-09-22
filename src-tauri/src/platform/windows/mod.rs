mod elevation;
mod exe_info;
mod launch;
mod windows_close;

use ::windows::Win32::Foundation::{CloseHandle, HANDLE};

pub use elevation::is_elevated;
pub use exe_info::{icon_png_base64, product_name};
pub use launch::launch;
pub use windows_close::{request_close, terminate};

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: every OwnedHandle wraps a live handle returned by the WinAPI and is closed exactly once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}
