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
use cursive_async_view::AsyncView;
use cursive_calendar_view::{CalendarView, EnglishLocale, ViewMode};
use tokio::runtime::Runtime;

use crate::config;
use crate::main_screen;
use crate::schedules::schedules_view;
use rust_i18n::t;

rust_i18n::i18n!();

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
        .on_submit(|s: &mut Cursive, item: &u8| {
            let mut cfg = config::get_config().unwrap();
            cfg.theme = *item;
            config::store_config(cfg).unwrap();
            s.pop_layer();
        })
        .selected(cfg.theme.to_usize());

    s.add_layer(Dialog::around(select.scrollable()).title(t!("prompts.specify_theme")));
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

        let schedules_view = schedules_view(siv);
        siv.add_layer(schedules_view);
    });

    s.add_layer(Dialog::around(calendar.with_name("calendar")).title(t!("prompts.specify_date")));
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
            main_screen(s);
        });

    main_screen(s);
    s.add_layer(dialog);
}

fn level_select_view(result: Result<Vec<lounge_parser::lists::BasicItem>, String>, is_setup: bool) -> Dialog {
    let mut dialog = Dialog::new()
        .title(t!("prompts.specify_level"))
        .dismiss_button(t!("actions.cancel"));

    match result {
        Ok(levels) => {
            let mut select = SelectView::<String>::new()
                .h_align(HAlign::Center)
                .autojump()
                .on_submit(move |s: &mut Cursive, v: &str| {
                    let mut cfg = config::get_config().unwrap();
                    cfg.level_id = v.to_string();
                    config::store_config(cfg).unwrap();
                    s.pop_layer();
                    group_chooser(s, is_setup);
                });

            for level in levels {
                select.add_item(level.label, level.id);
            }

            if is_setup {
                dialog.remove_button(0);
            }

            dialog.set_content(select.scrollable());
        }
        Err(err) => {
            dialog.set_content(TextView::new(t!("errors.some", e = err)));
        }
    }

    dialog
}

pub fn level_chooser(s: &mut Cursive, is_setup: bool) {
    let async_view = AsyncView::new_with_bg_creator(
        s,
        move || {
            let rt: Runtime = Runtime::new().unwrap();
            let levels_result = rt.block_on(lounge_parser::get_levels());

            // enough blocking, let's show the content
            Ok(levels_result)
        },
        move |result| level_select_view(result, is_setup),
    ); // create a text view from the string

    s.add_layer(async_view.with_width(40));
}

fn group_select_view(result: Result<Vec<lounge_parser::lists::BasicItem>, String>, is_setup: bool) -> Dialog {
    let mut dialog = Dialog::new()
        .title(t!("prompts.specify_group"))
        .dismiss_button(t!("actions.cancel"));

    match result {
        Ok(groups) => {
            let mut select = SelectView::<String>::new()
                .h_align(HAlign::Center)
                .autojump()
                .on_submit(move |s, v: &str| {
                    let mut cfg = config::get_config().unwrap();
                    cfg.group_id = v.to_string();
                    s.pop_layer();

                    if is_setup {
                        cfg.setup_passed = true;
                        prompt_grades_setup(s);
                    }

                    config::store_config(cfg).unwrap();
                });

            for group in groups {
                select.add_item(group.label, group.id);
            }

            if is_setup {
                dialog.remove_button(0);

                dialog.add_button(t!("actions.back"), move |s| {
                    s.pop_layer();
                    level_chooser(s, is_setup);
                });
            }

            dialog.set_content(select.scrollable());
        }
        Err(err) => {
            dialog.set_content(TextView::new(t!("errors.some", e = err)));
        }
    }

    dialog
}

pub fn group_chooser(s: &mut Cursive, is_setup: bool) {
    let cfg = config::get_config().unwrap();
    let async_view = AsyncView::new_with_bg_creator(
        s,
        move || {
            let rt: Runtime = Runtime::new().unwrap();
            let groups_result = rt.block_on(lounge_parser::get_groups(&cfg.level_id));

            // enough blocking, let's show the content
            Ok(groups_result)
        },
        move |result| group_select_view(result, is_setup),
    ); // create a text view from the string

    s.add_layer(async_view.with_width(40));
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
