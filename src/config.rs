use std::env;

pub const FRAME_INTERVAL_MS: f64 = 20.0;
pub const MAX_FEC_WIDTH: u32 = 5;
const DEFAULT_DELAY_MS: f64 = 120.0;
const MAX_RECOVERY_SLOTS: usize = 2_048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportConfig {
    pub fec_width: u32,
    pub recovery_slots: usize,
}

impl TransportConfig {
    pub fn from_env() -> Self {
        let delay_ms = env::var("DELAY_MS")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value: &f64| value.is_finite() && *value >= 0.0)
            .unwrap_or(DEFAULT_DELAY_MS);
        Self::for_delay(delay_ms)
    }

    pub fn for_delay(delay_ms: f64) -> Self {
        let delay_ms = if delay_ms.is_finite() && delay_ms >= 0.0 {
            delay_ms
        } else {
            DEFAULT_DELAY_MS
        };
        let fec_width = 1
            + (delay_ms / (2.0 * FRAME_INTERVAL_MS))
                .floor()
                .clamp(1.0, f64::from(MAX_FEC_WIDTH - 1)) as u32;
        let deadline_frames = (delay_ms / FRAME_INTERVAL_MS).ceil() as usize;
        let required_slots = deadline_frames
            .saturating_add(2 * (fec_width as usize - 1) + 4)
            .clamp(8, MAX_RECOVERY_SLOTS);
        let recovery_slots = required_slots.next_power_of_two();
        Self {
            fec_width,
            recovery_slots,
        }
    }
}
