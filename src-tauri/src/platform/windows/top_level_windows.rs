use std::collections::HashSet;

use ::windows::core::BOOL;
use ::windows::Win32::Foundation::{HWND, LPARAM};
use ::windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId};

struct WindowSearch<'a> {
    pids: &'a HashSet<u32>,
    windows: Vec<HWND>,
}

/// Every top-level window owned by the given processes, hidden ones included.
pub fn top_level_windows(pids: &HashSet<u32>) -> Vec<HWND> {
    let mut search = WindowSearch {
        pids,
        windows: Vec::new(),
    };
    // SAFETY: `search` outlives the synchronous EnumWindows call that receives its address.
    let enumerated =
        unsafe { EnumWindows(Some(collect_window), LPARAM(&mut search as *mut _ as isize)) };
    if let Err(error) = enumerated {
        log::warn!("EnumWindows failed: {error}");
    }
    search.windows
}

unsafe extern "system" fn collect_window(window: HWND, context: LPARAM) -> BOOL {
    // SAFETY: `context` is the `WindowSearch` pointer passed by `top_level_windows`.
    let search = unsafe { &mut *(context.0 as *mut WindowSearch) };
    let mut owner_pid = 0u32;
    // SAFETY: `owner_pid` is a valid out-pointer for the duration of the call.
    unsafe { GetWindowThreadProcessId(window, Some(&mut owner_pid)) };
    if search.pids.contains(&owner_pid) {
        search.windows.push(window);
    }
    true.into()
}
