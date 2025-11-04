use chrono::Utc;
use confy::ConfyError;
use cursive::{self, theme::Theme};
use serde_derive::{Deserialize, Serialize};

mod requests;
mod tui;

use rust_i18n::t;

rust_i18n::i18n!();

// Init translations for current crate.
// This will load Configuration using the `[package.metadata.i18n]` section in `Cargo.toml` if exists.
// Or you can pass arguments by `i18n!` to override it.

#[derive(Serialize, Deserialize)]
pub struct LoungeConfig {
    group_id: String,
    level_id: String,
    pin: String,
    last_name: String,
    setup_passed: bool,
    selected_date: i64,
    theme: u8,
}

impl ::std::default::Default for LoungeConfig {
    fn default() -> Self {
        Self {
            group_id: "".to_string(),
            level_id: "".to_string(),
            pin: "".to_string(),
            last_name: "".to_string(),
            setup_passed: false,
            selected_date: Utc::now().timestamp(),
            theme: 0,
        }
    }
}

fn main() {
    rust_i18n::set_locale("ru");

    let cfg: Result<LoungeConfig, ConfyError> = confy::load("lounge-tui", None);
    let mut siv = cursive::default();
    siv.set_window_title(t!("app_name"));

    match cfg {
        Ok(config) => {
            tui::setup::load_theme(&mut siv, &config.theme);

            if config.setup_passed {
                tui::main_screen(&mut siv)
            } else {
                tui::welcome(&mut siv)
            }
        }
        Err(error) => {
            tui::error(&mut siv, error);
        }
    }

    siv.run();
}
