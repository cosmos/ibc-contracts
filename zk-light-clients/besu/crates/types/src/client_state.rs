use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientState {
    pub latest_height: u64,
    pub trusting_period_seconds: u64,
    pub max_clock_drift_seconds: u64,
}
