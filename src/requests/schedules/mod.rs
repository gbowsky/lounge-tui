use crate::requests::errors::ErrorCode;
use chrono::{Duration, NaiveTime};
use regex::Regex;
use scraper::{Html, Selector};
use std::vec;
pub mod additional;

pub struct LessonItem {
    pub time_start: String,
    pub time_end: String,
    pub text: String,
    pub additional: additional::AdditionalLessonInfo,
    pub urls: Vec<LessonUrl>,
}

pub struct DayItem {
    pub day: String,
    pub month: String,
    pub week_day: String,
    pub lessons: Vec<LessonItem>,
}

fn remove_nbsp(string: &str) -> String {
    string.replace("&nbsp;", "")
}

pub struct LessonUrl {
    pub text: String,
    pub url: String,
}

fn detect_custom_time(text: &str) -> Option<(String, String, String)> {
    let custom_time_regex = Regex::new("[0-9]{2}[-:.][0-9]{2}").unwrap();
    let custom_time_full_regex =
        Regex::new(r"(?i),*\s+начало в [0-9]{2}[-:.][0-9]{2}?( час)?!*").unwrap();
    let separator_regex = Regex::new(r"[-:.]").unwrap();
    if custom_time_regex.is_match(&text) {
        let hours_minutes = custom_time_regex.find(&text).unwrap();
        let hours_minutes_split = separator_regex
            .split(hours_minutes.as_str())
            .collect::<Vec<_>>();
        let text = custom_time_full_regex.replace(&text, "");
        let (hours, minutes) = (
            hours_minutes_split.get(0).unwrap(),
            hours_minutes_split.get(1).unwrap(),
        );

        let new_time_start = NaiveTime::from_hms_opt(
            hours.to_string().parse::<u32>().unwrap(),
            minutes.to_string().parse::<u32>().unwrap(),
            0,
        );

        match new_time_start {
            Some(time_start) => {
                let duration = Duration::minutes(90);
                let time_end = time_start + duration;

                return Some((
                    time_start.format("%H:%M").to_string(),
                    time_end.format("%H:%M").to_string(),
                    text.to_string(),
                ));
            }
            None => {
                return None;
            }
        }
    }

    None
}

fn parse_urls(html: String) -> (Vec<LessonUrl>, String) {
    let anchor_selector = Selector::parse("a").unwrap();
    let mut html = Html::parse_fragment(&html);
    let mut result: Vec<LessonUrl> = Vec::new();

    for anchor in html.clone().select(&anchor_selector) {
        let href = anchor.attr("href");

        match href {
            Some(href) => {
                let text = anchor.text().collect::<Vec<_>>().join(" ");
                result.push(LessonUrl {
                    text: text.trim().to_string(),
                    url: href.to_string(),
                });
                let node_ids: Vec<_> = html.select(&anchor_selector).map(|x| x.id()).collect();
                for id in node_ids {
                    html.tree.get_mut(id).unwrap().detach();
                }
            }
            None => (),
        }
    }

    (
        result,
        remove_nbsp(&html.root_element().text().collect::<Vec<_>>().join(" ")),
    )
}

pub fn parse_schedules_table(html: String) -> Result<Vec<DayItem>, String> {
    let times_selector = Selector::parse("table > tbody > tr:nth-child(2) > td").unwrap();
    let rows_selector = Selector::parse("table > tbody > tr").unwrap();
    let html = Html::parse_fragment(&html);
    let rows = html.select(&rows_selector);
    let times = html.select(&times_selector);

    let rows_count = rows.clone().count();

    if rows_count == 0 {
        return Err(ErrorCode::FailedToParseSchedulesRows.get_description());
    }

    let first_row = rows.clone().nth(1).unwrap();
    let lesson_count = first_row.child_elements().count() - 1;
    let mut days: Vec<DayItem> = vec![];

    for rowcol in 2..rows_count {
        let day_month_el = rows.clone().nth(rowcol).unwrap().child_elements().nth(0);
        match day_month_el {
            Some(day_month_el) => {
                let date = day_month_el.text().collect::<Vec<_>>().join(" ");
                // 01.11, Mon
                let mut date_and_weekdays = date.trim().split(' ');
                let (date, week_day) = (
                    date_and_weekdays.nth(0).unwrap(),
                    date_and_weekdays.next().unwrap(),
                );
                // 01.11
                let mut day_month = date.split('.');
                let (day, month) = (day_month.nth(0).unwrap(), day_month.next().unwrap());

                days.insert(
                    rowcol - 2,
                    DayItem {
                        day: day.to_string(),
                        month: month.to_string(),
                        week_day: week_day.to_string(),
                        lessons: vec![],
                    },
                );

                for col in 0..=lesson_count {
                    let mut cols = rows.clone().nth(rowcol).unwrap().child_elements();
                    let text_el = cols.nth(col + 1);
                    let time_el = times.clone().nth(col);

                    let time_el_text = time_el.unwrap().text().collect::<Vec<_>>().join(" ");
                    let mut start_end = time_el_text.split("-");
                    let (start, end) = (
                        remove_nbsp(start_end.nth(0).unwrap()),
                        remove_nbsp(start_end.next().unwrap()),
                    );

                    let text_el_text = text_el.unwrap().text().collect::<Vec<_>>().join(" ");
                    if text_el_text.trim() != "" {
                        let (urls, text) = parse_urls(text_el.unwrap().inner_html());
                        let (additional, text) = additional::AdditionalLessonInfo::new(text);

                        match detect_custom_time(&text) {
                            Some((time_start, time_end, text)) => {
                                days[rowcol - 2].lessons.push(LessonItem {
                                    time_start,
                                    time_end,
                                    text: text.replace(", ", ""),
                                    additional,
                                    urls,
                                });
                            }
                            None => {
                                days[rowcol - 2].lessons.push(LessonItem {
                                    time_start: start.trim().to_string(),
                                    time_end: end.trim().to_string(),
                                    text: text.replace(", ", ""),
                                    additional,
                                    urls,
                                });
                            }
                        }
                    }
                }
            }
            None => return Err(ErrorCode::FailedToParseSchedulesRows.get_description()),
        }
    }

    Ok(days)
}
