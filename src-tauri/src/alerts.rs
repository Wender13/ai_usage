use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::rules::Alert;

pub fn dispatch<R: Runtime>(app: &AppHandle<R>, alerts: &[Alert]) {
    for alert in alerts {
        if alert.channel.notifies() {
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
        // The overlay channel has no effect until the overlay window exists.
    }
}
