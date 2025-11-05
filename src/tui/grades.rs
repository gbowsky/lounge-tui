use cursive::theme::{BaseColor, ColorStyle, ColorType, PaletteStyle};
use cursive::view::Nameable;
use cursive::{
    align::Align,
    view::{Margins, Resizable, Scrollable},
    views::{Dialog, LinearLayout, NamedView, PaddedView, TextView},
};
use tokio::runtime::Runtime;

use crate::config;
use crate::requests::{
    self,
    grades::{GradeResult, GradeType},
};

use rust_i18n::t;

rust_i18n::i18n!();

fn grade_type_to_string(grade_type: &GradeType) -> String {
    return t!("grades_type.".to_owned() + grade_type.to_string()).to_string();
}

fn grade_grade_to_string(grade: &GradeResult) -> String {
    return t!("grades_grade.".to_owned() + grade.to_string()).to_string();
}

fn grade_grade_color(grade: &GradeResult) -> ColorStyle {
    let red_style = ColorStyle::new(
        ColorType::Color(BaseColor::Black.dark()),
        ColorType::Color(BaseColor::Red.light()),
    );

    let yellow_style = ColorStyle::new(
        ColorType::Color(BaseColor::Black.dark()),
        ColorType::Color(BaseColor::Yellow.light()),
    );

    let green_style = ColorStyle::new(
        ColorType::Color(BaseColor::Black.dark()),
        ColorType::Color(BaseColor::Green.light()),
    );

    match grade {
        GradeResult::Absence => red_style,
        GradeResult::Failed => red_style,
        GradeResult::Two => red_style,
        GradeResult::NotAdmitted => red_style,
        GradeResult::Unknown => yellow_style,
        GradeResult::Three => yellow_style,
        _ => green_style,
    }
}

pub fn grades_view() -> NamedView<Dialog> {
    let mut semester_list = LinearLayout::vertical();
    let cfg = config::get_config().unwrap();
    let (pin, last_name) = (cfg.pin, cfg.last_name);
    let rt = Runtime::new().unwrap();
    let grades_result = rt.block_on(requests::get_grades(&pin, &last_name));

    match grades_result {
        Ok(semesters) => {
            for (index, semester) in semesters.iter().enumerate() {
                let mut grade_list = LinearLayout::vertical();

                for grade in semester {
                    let grade_item = LinearLayout::horizontal()
                        .child(TextView::new("■ ".to_owned() + &grade.name).full_width().max_width(30))
                        .child(
                            LinearLayout::vertical()
                                .child(
                                    TextView::new(format!(
                                        "{: ^7}",
                                        &grade_grade_to_string(&grade.grade)
                                    ))
                                    .align(Align::top_right())
                                    .style(grade_grade_color(&grade.grade)),
                                )
                                .child(
                                    TextView::new(grade_type_to_string(&grade.r#type))
                                        .style(PaletteStyle::Tertiary)
                                        .align(Align::top_right())
                                        .min_width(7)
                                        .full_width()
                                        .max_width(19),
                                )
                                .child(TextView::new(" ")),
                        );

                    grade_list.add_child(grade_item);
                }
                semester_list.add_child(PaddedView::new(
                    Margins {
                        left: 0,
                        right: 0,
                        top: 1,
                        bottom: 1,
                    },
                    TextView::new(
                        "===> Семестр №".to_owned() + &(index + 1).to_string() + "<===\n",
                    )
                    .align(Align::center()),
                ));
                semester_list.add_child(grade_list);
            }
        }
        Err(err) => {
            semester_list.add_child(TextView::new(t!("errors.grades", e = err)));
        }
    }

    Dialog::around(PaddedView::new(
        Margins {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        },
        semester_list.scrollable(),
    ))
    .title(t!("sections.grades"))
    .dismiss_button(t!("actions.close"))
    .with_name("grades")
}
