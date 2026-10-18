//! Rule evaluation. Pure: all time and state come in as arguments.

use chrono::{DateTime, TimeDelta, Utc};

use super::model::{Alert, Observation, Rule, RuleKind, RuleState};
use crate::domain::{ProviderId, UsageWindow, WindowKind};

/// Reset times closer than this belong to the same cycle. The API adds sub-second
/// jitter to `resets_at` on every response, while real cycles are hours apart.
const CYCLE_TOLERANCE: TimeDelta = TimeDelta::minutes(10);

/// Evaluates enabled rules against freshly fetched windows and returns the alerts to send.
/// Interval and before-reset rules are only as precise as the polling interval.
pub fn evaluate(
    rules: &[Rule],
    windows: &[UsageWindow],
    state: &mut RuleState,
    now: DateTime<Utc>,
) -> Vec<Alert> {
    let mut alerts = Vec::new();

    for rule in rules.iter().filter(|rule| rule.enabled) {
        let alert = |title: String, body: String| Alert {
            rule_id: rule.id.clone(),
            channel: rule.channel,
            title,
            body,
        };

        match rule.kind {
            RuleKind::Threshold {
                provider,
                window,
                percent,
            } => {
                for usage in matching(windows, provider, window) {
                    if crossed_threshold(&rule.id, usage, percent, state) {
                        alerts.push(alert(
                            format!("{}: {:.0}%", usage.label(), usage.utilization),
                            format!(
                                "Atingiu o limiar de {percent}%.{}",
                                reset_suffix(usage, now)
                            ),
                        ));
                    }
                }
            }
            RuleKind::BeforeReset {
                provider,
                window,
                minutes,
            } => {
                for usage in matching(windows, provider, window) {
                    if let Some(remaining) = reset_is_near(&rule.id, usage, minutes, state, now) {
                        alerts.push(alert(
                            format!(
                                "{} reinicia em {}",
                                usage.label(),
                                format_duration(remaining)
                            ),
                            format!("Uso atual: {:.0}%.", usage.utilization),
                        ));
                    }
                }
            }
            RuleKind::Interval { minutes } => {
                if interval_due(&rule.id, minutes, state, now) {
                    alerts.push(alert("Resumo de uso".into(), summary(windows, now)));
                }
            }
        }
    }

    alerts
}

fn matching(
    windows: &[UsageWindow],
    provider: Option<ProviderId>,
    window: Option<WindowKind>,
) -> impl Iterator<Item = &UsageWindow> {
    windows.iter().filter(move |usage| {
        provider.is_none_or(|p| p == usage.provider) && window.is_none_or(|w| w == usage.kind)
    })
}

fn same_cycle(a: DateTime<Utc>, b: DateTime<Utc>) -> bool {
    (a - b).abs() < CYCLE_TOLERANCE
}

fn fired_in_cycle(state: &RuleState, rule_id: &str, key: &str, resets_at: DateTime<Utc>) -> bool {
    state
        .fired
        .get(rule_id)
        .and_then(|windows| windows.get(key))
        .is_some_and(|&fired| same_cycle(fired, resets_at))
}

fn mark_fired(state: &mut RuleState, rule_id: &str, key: String, resets_at: DateTime<Utc>) {
    state
        .fired
        .entry(rule_id.to_owned())
        .or_default()
        .insert(key, resets_at);
}

/// The previous value is the last one seen in the same cycle, or 0 on the first
/// reading of a cycle, so starting the app above the threshold counts as a crossing.
fn crossed_threshold(
    rule_id: &str,
    usage: &UsageWindow,
    percent: f64,
    state: &mut RuleState,
) -> bool {
    let Some(resets_at) = usage.resets_at else {
        return false;
    };
    let key = usage.key();

    let seen = (rule_id.to_owned(), key.clone());
    let previous = state
        .last_seen
        .get(&seen)
        .filter(|observation| same_cycle(observation.resets_at, resets_at))
        .map_or(0.0, |observation| observation.utilization);
    state.last_seen.insert(
        seen,
        Observation {
            utilization: usage.utilization,
            resets_at,
        },
    );

    let crossed = previous < percent && usage.utilization >= percent;
    if !crossed || fired_in_cycle(state, rule_id, &key, resets_at) {
        return false;
    }
    mark_fired(state, rule_id, key, resets_at);
    true
}

