mod config;
mod grades;
mod schedules;
mod setup;

use confy::ConfyError;
use cursive::{self};
use rust_i18n::t;

rust_i18n::i18n!();

use cursive::{
    Cursive,
    align::Align,
    event::Event,
    menu::Tree,
    view::Resizable,
    views::{Dialog, LinearLayout, TextView},
};

use config::LoungeConfig;

use crate::{grades::grades_view, schedules::schedules_view};

pub fn main_screen(s: &mut Cursive) {
    for event in [
        Event::Key(cursive::event::Key::F2),
        Event::Key(cursive::event::Key::F1),
    ] {
        s.clear_global_callbacks(event);
    }

    s.add_global_callback(Event::Key(cursive::event::Key::F1), |s| {
        let schedules_view = schedules_view(s);
        s.set_autohide_menu(true);
        s.add_layer(schedules_view);
    });
    s.add_global_callback(Event::Key(cursive::event::Key::F2), |s| {
        let grades_view = grades_view(s);
        s.set_autohide_menu(true);
        s.add_layer(grades_view);
    });

    s.screen_mut().add_transparent_layer(
        LinearLayout::vertical()
            .child(
                TextView::new(include_str!("logo.txt"))
                    .align(Align::center())
                    .no_wrap(),
            )
            .child(TextView::new(t!("about_description_1")).align(Align::center()))
            .child(TextView::new(t!("about_description_2")).align(Align::center()))
            .child(TextView::new(t!("about_developer")).align(Align::center()))
            .child(TextView::new(format!("\n{}", t!("about_lol"))).align(Align::center()))
            .full_screen(),
    );

    // MARK: Menubar
    s.menubar().clear();
    s.set_autohide_menu(false);
    s.menubar()
        .add_leaf(format!("[F1] {}", t!("sections.schedules")), |s| {
            let schedules_view = schedules_view(s);
            s.set_autohide_menu(true);
            s.add_layer(schedules_view);
        });
    s.menubar()
        .add_leaf(format!("[F2] {}", t!("sections.grades")), |s| {
            let grades_view = grades_view(s);
            s.set_autohide_menu(true);
            s.add_layer(grades_view);
        });
    s.menubar().add_delimiter();

    let settings_tree = Tree::new()
        .leaf(t!("actions.specify_level_group"), |s| {
            setup::level_chooser(s, false)
        })
        .leaf(t!("actions.specify_group"), |s| setup::group_chooser(s, false))
        .delimiter()
        .leaf(t!("actions.specify_grades_data"), |s| {
            setup::grades_settings(s)
        })
        .delimiter()
        .leaf(t!("actions.specify_theme"), |s| {
            setup::select_theme(s);
        });

    s.menubar()
        .add_subtree(format!("[▼] {}", t!("sections.settings")), settings_tree);
}

pub fn welcome(s: &mut Cursive) {
    s.add_layer(
        Dialog::text(t!("welcome"))
            .title(t!("app_name"))
            .button(t!("actions.next"), |s| {
                s.pop_layer();
                setup::level_chooser(s, true);
            }),
    );
}

pub fn error_dialog(s: &mut Cursive, error: ConfyError) {
    s.pop_layer();
    s.add_layer(
        Dialog::text(t!(
            "errors.config",
            e = &error.to_string(),
            path = config::get_store_path().unwrap().to_str().unwrap()
        ))
        .title(t!("error"))
        .button(t!("actions.exit"), |s| s.quit())
        .button(
            t!("config_reset"),
            |s| {
                config::store_config(LoungeConfig::default()).unwrap();
                s.quit();
            },
        ),
    );
}

fn main() {
    rust_i18n::set_locale("ru");
    let cfg: Result<config::LoungeConfig, ConfyError> = config::get_config();

    let mut siv = cursive::default();
    siv.set_window_title(t!("app_name"));

    match cfg {
        Ok(config) => {
            setup::load_theme(&mut siv, &config.theme);

            if config.setup_passed {
                main_screen(&mut siv)
            } else {
                welcome(&mut siv)
            }
        }
        Err(error) => {
            error_dialog(&mut siv, error);
        }
    }

    siv.run();
}
