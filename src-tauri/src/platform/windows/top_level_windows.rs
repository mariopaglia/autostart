use std::collections::HashSet;

use ::windows::core::BOOL;
use ::windows::Win32::Foundation::{HWND, LPARAM};
use ::windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindow, GetWindowLongPtrW, GetWindowThreadProcessId, IsWindowVisible,
    GWL_EXSTYLE, GW_OWNER, WS_EX_TOOLWINDOW,
};

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

/// Processes that show a regular app window: visible, not owned by another window, not a tool window.
pub fn pids_with_visible_windows() -> HashSet<u32> {
    let mut pids = HashSet::new();
    // SAFETY: `pids` outlives the synchronous EnumWindows call that receives its address.
    let enumerated = unsafe {
        EnumWindows(
            Some(collect_app_window_owner),
            LPARAM(&mut pids as *mut _ as isize),
        )
    };
    if let Err(error) = enumerated {
        log::warn!("EnumWindows failed: {error}");
    }
    pids
}

unsafe extern "system" fn collect_app_window_owner(window: HWND, context: LPARAM) -> BOOL {
    // SAFETY: `context` is the `HashSet` pointer passed by `pids_with_visible_windows`.
    let pids = unsafe { &mut *(context.0 as *mut HashSet<u32>) };
    if is_app_window(window) {
        let mut owner_pid = 0u32;
        // SAFETY: `owner_pid` is a valid out-pointer for the duration of the call.
        unsafe { GetWindowThreadProcessId(window, Some(&mut owner_pid)) };
        pids.insert(owner_pid);
    }
    true.into()
}

fn is_app_window(window: HWND) -> bool {
    // SAFETY: querying the state of a window handle has no memory-safety requirements.
    unsafe {
        let has_owner = GetWindow(window, GW_OWNER).is_ok_and(|owner| !owner.is_invalid());
        let extended_style = GetWindowLongPtrW(window, GWL_EXSTYLE) as u32;
        IsWindowVisible(window).as_bool() && !has_owner && extended_style & WS_EX_TOOLWINDOW.0 == 0
    }
}
