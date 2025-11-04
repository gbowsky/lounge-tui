use cursive::view::Nameable;
use cursive::{
    align::Align,
    view::{Margins, Resizable, Scrollable},
    views::{Dialog, LinearLayout, NamedView, PaddedView, TextView},
};
use tokio::runtime::Runtime;

use crate::{
    LoungeConfig,
    requests::{
        self,
        grades::{GradeResult, GradeType},
    },
};

use rust_i18n::t;

rust_i18n::i18n!();


fn grade_type_to_string(grade_type: &GradeType) -> String {
    return t!("grades_type.".to_owned() + grade_type.to_string()).to_string();
}

fn grade_grade_to_string(grade: &GradeResult) -> String {
    return t!("grades_grade.".to_owned() + grade.to_string()).to_string();
}

pub fn grades_view() -> NamedView<Dialog> {
    let mut semester_list = LinearLayout::vertical();
    let cfg: LoungeConfig = confy::load("lounge-tui", None).unwrap();
    let (pin, last_name) = (cfg.pin, cfg.last_name);
    let rt = Runtime::new().unwrap();
    let grades_result = rt.block_on(requests::get_grades(&pin, &last_name));

    match grades_result {
        Ok(semesters) => {
            for (index, semester) in semesters.iter().enumerate() {
                let mut grade_list = LinearLayout::vertical();

                for grade in semester {
                    let grade_item = LinearLayout::horizontal()
                        .child(TextView::new("■ ".to_owned() + &grade.name).fixed_width(35))
                        .child(
                            LinearLayout::vertical()
                                .child(
                                    TextView::new(grade_type_to_string(&grade.r#type))
                                        .align(Align::center_right())
                                        .fixed_width(11),
                                )
                                .child(
                                    TextView::new(grade_grade_to_string(&grade.grade))
                                        .align(Align::center_right())
                                        .fixed_width(11),
                                ),
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