/// Returns the time left when the rule should fire. Windows at 0% are skipped,
/// since their reset changes nothing.
fn reset_is_near(
    rule_id: &str,
    usage: &UsageWindow,
    minutes: u32,
    state: &mut RuleState,
    now: DateTime<Utc>,
) -> Option<TimeDelta> {
    let resets_at = usage.resets_at?;
    let remaining = resets_at - now;
    let in_range = remaining > TimeDelta::zero() && remaining <= TimeDelta::minutes(minutes.into());
    if !in_range || usage.utilization <= 0.0 {
        return None;
    }

    let key = usage.key();
    if fired_in_cycle(state, rule_id, &key, resets_at) {
        return None;
    }
    mark_fired(state, rule_id, key, resets_at);
    Some(remaining)
}

/// The first evaluation only records a baseline, so starting the app does not fire.
fn interval_due(rule_id: &str, minutes: u32, state: &mut RuleState, now: DateTime<Utc>) -> bool {
    let last = state.interval_last.entry(rule_id.to_owned()).or_insert(now);
    if now - *last < TimeDelta::minutes(minutes.into()) {
        return false;
    }
    *last = now;
    true
}

fn summary(windows: &[UsageWindow], now: DateTime<Utc>) -> String {
    if windows.is_empty() {
        return "Nenhum dado de uso disponível no momento.".into();
    }
    windows
        .iter()
        .map(|usage| {
            let reset = usage
                .resets_at
                .filter(|&resets_at| resets_at > now)
                .map(|resets_at| format!(" (reinicia em {})", format_duration(resets_at - now)))
                .unwrap_or_default();
            format!("{}: {:.0}%{reset}", usage.label(), usage.utilization)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn reset_suffix(usage: &UsageWindow, now: DateTime<Utc>) -> String {
    usage
        .resets_at
        .filter(|&resets_at| resets_at > now)
        .map(|resets_at| format!(" Reinicia em {}.", format_duration(resets_at - now)))
        .unwrap_or_default()
}

/// Compact duration such as "3d 4h", "2h 10min" or "8min".
fn format_duration(duration: TimeDelta) -> String {
    let total = duration.num_minutes();
    if total < 1 {
        return "menos de 1 min".into();
    }
    let (days, hours, minutes) = (total / (24 * 60), total % (24 * 60) / 60, total % 60);
    match (days, hours, minutes) {
        (d, 0, _) if d > 0 => format!("{d}d"),
        (d, h, _) if d > 0 => format!("{d}d {h}h"),
        (_, h, 0) if h > 0 => format!("{h}h"),
        (_, h, m) if h > 0 => format!("{h}h {m}min"),
        (_, _, m) => format!("{m}min"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::model::Channel;

    fn base() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-18T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn minutes(n: i64) -> DateTime<Utc> {
        base() + TimeDelta::minutes(n)
    }

    fn window(kind: WindowKind, utilization: f64, resets_at: Option<DateTime<Utc>>) -> UsageWindow {
        UsageWindow {
            provider: ProviderId::Claude,
            kind,
            utilization,
            resets_at,
        }
    }

    /// 5h window resetting at `resets_at`.
    fn session(utilization: f64, resets_at: DateTime<Utc>) -> UsageWindow {
        window(WindowKind::Rolling5h, utilization, Some(resets_at))
    }

    fn rule(id: &str, kind: RuleKind) -> Rule {
        Rule {
            id: id.into(),
            enabled: true,
            channel: Channel::Notification,
            kind,
        }
    }

    fn threshold(window: Option<WindowKind>, percent: f64) -> Rule {
        rule(
            "threshold",
            RuleKind::Threshold {
                provider: Some(ProviderId::Claude),
                window,
                percent,
            },
        )
    }

    /// Evaluates one rule against the windows and returns how many alerts fired.
    fn fire_count(
        rule: &Rule,
        windows: &[UsageWindow],
        state: &mut RuleState,
        now: DateTime<Utc>,
    ) -> usize {
        evaluate(std::slice::from_ref(rule), windows, state, now).len()
    }

    #[test]
    fn threshold_fires_once_when_crossed() {
        let rule = threshold(Some(WindowKind::Rolling5h), 80.0);
        let reset = minutes(300);
        let mut state = RuleState::default();

        assert_eq!(
            fire_count(&rule, &[session(70.0, reset)], &mut state, minutes(0)),
            0
        );
        let alerts = evaluate(
            std::slice::from_ref(&rule),
            &[session(85.0, reset)],
            &mut state,
            minutes(3),
        );

        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].rule_id, "threshold");
        assert_eq!(alerts[0].title, "Claude · Sessão (5h): 85%");
    }

    #[test]
    fn threshold_does_not_repeat_while_above() {
        let rule = threshold(None, 80.0);
        let reset = minutes(300);
        let mut state = RuleState::default();

        assert_eq!(
            fire_count(&rule, &[session(85.0, reset)], &mut state, minutes(0)),
            1
        );
        assert_eq!(
            fire_count(&rule, &[session(90.0, reset)], &mut state, minutes(3)),
            0
        );
        assert_eq!(
            fire_count(&rule, &[session(99.0, reset)], &mut state, minutes(6)),
            0
        );
    }

    #[test]
    fn threshold_does_not_refire_after_dropping_in_same_cycle() {
        let rule = threshold(None, 80.0);
        let reset = minutes(300);
        let mut state = RuleState::default();

        assert_eq!(
            fire_count(&rule, &[session(85.0, reset)], &mut state, minutes(0)),
            1
        );
        assert_eq!(
            fire_count(&rule, &[session(60.0, reset)], &mut state, minutes(3)),
            0
        );
        assert_eq!(
            fire_count(&rule, &[session(90.0, reset)], &mut state, minutes(6)),
            0
        );
    }

    #[test]
    fn new_cycle_rearms_threshold() {
        let rule = threshold(None, 80.0);
        let first_reset = minutes(300);
        let next_reset = minutes(600);
        let mut state = RuleState::default();

        assert_eq!(
            fire_count(&rule, &[session(85.0, first_reset)], &mut state, minutes(0)),
            1
        );
        assert_eq!(
            fire_count(
                &rule,
                &[session(10.0, next_reset)],
                &mut state,
                minutes(301)
            ),
            0
        );
        assert_eq!(
            fire_count(
                &rule,
                &[session(81.0, next_reset)],
                &mut state,
                minutes(400)
            ),
            1
        );
    }

    #[test]
    fn first_reading_above_threshold_fires() {
        let rule = threshold(None, 80.0);
        let mut state = RuleState::default();

        assert_eq!(
            fire_count(
                &rule,
                &[session(95.0, minutes(300))],
                &mut state,
                minutes(0)
            ),
            1
        );
    }

    #[test]
    fn restart_with_persisted_state_does_not_repeat() {
        let rule = threshold(None, 80.0);
        let reset = minutes(300);
        let mut state = RuleState::default();
        assert_eq!(
            fire_count(&rule, &[session(85.0, reset)], &mut state, minutes(0)),
            1
        );

        // Only `fired` is persisted; the in-memory history is lost on restart.
        let mut restarted = RuleState::with_fired(state.fired.clone());

        assert_eq!(
            fire_count(&rule, &[session(88.0, reset)], &mut restarted, minutes(10)),
            0
        );
    }

    #[test]
    fn reset_jitter_is_the_same_cycle() {
        let rule = threshold(None, 80.0);
        let reset = minutes(300);
        let jittered = reset + TimeDelta::microseconds(25);
        let mut state = RuleState::default();

        assert_eq!(
            fire_count(&rule, &[session(85.0, reset)], &mut state, minutes(0)),
            1
        );
        assert_eq!(
            fire_count(&rule, &[session(70.0, jittered)], &mut state, minutes(3)),
            0
        );
        assert_eq!(
            fire_count(&rule, &[session(90.0, jittered)], &mut state, minutes(6)),
            0
        );
    }

    #[test]
    fn any_target_tracks_each_window_independently() {
        let rule = rule(
            "any",
            RuleKind::Threshold {
                provider: None,
                window: None,
                percent: 80.0,
            },
        );
        let week_reset = minutes(5000);
        let mut state = RuleState::default();

        let first = [
            session(85.0, minutes(300)),
            window(WindowKind::Rolling7d, 50.0, Some(week_reset)),
        ];
        let alerts = evaluate(std::slice::from_ref(&rule), &first, &mut state, minutes(0));
        assert_eq!(alerts.len(), 1);
        assert!(alerts[0].title.contains("Sessão"));

        let second = [
            session(90.0, minutes(300)),
            window(WindowKind::Rolling7d, 82.0, Some(week_reset)),
        ];
        let alerts = evaluate(std::slice::from_ref(&rule), &second, &mut state, minutes(3));
        assert_eq!(alerts.len(), 1);
        assert!(alerts[0].title.contains("Semanal"));
    }

    #[test]
    fn specific_window_ignores_other_windows() {
        let rule = threshold(Some(WindowKind::Rolling7d), 80.0);
        let mut state = RuleState::default();

        assert_eq!(
            fire_count(
                &rule,
                &[session(95.0, minutes(300))],
                &mut state,
                minutes(0)
            ),
            0
        );
    }

    #[test]
    fn threshold_without_reset_time_does_not_fire() {
        let rule = threshold(None, 80.0);
        let mut state = RuleState::default();

        let windows = [window(WindowKind::Rolling5h, 95.0, None)];

        assert_eq!(fire_count(&rule, &windows, &mut state, minutes(0)), 0);
    }

    #[test]
    fn reaching_exactly_the_threshold_fires() {
        let rule = threshold(None, 80.0);
        let mut state = RuleState::default();

        assert_eq!(
            fire_count(
                &rule,
                &[session(80.0, minutes(300))],
                &mut state,
                minutes(0)
            ),
            1
        );
    }

    #[test]
    fn before_reset_fires_once_and_rearms() {
        let rule = rule(
            "before",
            RuleKind::BeforeReset {
                provider: None,
                window: None,
                minutes: 15,
            },
        );
        let first_reset = minutes(20);
        let next_reset = minutes(320);
        let mut state = RuleState::default();

        assert_eq!(
            fire_count(&rule, &[session(40.0, first_reset)], &mut state, minutes(0)),
            0
        );
        let alerts = evaluate(
            std::slice::from_ref(&rule),
            &[session(40.0, first_reset)],
            &mut state,
            minutes(6),
        );
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].title, "Claude · Sessão (5h) reinicia em 14min");
        assert_eq!(
            fire_count(&rule, &[session(45.0, first_reset)], &mut state, minutes(9)),
            0
        );

        assert_eq!(
            fire_count(
                &rule,
                &[session(30.0, next_reset)],
                &mut state,
                minutes(310)
            ),
            1
        );
    }

    #[test]
    fn before_reset_ignores_past_resets_and_unused_windows() {
        let rule = rule(
            "before",
            RuleKind::BeforeReset {
                provider: None,
                window: None,
                minutes: 15,
            },
        );
        let mut state = RuleState::default();

        assert_eq!(
            fire_count(&rule, &[session(40.0, minutes(-1))], &mut state, minutes(0)),
            0
        );
        assert_eq!(
            fire_count(&rule, &[session(0.0, minutes(10))], &mut state, minutes(0)),
            0
        );
    }

    #[test]
    fn interval_fires_every_n_minutes_after_baseline() {
        let rule = rule("interval", RuleKind::Interval { minutes: 10 });
        let windows = [session(42.0, minutes(300))];
        let mut state = RuleState::default();

        assert_eq!(fire_count(&rule, &windows, &mut state, minutes(0)), 0);
        assert_eq!(fire_count(&rule, &windows, &mut state, minutes(9)), 0);
        let alerts = evaluate(
            std::slice::from_ref(&rule),
            &windows,
            &mut state,
            minutes(10),
        );
        assert_eq!(alerts.len(), 1);
        assert_eq!(
            alerts[0].body,
            "Claude · Sessão (5h): 42% (reinicia em 4h 50min)"
        );
        assert_eq!(fire_count(&rule, &windows, &mut state, minutes(15)), 0);
        assert_eq!(fire_count(&rule, &windows, &mut state, minutes(20)), 1);
    }

    #[test]
    fn disabled_rules_never_fire() {
        let mut threshold = threshold(None, 80.0);
        threshold.enabled = false;
        let mut interval = rule("interval", RuleKind::Interval { minutes: 1 });
        interval.enabled = false;
        let rules = [threshold, interval];
        let mut state = RuleState::default();

        for n in 0..5 {
            let alerts = evaluate(
                &rules,
                &[session(95.0, minutes(300))],
                &mut state,
                minutes(n),
            );
            assert!(alerts.is_empty());
        }
    }

    #[test]
    fn forgetting_a_rule_rearms_it() {
        let rule = threshold(None, 80.0);
        let reset = minutes(300);
        let mut state = RuleState::default();
        assert_eq!(
            fire_count(&rule, &[session(85.0, reset)], &mut state, minutes(0)),
            1
        );

        state.forget(&rule.id);

        assert_eq!(
            fire_count(&rule, &[session(85.0, reset)], &mut state, minutes(3)),
            1
        );
    }

    #[test]
    fn formats_durations() {
        assert_eq!(format_duration(TimeDelta::seconds(30)), "menos de 1 min");
        assert_eq!(format_duration(TimeDelta::minutes(8)), "8min");
        assert_eq!(format_duration(TimeDelta::minutes(120)), "2h");
        assert_eq!(format_duration(TimeDelta::minutes(130)), "2h 10min");
        assert_eq!(format_duration(TimeDelta::days(3)), "3d");
        assert_eq!(format_duration(TimeDelta::minutes(3 * 1440 + 250)), "3d 4h");
    }
}
