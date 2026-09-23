#[cfg(not(windows))]
mod fallback;
#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use self::windows::*;
#[cfg(not(windows))]
pub use fallback::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutTarget {
    pub target_path: std::path::PathBuf,
    pub args: Option<String>,
    pub working_dir: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LaunchOptions {
    pub elevated: bool,
    pub minimized: bool,
}
