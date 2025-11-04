use regex::Regex;

pub struct AdditionalLessonInfo {
    pub teacher_name: Option<String>,
    pub classroom: Option<String>,
    pub online: bool,
    pub groups: Option<String>,
    pub r#type: LessonType,
}

pub enum LessonType {
    Unknown,
    Practice,
    Lecture,
    Exam,
    SubjectReport,
    Consultation,
    SubjectReportWithGrade,
    CourseWorkDefend,
    Meeting,
}

fn parse_classroom(text: String) -> (Option<String>, String) {
    let classroom_regex =
        Regex::new(r"(?i), ?ауд\. ?\w{1,2}-?[0-9]{1,3}-?[0-9](-web|-к)?").unwrap();

    match classroom_regex.find(&text) {
        Some(classroom) => {
            let text = classroom_regex.replace(&text, "");
            let classroom = classroom.as_str().replace(", ", "");

            let classroom_splitted: Vec<&str> = classroom.split(' ').collect();

            match classroom_splitted.get(1) {
                Some(classroom) => (Some(classroom.to_string()), text.to_string()),
                None => (Some(classroom.replace("ауд.", "")), text.to_string()),
            }
        }
        None => (None, text),
    }
}

fn parse_teacher(text: String) -> (String, String) {
    let teacher_regex = Regex::new(r", .* .\..\.").unwrap();
    let teacher = teacher_regex.find(&text);
    match teacher {
        Some(teacher) => {
            let text = teacher_regex.replace(&text, "");
            (teacher.as_str().replace(", ", ""), text.to_string())
        }
        None => ("".to_string(), text),
    }
}

impl LessonType {
    pub fn parse_from_text(text: &str) -> (Self, String) {
        let lecture = Regex::new(r"(?i),? ?-?Лекц").unwrap();
        let practice = Regex::new(r"(?i),? ?-?Прак").unwrap();
        let consultation = Regex::new("(?i),? ?-?Конс").unwrap();
        let subject_report_with_grade = Regex::new(r"(?i),? ?-?ДифЗ").unwrap();
        let exam = Regex::new(r"(?i),? ?-?Экз").unwrap();
        let subject_report = Regex::new(r"(?i),? ?-?Зач").unwrap();
        let course_work_defend = Regex::new(r"(?i),? ?-?ЗКР").unwrap();
        let meeting = Regex::new(r"(?i),? ?-?Собр").unwrap();
        let text = text;

        if lecture.is_match(&text) {
            let text = &lecture.replace(&text, "").to_string();
            return (Self::Lecture, text.to_string());
        }

        if practice.is_match(&text) {
            let text = &practice.replace(&text, "").to_string();
            return (Self::Practice, text.to_string());
        }

        if consultation.is_match(&text) {
            let text = &consultation.replace(&text, "").to_string();
            return (Self::Consultation, text.to_string());
        }

        if subject_report_with_grade.is_match(&text) {
            let text = &subject_report_with_grade.replace(&text, "").to_string();
            return (Self::SubjectReportWithGrade, text.to_string());
        }

        if exam.is_match(&text) {
            let text = &exam.replace(&text, "").to_string();
            return (Self::Exam, text.to_string());
        }

        if subject_report.is_match(&text) {
            let text = &subject_report.replace(&text, "").to_string();
            return (Self::SubjectReport, text.to_string());
        }

        if course_work_defend.is_match(&text) {
            let text = &course_work_defend.replace(&text, "").to_string();
            return (Self::CourseWorkDefend, text.to_string());
        }

        if meeting.is_match(&text) {
            let text = &meeting.replace(&text, "").to_string();
            return (Self::Meeting, text.to_string());
        }

        (Self::Unknown, text.to_string())
    }

    pub fn to_text(self) -> String {
        match self {
            Self::Consultation => "consultation".to_string(),
            Self::SubjectReport => "subject_report".to_string(),
            Self::SubjectReportWithGrade => "subject_report_with_grade".to_string(),
            Self::Lecture => "lecture".to_string(),
            Self::Practice => "practice".to_string(),
            Self::Exam => "exam".to_string(),
            Self::CourseWorkDefend => "course_work_defend".to_string(),
            Self::Unknown => "unknown".to_string(),
            Self::Meeting => "meeting".to_string(),
        }
    }
}

impl AdditionalLessonInfo {
    pub fn new(text: String) -> (Self, String) {
        let mut result = Self {
            teacher_name: None,
            classroom: None,
            online: false,
            groups: None,
            r#type: LessonType::Unknown,
        };

        let mut text = text;

        if text.contains("ОНЛАЙН!") {
            text = text.replace("ОНЛАЙН!", "");
            result.online = true;
        }

        if text.contains("Вход на собрание") {
            text = text.replace("Вход на собрание", "");
            result.online = true;
        }

        if text.contains("Вход на занятие") {
            text = text.replace("Вход на занятие", "");
            result.online = true;
        }

        let (r#type, text) = LessonType::parse_from_text(&text);
        result.r#type = r#type;

        let (teacher, text) = parse_teacher(text);
        result.teacher_name = Some(teacher);

        let (classroom, text) = parse_classroom(text);
        result.classroom = classroom;

        (result, text.trim().to_string())
    }
}
