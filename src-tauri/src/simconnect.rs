use std::time::{Duration, Instant};

use crate::platform;

const SIMCONNECT_TRIGGERS: [&str; 2] = ["flightsimulator.exe", "flightsimulator2024.exe"];
const POLL_INTERVAL: Duration = Duration::from_secs(2);
/// MSFS 2024 can take several minutes to reach the main menu on modest machines.
pub const WAIT_TIMEOUT: Duration = Duration::from_secs(10 * 60);

pub fn supports_simconnect(trigger_process_name: &str) -> bool {
    let normalized = trigger_process_name.trim().to_lowercase();
    SIMCONNECT_TRIGGERS.contains(&normalized.as_str())
}

/// Returns whether SimConnect became available before the timeout.
pub async fn wait_until_available(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if platform::is_simconnect_available() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_both_msfs_versions_regardless_of_case() {
        assert!(supports_simconnect("FlightSimulator.exe"));
        assert!(supports_simconnect("FlightSimulator2024.exe"));
        assert!(supports_simconnect(" flightsimulator2024.EXE "));
    }

    #[test]
    fn rejects_other_simulators() {
        assert!(!supports_simconnect("X-Plane.exe"));
        assert!(!supports_simconnect("FlightSimulatorX.exe"));
    }
}
