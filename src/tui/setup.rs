use chrono::Utc;
use chrono::prelude::*;
use cursive::align::HAlign;
use cursive::reexports::enumset::__internal::EnumSetTypeRepr;
use cursive::theme::Theme;
use cursive::view::Nameable;
use cursive::views::EditView;
use cursive::views::LinearLayout;
use cursive::{
    Cursive,
    view::Scrollable,
    views::{Dialog, SelectView, TextView},
};
use cursive_calendar_view::{CalendarView, EnglishLocale, ViewMode};
use tokio::runtime::Runtime;

use crate::config;
use crate::tui::schedules;
use crate::tui::{self};
use crate::{requests};
use rust_i18n::t;

rust_i18n::i18n!("locales");

pub fn load_theme(s: &mut Cursive, theme: &u8) {
    match theme {
        1 => s.load_toml(include_str!("themes/monokai.toml")).unwrap(),
        2 => s
            .load_toml(include_str!("themes/catpuccin-mocha.toml"))
            .unwrap(),
        3 => s
            .load_toml(include_str!("themes/catpuccin-latte.toml"))
            .unwrap(),
        4 => s
            .load_toml(include_str!("themes/monokai-pro.toml"))
            .unwrap(),
        _ => s.set_theme(Theme::default()),
    };
}

pub fn select_theme(s: &mut Cursive) {
    let cfg = config::get_config().unwrap();

    let select = SelectView::new()
        .h_align(HAlign::Center)
        .autojump()
        .item("Default", 0)
        .item("Monokai", 1)
        .item("Catpuccin Mocha", 2)
        .item("Catpuccin Latte", 3)
        .item("Monokai Pro", 4)
        .on_select(|s, item| {
            load_theme(s, item);
        })
        .on_submit(| s: &mut Cursive, item: &u8 | {
            let mut cfg = config::get_config().unwrap();
            cfg.theme = *item;
            config::store_config(cfg).unwrap();
            s.pop_layer();
        })
        .selected(cfg.theme.to_usize());

    s.add_layer(
        Dialog::around(select.scrollable())
            .title(t!("prompts.specify_theme")),
    );
}

pub fn select_date(s: &mut Cursive) {
    let cfg = config::get_config().unwrap();

    let mut calendar = CalendarView::<Utc, EnglishLocale>::new(
        Utc.timestamp_opt(cfg.selected_date, 0).unwrap().date(),
    );

    calendar.set_view_mode(ViewMode::Year);
    calendar.set_earliest_date(Some(Utc.ymd(2023, 1, 1)));
    calendar.set_latest_date(Some(Utc.ymd(2040, 12, 31)));
    calendar.set_show_iso_weeks(true);

    calendar.set_on_submit(move |siv: &mut Cursive, date: &Date<Utc>| {
        let mut cfg = config::get_config().unwrap();
        cfg.selected_date = Date::<Utc>::from(*date)
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .unwrap()
            .timestamp();
        config::store_config(cfg).unwrap();
        siv.pop_layer();

        siv.add_layer(schedules::schedules_view());
    });

    s.add_layer(Dialog::around(calendar.with_name("calendar")).title(t!("prompts.specify_date")));
}

pub fn setup_level(s: &mut Cursive) {
    s.pop_layer();

    let rt: Runtime = Runtime::new().unwrap();
    let levels_result = rt.block_on(requests::get_levels());

    match levels_result {
        Ok(levels) => {
            let mut select = SelectView::<String>::new()
                .h_align(HAlign::Center)
                .autojump()
                .on_submit(level_submit);

            for level in levels {
                select.add_item(level.label, level.id);
            }

            s.add_layer(Dialog::around(select.scrollable()).title(t!("prompts.specify_level")));
        }
        Err(err) => some_error(s, err),
    }
}

pub fn some_error(s: &mut Cursive, e: String) {
    s.pop_layer();
    s.add_layer(
        Dialog::around(TextView::new(t!("errors.some", e = e)))
            .button(t!("actions.exit"), |s| s.quit()),
    );
}

fn level_submit(s: &mut Cursive, id: &str) {
    let mut cfg = config::get_config().unwrap();
    cfg.level_id = id.to_string();
    config::store_config(cfg).unwrap();

    s.pop_layer();
    setup_group(s);
}

