use std::collections::HashMap;

use ::windows::Win32::Foundation::STILL_ACTIVE;
use ::windows::Win32::System::Threading::{
    GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

use super::OwnedHandle;

/// Windows keeps a process's exit code readable while someone holds a handle to it, so the
/// handle is opened while the process is still running.
#[derive(Default)]
pub struct ExitWatcher {
    handles: HashMap<u32, OwnedHandle>,
}

impl ExitWatcher {
    pub fn watch(&mut self, pid: u32) {
        if self.handles.contains_key(&pid) {
            return;
        }
        // SAFETY: OpenProcess has no memory-safety preconditions; the returned handle is owned
        // by OwnedHandle and closed exactly once on drop.
        if let Ok(handle) = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) } {
            self.handles.insert(pid, OwnedHandle(handle));
        }
    }

    /// One entry per watched process: `None` while it runs, its exit code once it exited.
    pub fn exit_codes(&self) -> Vec<Option<u32>> {
        self.handles.values().map(exit_code).collect()
    }
}

fn exit_code(handle: &OwnedHandle) -> Option<u32> {
    let mut code = 0u32;
    // SAFETY: the handle is live (owned by the watcher) and `code` is a valid out-pointer.
    unsafe { GetExitCodeProcess(handle.0, &mut code) }.ok()?;
    // A process that really exits with 259 is indistinguishable from a running one; it is
    // treated as running, which only means it is not reopened.
    (code != STILL_ACTIVE.0 as u32).then_some(code)
}
