use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::overlay;
use crate::rules::Alert;

pub fn dispatch<R: Runtime>(app: &AppHandle<R>, alerts: Vec<Alert>) {
    for alert in alerts.iter().filter(|alert| alert.channel.notifies()) {
        let shown = app
            .notification()
            .builder()
            .title(&alert.title)
            .body(&alert.body)
            .show();
        if let Err(error) = shown {
            eprintln!("could not show notification: {error}");
        }
    }

    let for_overlay = alerts
        .into_iter()
        .filter(|alert| alert.channel.shows_overlay())
        .collect();
    overlay::push_alerts(app, for_overlay);
}
