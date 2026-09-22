mod elevation;
mod exe_info;
mod windows_close;

use std::os::windows::process::CommandExt;
use std::process::Command;

use ::windows::Win32::System::Threading::{CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS};

pub use elevation::{is_elevated, launch_elevated};
pub use exe_info::{icon_png_base64, product_name};
pub use windows_close::{request_close, terminate};

pub fn configure_launch(command: &mut Command, args: Option<&str>) {
    // Detached so helper apps outlive AutoStart; raw_arg keeps the user's quoting untouched.
    command.creation_flags(DETACHED_PROCESS.0 | CREATE_NEW_PROCESS_GROUP.0);
    if let Some(args) = args {
        command.raw_arg(args);
    }
}
