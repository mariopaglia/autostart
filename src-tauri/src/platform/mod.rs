#[cfg(not(windows))]
mod fallback;
#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use self::windows::*;
#[cfg(not(windows))]
pub use fallback::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LaunchOptions {
    pub elevated: bool,
    pub minimized: bool,
}
