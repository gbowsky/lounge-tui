use crate::{
    config, requests::{self}, tui::{self, setup}
};
use chrono::{Days, TimeZone, Utc};
use cursive::view::Nameable;
use cursive::{
    align::Align,
    view::{Margins, Scrollable},
    views::{Button, Dialog, LinearLayout, NamedView, PaddedView, TextView},
};
use tokio::runtime::Runtime;

use rust_i18n::t;

rust_i18n::i18n!("locales");


fn schedules_additional_type_to_text(text: &str) -> String {
    return t!("schedules_type.".to_owned() + text).to_string()
}

pub fn schedules_view() -> NamedView<Dialog> {
    let mut schedules_list = LinearLayout::vertical();
    let cfg = config::get_config().unwrap();
    let date = Utc.timestamp(cfg.selected_date, 0);
    let date_from_formatted = date.format("%d.%m.%Y").to_string();
    let date_to = date.checked_add_days(Days::new(7)).unwrap();
    let date_to = date_to.format("%d.%m.%Y").to_string();
    let rt = Runtime::new().unwrap();
    let schedules_result = rt.block_on(requests::get_schedules(
        &date_from_formatted,
        &date_to,
        &cfg.group_id,
    ));

    match schedules_result {
        Ok(schedules) => {
            for day in schedules {
                let mut lessons = LinearLayout::vertical();

                for lesson in day.lessons {
                    let place_text = if !lesson.additional.online {
                        lesson
                            .additional
                            .classroom
                            .unwrap_or(t!("unknown").to_string()).to_string()
                    } else {
                        t!("online").to_string()
                    };

                    let place_char = if lesson.additional.online {
                        "◇ "
                    } else {
                        "◆ "
                    };

                    lessons.add_child(TextView::new(
                        place_char.to_owned()
                            + &lesson.time_start
                            + " - "
                            + &lesson.time_end
                            + " · "
                            + &place_text,
                    ));
                    lessons.add_child(TextView::new(lesson.text));

                    let mut url_bar = LinearLayout::horizontal();
                    url_bar.add_child(TextView::new(schedules_additional_type_to_text(
                        &lesson.additional.r#type.to_text(),
                    )));
                    if !lesson.urls.is_empty() {
                        url_bar.add_child(TextView::new(", "));
                    }
                    for link in lesson.urls {
                        url_bar.add_child(Button::new(link.text, move |_s| {
                            let _ = open::that(link.url.clone());
                        }));
                    }
                    lessons.add_child(url_bar);
                }

                schedules_list.add_child(
                    TextView::new(
                        "===> ".to_owned()
                            + &day.day
                            + "."
                            + &day.month
                            + " "
                            + &day.week_day
                            + " <===",
                    )
                    .align(Align::center()),
                );

                schedules_list.add_child(PaddedView::new(
                    Margins {
                        left: 1,
                        right: 0,
                        top: 0,
                        bottom: 1,
                    },
                    lessons,
                ));
            }
        }
        Err(err) => {
            schedules_list.add_child(TextView::new(t!("error.schedules", e = &err)));
        }
    }

    return Dialog::around(
        LinearLayout::vertical()
            .child(PaddedView::new(
                Margins {
                    left: 1,
                    right: 1,
                    top: 1,
                    bottom: 1,
                },
                LinearLayout::horizontal()
                    .child(TextView::new(date_from_formatted + " - " + &date_to))
                    .child(TextView::new(" "))
                    .child(Button::new(t!("schedules.change_date"), |s| {
                        setup::select_date(s);
                    }))
                    .child(TextView::new(" "))
                    .child(Button::new(t!("schedules.today"), |s| {
                        let mut cfg = config::get_config().unwrap();
                        cfg.selected_date = Utc::now().timestamp();
                        config::store_config(cfg).unwrap();
                        tui::main_screen(s);
                        s.add_layer(schedules_view());
                    })),
            ))
            .child(schedules_list.scrollable()),
    )
    .title(t!("sections.schedules"))
    .dismiss_button(t!("actions.close"))
    .with_name("schedules");
}
