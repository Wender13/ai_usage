use std::time::Duration;

use serde::{Deserialize, Serialize};

const MIN_POLL_INTERVAL_MINUTES: u32 = 1;
const DEFAULT_POLL_INTERVAL_MINUTES: u32 = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub poll_interval_minutes: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            poll_interval_minutes: DEFAULT_POLL_INTERVAL_MINUTES,
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
    use super::*;

    #[test]
    fn poll_interval_is_clamped_to_minimum() {
        let settings = Settings {
            poll_interval_minutes: 0,
        };

        assert!(settings.validate().is_err());
        assert_eq!(settings.poll_interval(), Duration::from_secs(60));
    }
}
