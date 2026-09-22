use std::collections::HashSet;

use ::windows::Win32::UI::WindowsAndMessaging::{
    IsIconic, IsWindowVisible, ShowWindow, SW_SHOWMINNOACTIVE,
};

use super::top_level_windows::top_level_windows;

/// Minimizes each visible window once; a window the user restores afterwards is left alone.
pub fn minimize_new_windows(pids: &HashSet<u32>, already_minimized: &mut HashSet<isize>) {
    for window in top_level_windows(pids) {
        let handle = window.0 as isize;
        // SAFETY: querying the state of a window handle has no memory-safety requirements.
        let is_candidate =
            unsafe { IsWindowVisible(window).as_bool() && !IsIconic(window).as_bool() };
        if !is_candidate || !already_minimized.insert(handle) {
            continue;
        }
        // SW_SHOWMINNOACTIVE minimizes without activating the next window, so the simulator keeps focus.
        // SAFETY: ShowWindow only reads its arguments.
        let _ = unsafe { ShowWindow(window, SW_SHOWMINNOACTIVE) };
    }
}
