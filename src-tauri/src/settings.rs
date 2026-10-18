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
    pub overlay: OverlayPrefs,
    /// Physical top-left position after the user moved the overlay;
    /// `None` places it at `overlay.anchor`.
    pub overlay_position: Option<OverlayPosition>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            poll_interval_minutes: DEFAULT_POLL_INTERVAL_MINUTES,
            overlay_enabled: false,
            overlay: OverlayPrefs::default(),
            overlay_position: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlayPosition {
    pub x: i32,
    pub y: i32,
}

/// What the overlay shows and how it looks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayPrefs {
    pub anchor: Anchor,
    /// Disables dragging, so the overlay stays where it is.
    pub locked: bool,
    /// Background opacity, from 0 to 100.
    pub opacity: u8,
    pub show_provider: bool,
    pub show_label: bool,
    pub show_percent: bool,
    pub show_reset: bool,
    /// Keys of hidden windows (e.g. `claude:rolling_5h`), so new windows show by default.
    pub hidden_windows: Vec<String>,
}

impl OverlayPrefs {
    pub fn validate(&self) -> Result<(), String> {
        if self.opacity > 100 {
            return Err("A opacidade deve estar entre 0 e 100%.".into());
        }
        Ok(())
    }
}

impl Default for OverlayPrefs {
    fn default() -> Self {
        Self {
            anchor: Anchor::TopRight,
            locked: false,
            opacity: 86,
            show_provider: true,
            show_label: true,
            show_percent: true,
            show_reset: false,
            hidden_windows: Vec::new(),
        }
    }
}

pub fn validate_poll_interval(minutes: u32) -> Result<(), String> {
    if minutes < MIN_POLL_INTERVAL_MINUTES {
        return Err(format!(
            "O intervalo de consulta deve ser de pelo menos {MIN_POLL_INTERVAL_MINUTES} minuto."
        ));
    }
    Ok(())
}

impl Settings {
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
        assert_eq!(settings.overlay, OverlayPrefs::default());
        assert_eq!(settings.overlay_position, None);
    }

    #[test]
    fn poll_interval_is_clamped_to_minimum() {
        let settings = Settings {
            poll_interval_minutes: 0,
            ..Settings::default()
        };

        assert!(validate_poll_interval(settings.poll_interval_minutes).is_err());
        assert_eq!(settings.poll_interval(), Duration::from_secs(60));
    }
}
