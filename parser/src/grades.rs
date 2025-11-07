use scraper::{Html, Selector};
use std::vec;

pub enum GradeType {
    SubjectReportWithGrade,
    SubjectReport,
    Exam,
    OnlineCourseWork,
    OfflineCourseWork,
    GovExam,
    Unknown,
}

impl GradeType {
    pub fn to_string(&self) -> &str {
        match self {
            GradeType::GovExam => "gov_exam",
            GradeType::Exam => "exam",
            GradeType::OfflineCourseWork => "offline_course_work",
            GradeType::OnlineCourseWork => "online_course_work",
            GradeType::SubjectReport => "subject_report",
            GradeType::SubjectReportWithGrade => "subject_report_with_grade",
            GradeType::Unknown => "unknown",
        }
    }

    pub fn from_parsed(parsed: &str) -> GradeType {
        match parsed {
            "Государственный экзамен" => GradeType::GovExam,
            "Экзамен" => GradeType::Exam,
            "Курсовая работа (очно)" => GradeType::OfflineCourseWork,
            "Курсовая работа (заочно)" => GradeType::OnlineCourseWork,
            "Зачёт" => GradeType::SubjectReport,
            "Дифференцированный зачет" => GradeType::SubjectReportWithGrade,
            _ => GradeType::Unknown,
        }
    }
}

pub enum GradeResult {
    Failed,
    Passed,
    Absence,
    NotAdmitted,
    Two,
    Three,
    Four,
    Five,
    Unknown,
}

impl GradeResult {
    pub fn to_string(&self) -> &str {
        match self {
            GradeResult::Failed => "failed",
            GradeResult::Passed => "passed",
            GradeResult::Absence => "absence",
            GradeResult::NotAdmitted => "not_admitted",
            GradeResult::Two => "2",
            GradeResult::Three => "3",
            GradeResult::Four => "4",
            GradeResult::Five => "5",
            GradeResult::Unknown => "unknown",
        }
    }
    pub fn from_parsed(parsed: &str) -> GradeResult {
        match parsed {
            "н/я" => GradeResult::Absence,
            "зач." => GradeResult::Passed,
            "н/зач." => GradeResult::Failed,
            "5" => GradeResult::Five,
            "4" => GradeResult::Four,
            "3" => GradeResult::Three,
            "2" => GradeResult::Two,
            "н/доп." => GradeResult::NotAdmitted,
            _ => GradeResult::Unknown,
        }
    }
}

pub struct GradeItem {
    pub name: String,
    pub r#type: GradeType,
    pub grade: GradeResult,
}

pub fn parse_grade_table(html: String) -> [Vec<GradeItem>; 8] {
    let html = Html::parse_fragment(&html);
    let table_selector = Selector::parse("table").unwrap();
    let rows_selector = Selector::parse("tr").unwrap();
    let data_selector = Selector::parse("td").unwrap();
    let mut semesters: [Vec<GradeItem>; 8] = [
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
    ];

    // Таблица - один семестр
    for (index, table) in html.select(&table_selector).enumerate() {
        let mut result: Vec<GradeItem> = Vec::new();
        for (index, row) in table.select(&rows_selector).enumerate() {
            if index == 0 {
                // пропускаем шапку таблицы
                continue;
            }

            let mut data = row.select(&data_selector);
            let discipline_name = data.nth(0).unwrap();
            let grade_type = data.next().unwrap();
            let grade_result = data.next().unwrap();

            result.push(GradeItem {
                name: discipline_name
                    .inner_html()
                    .replace("&nbsp;", "")
                    .to_owned(),
                r#type: GradeType::from_parsed(
                    &grade_type.inner_html().replace("&nbsp;", "").to_owned(),
                ),
                grade: GradeResult::from_parsed(
                    &grade_result.child_elements().nth(0).unwrap().inner_html(),
                ),
            })
        }
        semesters[index] = result;
    }

    semesters
}