fn setup_group(s: &mut Cursive) {
    let cfg = config::get_config().unwrap();
    let rt: Runtime = Runtime::new().unwrap();
    let groups_result = rt.block_on(requests::get_groups(&cfg.level_id));

    match groups_result {
        Ok(groups) => {
            let mut select = SelectView::<String>::new()
                .h_align(HAlign::Center)
                .autojump()
                .on_submit(group_submit);

            for group in groups {
                select.add_item(group.label, group.id);
            }

            s.add_layer(
                Dialog::around(select.scrollable())
                    .title(t!("prompts.specify_group"))
                    .button(t!("actions.back"), |s| setup_level(s)),
            );
        }
        Err(err) => some_error(s, err),
    }
}

fn group_submit(s: &mut Cursive, id: &str) {
    let mut cfg = config::get_config().unwrap();
    cfg.group_id = id.to_string();
    cfg.setup_passed = true;
    config::store_config(cfg).unwrap();

    s.pop_layer();
    prompt_grades_setup(s);
}

fn prompt_grades_setup(s: &mut Cursive) {
    let dialog = Dialog::new()
        .content(TextView::new(t!("prompts.setup_grades")))
        .title(t!("grades_setup"))
        .button(t!("actions.yes"), |s| {
            s.pop_layer();
            grades_settings(s);
        })
        .button(t!("actions.no"), |s| {
            s.pop_layer();
            tui::main_screen(s)
        });

    tui::main_screen(s);
    s.add_layer(dialog);
}

pub fn level_chooser(s: &mut Cursive) {
    let rt: Runtime = Runtime::new().unwrap();
    let levels_result = rt.block_on(requests::get_levels());

    match levels_result {
        Ok(levels) => {
            let mut select = SelectView::<String>::new()
                .h_align(HAlign::Center)
                .autojump()
                .on_submit(|s: &mut Cursive, v: &str| {
                    let mut cfg = config::get_config().unwrap();
                    cfg.level_id = v.to_string();
                    config::store_config(cfg).unwrap();
                    s.pop_layer();
                    group_chooser(s);
                });

            for level in levels {
                select.add_item(level.label, level.id);
            }

            s.add_layer(
                Dialog::around(select.scrollable())
                    .title(t!("prompts.specify_level"))
                    .dismiss_button(t!("actions.cancel")),
            );
        }
        Err(err) => some_error(s, err),
    }
}

pub fn group_chooser(s: &mut Cursive) {
    let cfg = config::get_config().unwrap();
    let rt: Runtime = Runtime::new().unwrap();
    let groups_result = rt.block_on(requests::get_groups(&cfg.level_id));

    match groups_result {
        Ok(groups) => {
            let mut select = SelectView::<String>::new()
                .h_align(HAlign::Center)
                .autojump()
                .on_submit(|s, v: &str| {
                    let mut cfg = config::get_config().unwrap();
                    cfg.group_id = v.to_string();
                    config::store_config(cfg).unwrap();
                    s.pop_layer();
                });

            for group in groups {
                select.add_item(group.label, group.id);
            }

            s.add_layer(
                Dialog::around(select.scrollable())
                    .title(t!("prompts.specify_group"))
                    .dismiss_button(t!("actions.cancel")),
            );
        }
        Err(err) => some_error(s, err),
    }
}

pub fn grades_settings(s: &mut Cursive) {
    let cfg = config::get_config().unwrap();
    let pin = EditView::new().content(cfg.pin).with_name("pin-input");
    let last_name = EditView::new()
        .content(cfg.last_name)
        .with_name("last_name_input");

    let dialog = Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new(t!("grades_setup_detailed")))
            .child(TextView::new(t!("prompts.enter_pin")))
            .child(pin)
            .child(TextView::new(t!("prompts.enter_last_name")))
            .child(last_name),
    )
    .button("Применить", |s| {
        let pin = s.find_name::<EditView>("pin-input").unwrap();
        let last_name = s.find_name::<EditView>("last_name_input").unwrap();
        let mut cfg = config::get_config().unwrap();
        cfg.pin = pin.get_content().to_string();
        cfg.last_name = last_name.get_content().to_string();
        config::store_config(cfg).unwrap();
        s.pop_layer();
    })
    .dismiss_button(t!("actions.cancel"))
    .title(t!("grades_setup"));
    s.add_layer(dialog);
}
