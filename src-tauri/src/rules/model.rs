use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{ProviderId, WindowKind};
use crate::i18n::{Key, t};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Notification,
    Overlay,
    Both,
}

impl Channel {
    pub fn notifies(self) -> bool {
        matches!(self, Self::Notification | Self::Both)
    }

    pub fn shows_overlay(self) -> bool {
        matches!(self, Self::Overlay | Self::Both)
    }
}

/// In `provider` and `window`, `None` means "any".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RuleKind {
    /// Fires when utilization crosses `percent`, at most once per reset cycle.
    Threshold {
        provider: Option<ProviderId>,
        window: Option<WindowKind>,
        percent: f64,
    },
    /// Usage summary every `minutes`.
    Interval { minutes: u32 },
    /// Fires once when the window resets in `minutes` or less.
    BeforeReset {
        provider: Option<ProviderId>,
        window: Option<WindowKind>,
        minutes: u32,
    },
}

impl RuleKind {
    pub fn validate(&self) -> Result<(), String> {
        match *self {
            // Also rejects NaN.
            Self::Threshold { percent, .. } if !(percent > 0.0 && percent <= 100.0) => {
                Err(t(Key::ThresholdRange).into())
            }
            Self::Interval { minutes: 0 } | Self::BeforeReset { minutes: 0, .. } => {
                Err(t(Key::MinutesMinimum).into())
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub enabled: bool,
    pub channel: Channel,
    pub kind: RuleKind,
}

/// A rule as sent by the UI on creation, before it gets an id.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleInput {
    pub enabled: bool,
    pub channel: Channel,
    pub kind: RuleKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub rule_id: String,
    pub channel: Channel,
    pub title: String,
    pub body: String,
}

/// For each rule id and window key, the `resets_at` of the cycle in which the rule last fired.
pub type FiredCycles = HashMap<String, HashMap<String, DateTime<Utc>>>;

#[derive(Debug, Clone, Copy)]
pub(super) struct Observation {
    pub utilization: f64,
    pub resets_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct RuleState {
    /// Persisted, so a restart does not repeat alerts of the current cycle.
    pub fired: FiredCycles,
    /// Last utilization seen per (rule id, window key). In memory only.
    pub(super) last_seen: HashMap<(String, String), Observation>,
    /// Baseline or last firing of each interval rule. In memory only.
    pub(super) interval_last: HashMap<String, DateTime<Utc>>,
}

impl RuleState {
    pub fn with_fired(fired: FiredCycles) -> Self {
        Self {
            fired,
            ..Self::default()
        }
    }

    /// Drops everything known about a rule, so an edited rule starts fresh.
    pub fn forget(&mut self, rule_id: &str) {
        self.fired.remove(rule_id);
        self.last_seen.retain(|(id, _), _| id != rule_id);
        self.interval_last.remove(rule_id);
    }
}

/// Rules and their evaluation state, kept under a single lock.
#[derive(Debug, Default)]
pub struct RuleBook {
    pub rules: Vec<Rule>,
    pub state: RuleState,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn validates_rule_kinds() {
        let threshold = |percent| RuleKind::Threshold {
            provider: None,
            window: None,
            percent,
        };
        assert!(threshold(80.0).validate().is_ok());
        assert!(threshold(100.0).validate().is_ok());
        for invalid in [0.0, -5.0, 100.5, f64::NAN] {
            assert!(threshold(invalid).validate().is_err(), "{invalid}");
        }

        assert!(RuleKind::Interval { minutes: 1 }.validate().is_ok());
        assert!(RuleKind::Interval { minutes: 0 }.validate().is_err());
        let before_reset = RuleKind::BeforeReset {
            provider: None,
            window: None,
            minutes: 0,
        };
        assert!(before_reset.validate().is_err());
    }

    /// Guards the JSON contract with src/lib/types.ts.
    #[test]
    fn rule_json_shape() {
        let rule = Rule {
            id: "r1".into(),
            enabled: true,
            channel: Channel::Both,
            kind: RuleKind::BeforeReset {
                provider: None,
                window: Some(WindowKind::Rolling5h),
                minutes: 15,
            },
        };
        let expected = json!({
            "id": "r1",
            "enabled": true,
            "channel": "both",
            "kind": {"type": "beforeReset", "provider": null, "window": "rolling_5h", "minutes": 15}
        });

        assert_eq!(serde_json::to_value(&rule).unwrap(), expected);
        assert_eq!(serde_json::from_value::<Rule>(expected).unwrap(), rule);
    }
}
