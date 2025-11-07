use std::env;

use crate::{config, setup};
use chrono::{Days, TimeZone, Utc};
use cursive::{
    Cursive,
    align::Align,
    theme::{Effects, Style},
    utils::markup::StyledString,
    view::{Margins, Scrollable},
    views::{Button, Dialog, LinearLayout, NamedView, PaddedView, TextView},
};
use cursive::{
    theme::{BaseColor, ColorStyle, PaletteStyle},
    view::{Nameable, Resizable},
};
use lounge_parser::{
    get_schedules,
    schedules::{LessonItem, LessonUrl, additional::LessonType},
};
use tokio::runtime::Runtime;

use rust_i18n::t;

rust_i18n::i18n!();

fn schedules_additional_type_to_text(text: &str) -> String {
    return t!("schedules_type.".to_owned() + text).to_string();
}

fn schedules_ssh_link_dialog(s: &mut Cursive, url: &str) {
    let hint_text = t!("ssh_link_hint", url = url);
    let dialog = Dialog::around(TextView::new(hint_text).align(Align::center()))
        .dismiss_button(t!("actions.close"));

    s.add_layer(dialog);
}

fn schedules_links_view(urls: Vec<LessonUrl>) -> LinearLayout {
    let mut url_button_bar = LinearLayout::horizontal();

    for link in urls {
        let url_button = Button::new(link.text, move |s| match env::var("SSH_CONNECTION") {
            Ok(_) => {
                schedules_ssh_link_dialog(s, &link.url);
            }
            Err(_) => {
                let _ = open::that(link.url.clone());
            }
        });
        url_button_bar.add_child(url_button);
    }

    url_button_bar
}

fn schedules_type_difficulty_view(r#type: &LessonType) -> LinearLayout {
    let color = match r#type {
        LessonType::Lecture => ColorStyle::front(BaseColor::Green),
        LessonType::Practice => ColorStyle::front(BaseColor::Green),
        LessonType::Meeting => ColorStyle::front(BaseColor::Yellow),
        LessonType::SubjectReport => ColorStyle::front(BaseColor::Yellow),
        LessonType::SubjectReportWithGrade => ColorStyle::front(BaseColor::Yellow),
        LessonType::Consultation => ColorStyle::front(BaseColor::Yellow),
        LessonType::Exam => ColorStyle::front(BaseColor::Red),
        _ => ColorStyle::front(BaseColor::White),
    };

    LinearLayout::vertical()
        .child(TextView::new("║ ").style(color))
        .child(TextView::new("║ ").style(color))
}

fn schedules_lesson_place_str(lesson: &LessonItem) -> String {
    let lesson_place = if !lesson.additional.online {
        lesson
            .additional
            .classroom.clone()
            .unwrap_or(t!("unknown").to_string())
            .to_string()
    } else {
        t!("online").to_string()
    };
    lesson_place
}

fn lesson_times_view(lesson: &LessonItem) -> LinearLayout {
    let lesson_times = LinearLayout::vertical()
        .child(TextView::new(&lesson.time_start).align(Align::center_right()))
        .child(
            TextView::new(&lesson.time_end)
                .align(Align::center_right())
                .style(PaletteStyle::Tertiary),
        );
    lesson_times
}

fn lesson_type_place_view(lesson_type: String, lesson_place: String) -> LinearLayout {
    let lesson_type_place = LinearLayout::horizontal()
        .child(
            TextView::new(lesson_type)
                .style(ColorStyle::new(BaseColor::White, BaseColor::Red)),
        )
        .child(TextView::new(" "))
        .child(
            TextView::new(lesson_place)
                .style(ColorStyle::new(BaseColor::White, BaseColor::Blue)),
        );
    lesson_type_place
}

pub fn schedules_view() -> NamedView<Dialog> {
    let mut schedules_list = LinearLayout::vertical();
    let cfg = config::get_config().unwrap();
    let date = Utc.timestamp(cfg.selected_date, 0);
    let date_from_formatted = date.format("%d.%m.%Y").to_string();
    let date_to = date.checked_add_days(Days::new(7)).unwrap();
    let date_to = date_to.format("%d.%m.%Y").to_string();
    let rt = Runtime::new().unwrap();
    let schedules_result =
        rt.block_on(get_schedules(&date_from_formatted, &date_to, &cfg.group_id));

    match schedules_result {
        Ok(schedules) => {
            for day in schedules {
                let mut lesson_list_view = LinearLayout::vertical();

                for lesson in day.lessons {
                    let lesson_type =
                        schedules_additional_type_to_text(&lesson.additional.r#type.to_text());
                    let lesson_place = schedules_lesson_place_str(&lesson);
                    let lesson_times = lesson_times_view(&lesson);

                    // type & place / text / urls / etc
                    let mut lesson_text = StyledString::new();
                    lesson_text.append_plain(&lesson.text);
                    match lesson.additional.teacher_name {
                        Some(teacher) => {
                            lesson_text.append_plain(" ");
                            lesson_text.append_styled(
                                teacher,
                                Style {
                                    effects: Effects::empty(),
                                    color: ColorStyle::tertiary(),
                                },
                            );
                        }
                        None => (),
                    }

                    let lesson_body: LinearLayout = LinearLayout::vertical()
                        .child(lesson_type_place_view(lesson_type, lesson_place))
                        .child(TextView::new(lesson_text).full_width().max_width(40))
                        .child(schedules_links_view(lesson.urls).child(TextView::new(" ")));

                    // difficulty | body | times
                    let lesson_view = LinearLayout::horizontal()
                        .child(schedules_type_difficulty_view(&lesson.additional.r#type))
                        .child(lesson_body)
                        .child(lesson_times);

                    lesson_list_view.add_child(lesson_view);
                }

                schedules_list.add_child(TextView::new(format!(
                    "{}, {}.{}\n\n",
                    &day.week_day, &day.day, &day.month
                )));

                schedules_list.add_child(PaddedView::new(Margins::tb(0, 1), lesson_list_view));
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
                        s.pop_layer();
                        s.add_layer(schedules_view());
                    })),
            ))
            .child(schedules_list.scrollable()),
    )
    .title(t!("sections.schedules"))
    .button(t!("actions.close"), |s| {
        s.set_autohide_menu(false);
        s.pop_layer();
    })
    .with_name("schedules");
}