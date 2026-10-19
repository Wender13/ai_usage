//! Command-line flags. A second launch forwards its flags to the running instance
//! (single-instance plugin), so `ai_usage --toggle` works as a keyboard shortcut command.

use tauri::{AppHandle, Manager, Runtime};

use crate::state::AppState;
use crate::{overlay, windows};

/// Starts with only the tray icon; used by autostart.
const HIDDEN: &str = "--hidden";
/// Shows the overlay without the main window.
const OVERLAY: &str = "--overlay";
/// Turns the overlay on or off.
const TOGGLE: &str = "--toggle";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Launch {
    Main,
    Hidden,
    Overlay,
    Toggle,
}

impl Launch {
    /// Reads the flags after the program name; when several are given, the one that
    /// does the most wins (toggle, then overlay, then hidden).
    pub fn parse(args: &[String]) -> Self {
        let has = |flag: &str| args.iter().skip(1).any(|arg| arg == flag);
        if has(TOGGLE) {
            Self::Toggle
        } else if has(OVERLAY) {
            Self::Overlay
        } else if has(HIDDEN) {
            Self::Hidden
        } else {
            Self::Main
        }
    }

    pub fn apply<R: Runtime>(self, app: &AppHandle<R>) {
        let overlay_on = match self {
            Self::Main => {
                windows::show_main(app);
                return;
            }
            Self::Hidden => return,
            Self::Overlay => true,
            Self::Toggle => !app.state::<AppState>().settings().overlay_enabled,
        };
        if let Err(error) = overlay::set_enabled(app, overlay_on) {
            eprintln!("{error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Launch {
        let args: Vec<String> = std::iter::once("ai_usage")
            .chain(args.iter().copied())
            .map(String::from)
            .collect();
        Launch::parse(&args)
    }

    #[test]
    fn parses_each_flag() {
        assert_eq!(parse(&[]), Launch::Main);
        assert_eq!(parse(&["--hidden"]), Launch::Hidden);
        assert_eq!(parse(&["--overlay"]), Launch::Overlay);
        assert_eq!(parse(&["--toggle"]), Launch::Toggle);
    }

    #[test]
    fn strongest_flag_wins_and_unknown_args_are_ignored() {
        assert_eq!(parse(&["--hidden", "--overlay"]), Launch::Overlay);
        assert_eq!(
            parse(&["--overlay", "--toggle", "--hidden"]),
            Launch::Toggle
        );
        assert_eq!(parse(&["--verbose"]), Launch::Main);
    }

    #[test]
    fn program_name_is_not_a_flag() {
        assert_eq!(Launch::parse(&["--toggle".to_owned()]), Launch::Main);
    }
}
