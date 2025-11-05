use confy::ConfyError;
use cursive::{
    Cursive,
    align::Align,
    event::Event,
    menu::Tree,
    view::{Finder, Resizable},
    views::{Dialog, LinearLayout, TextView},
};
use rust_i18n::t;

use crate::config::{self, LoungeConfig};
mod grades;
mod schedules;
pub mod setup;

rust_i18n::i18n!("locales");

pub fn main_screen(s: &mut Cursive) {
    for event in [
        Event::Key(cursive::event::Key::F2),
        Event::Key(cursive::event::Key::F1),
        Event::Key(cursive::event::Key::Esc),
    ] {
        s.clear_global_callbacks(event);
    }

    s.add_global_callback(Event::Key(cursive::event::Key::F1), |s| {
        s.set_autohide_menu(true);
        s.add_layer(schedules::schedules_view());
        
    });
    s.add_global_callback(Event::Key(cursive::event::Key::F2), |s| {
        s.set_autohide_menu(true);
        s.add_layer(grades::grades_view());
    });

    // s.add_layer(
    //     LinearLayout::vertical()
    //         .child(TextView::new(t!("app_name")).center())
    //         .child(TextView::new(t!("about_description")))
    //         .child(TextView::new(t!("about_developer"))).full_screen(),
    // );

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
            s.set_autohide_menu(true);
            s.add_layer(schedules::schedules_view());
        });
    s.menubar()
        .add_leaf(format!("[F2] {}", t!("sections.grades")), |s| {
            s.set_autohide_menu(true);
            s.add_layer(grades::grades_view());
        });
    s.menubar().add_delimiter();

    let settings_tree = Tree::new()
        .leaf(t!("actions.specify_level_group"), |s| {
            setup::level_chooser(s)
        })
        .leaf(t!("actions.specify_group"), |s| setup::group_chooser(s))
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
    s.pop_layer();
    s.add_layer(
        Dialog::text(t!("welcome"))
            .title(t!("app_name"))
            .button(t!("actions.next"), |s| setup::setup_level(s)),
    );
}

pub fn error(s: &mut Cursive, error: ConfyError) {
    s.pop_layer();
    s.add_layer(
        Dialog::text(
            "Произошла ошибка при попытке прочесть или записать конфигурационный файл:\n"
                .to_owned()
                + &error.to_string()
                + "\n"
                + config::get_store_path().unwrap().to_str().unwrap(),
        )
        .title("Ошибка")
        .button("Выход", |s| s.quit())
        .button(
            "Сброс настроек (программа закроется)",
            |s| {
                config::store_config(LoungeConfig::default()).unwrap();
                s.quit();
            },
        ),
    );
}
