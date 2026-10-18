use std::time::Duration;

use serde::{Deserialize, Serialize};

pub const SETTINGS_CHANGED: &str = "settings://changed";

const MIN_POLL_INTERVAL_MINUTES: u32 = 1;
const DEFAULT_POLL_INTERVAL_MINUTES: u32 = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub poll_interval_minutes: u32,
    pub overlay_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            poll_interval_minutes: DEFAULT_POLL_INTERVAL_MINUTES,
            overlay_enabled: false,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.poll_interval_minutes < MIN_POLL_INTERVAL_MINUTES {
            return Err(format!(
                "O intervalo de consulta deve ser de pelo menos {MIN_POLL_INTERVAL_MINUTES} minuto."
            ));
        }
        Ok(())
    }

    /// Clamped to the minimum, in case the stored value was edited by hand.
    pub fn poll_interval(&self) -> Duration {
        let minutes = self.poll_interval_minutes.max(MIN_POLL_INTERVAL_MINUTES);
        Duration::from_secs(u64::from(minutes) * 60)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn missing_fields_take_defaults() {
        // Settings saved before the overlay existed.
        let settings: Settings = serde_json::from_value(json!({"pollIntervalMinutes": 5})).unwrap();

        assert_eq!(settings.poll_interval_minutes, 5);
        assert!(!settings.overlay_enabled);
    }

    #[test]
    fn poll_interval_is_clamped_to_minimum() {
        let settings = Settings {
            poll_interval_minutes: 0,
            ..Settings::default()
        };

        assert!(settings.validate().is_err());
        assert_eq!(settings.poll_interval(), Duration::from_secs(60));
    }
}
