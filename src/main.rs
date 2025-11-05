use confy::ConfyError;
use cursive::{self};
mod requests;
mod tui;
mod config;

use rust_i18n::t;

rust_i18n::i18n!();

fn main() {
    rust_i18n::set_locale("ru");
    let cfg: Result<config::LoungeConfig, ConfyError> = config::get_config();

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
