use std::time::{Duration, Instant};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

pub const MINIMIZED_FLAG: &str = "--minimized";
const WAIT_FOR_PID_FLAG: &str = "--wait-for-pid";
const PREVIOUS_INSTANCE_TIMEOUT: Duration = Duration::from_secs(10);
const PREVIOUS_INSTANCE_POLL: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LaunchArgs {
    pub minimized: bool,
    pub previous_instance_pid: Option<u32>,
}

impl LaunchArgs {
    pub fn from_env() -> Self {
        Self::parse(std::env::args().skip(1))
    }

    pub fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let mut parsed = Self::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                MINIMIZED_FLAG => parsed.minimized = true,
                WAIT_FOR_PID_FLAG => {
                    parsed.previous_instance_pid = args.next().and_then(|pid| pid.parse().ok());
                }
                _ => {}
            }
        }
        parsed
    }
}

pub fn wait_for_pid_args(pid: u32) -> String {
    format!("{WAIT_FOR_PID_FLAG} {pid}")
}

/// An elevated relaunch starts while the old instance is still exiting; waiting keeps the
/// single-instance plugin from handing control back to the instance that is going away.
pub fn wait_for_previous_instance(pid: u32) {
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    let deadline = Instant::now() + PREVIOUS_INSTANCE_TIMEOUT;

    while Instant::now() < deadline {
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing(),
        );
        if system.process(pid).is_none() {
            return;
        }
        std::thread::sleep(PREVIOUS_INSTANCE_POLL);
    }
    log::warn!("previous instance {pid} did not exit in time");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> LaunchArgs {
        LaunchArgs::parse(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn no_args_means_a_normal_start() {
        assert_eq!(parse(&[]), LaunchArgs::default());
    }

    #[test]
    fn detects_minimized_flag() {
        assert!(parse(&["--minimized"]).minimized);
    }

    #[test]
    fn reads_previous_instance_pid() {
        let args = parse(&["--wait-for-pid", "4242"]);
        assert_eq!(args.previous_instance_pid, Some(4242));
        assert!(!args.minimized);
    }

    #[test]
    fn ignores_invalid_pid() {
        assert_eq!(
            parse(&["--wait-for-pid", "abc"]).previous_instance_pid,
            None
        );
    }

    #[test]
    fn wait_for_pid_args_round_trip() {
        let args = LaunchArgs::parse(wait_for_pid_args(7).split(' ').map(str::to_owned));
        assert_eq!(args.previous_instance_pid, Some(7));
    }
}
