use ::windows::core::w;
use ::windows::Win32::Foundation::{GetLastError, ERROR_SEM_TIMEOUT};
use ::windows::Win32::System::Pipes::WaitNamedPipeW;

/// The MSFS SimConnect server opens its named pipe once it accepts addon connections.
pub fn is_simconnect_available() -> bool {
    let pipe = w!(r"\\.\pipe\Microsoft Flight Simulator\SimConnect");
    // SAFETY: `pipe` is a static, null-terminated wide string.
    if unsafe { WaitNamedPipeW(pipe, 1) }.as_bool() {
        return true;
    }
    // A timeout means every pipe instance is busy, which still proves the server is running.
    // SAFETY: GetLastError only reads the calling thread's last-error value.
    let last_error = unsafe { GetLastError() };
    last_error == ERROR_SEM_TIMEOUT
}
