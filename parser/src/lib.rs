pub mod errors;
pub mod grades;
pub mod schedules;
pub mod lists;

pub async fn get_schedules(
    date_from: &str,
    date_to: &str,
    group_id: &str,
) -> Result<Vec<schedules::DayItem>, String> {
    let params = [
        ("exam", "0"),
        ("formo", "0"),
        ("allp", "0"),
        ("hour", "0"),
        ("datafrom", &date_from),
        ("dataend", &date_to),
        ("rtype", "1"),
        ("group", &group_id),
        ("tuttabl", "0"),
    ];
    let client = reqwest::Client::new();
    let request = client
        .post("http://inet.ibi.spb.ru/raspisan/rasp.php")
        .form(&params)
        .send()
        .await;

    match request {
        Ok(request) => {
            let text = request.text().await;
            match text {
                Ok(html) => {
                    if html.contains(
                        "Информации для отображения отчета не обнаружено! Измените период.",
                    ) {
                        return Ok(vec![]);
                    }

                    return schedules::parse_schedules_table(html);
                }
                Err(err) => {
                    return Err(errors::ErrorCode::IbiBadResponse.get_description());
                }
            }
        }
        Err(e) => {
            return Err(errors::ErrorCode::FailedToRetrieve.get_description());
        }
    }
}

pub async fn get_grades(pin: &str, last_name: &str) -> Result<[Vec<grades::GradeItem>; 8], String> {
    let params = [("rtype", "6"), ("fio1", &last_name), ("pin1", &pin)];
    let client = reqwest::Client::new();
    let request = client
        .post("http://inet.ibi.spb.ru/raspisan/rasp.php")
        .form(&params)
        .send()
        .await;

    match request {
        Ok(request) => {
            let text = request.text().await;
            match text {
                Ok(html) => {
                    if html.contains("Введенная фамилия не соответствует пин коду!")
                    {
                        return Err(errors::ErrorCode::DataMismatchError.get_description());
                    }

                    Ok(grades::parse_grade_table(html))
                }
                Err(err) => {
                    return Err(errors::ErrorCode::IbiBadResponse.get_description());
                }
            }
        }
        Err(e) => {
            return Err(errors::ErrorCode::FailedToRetrieve.get_description());
        }
    }
}

pub async fn get_teachers() -> Result<Vec<lists::BasicItem>, String> {
    let request = reqwest::get("http://inet.ibi.spb.ru/raspisan/menu.php?tmenu=2&cod=").await;

    match request {
        Ok(request) => {
            let text = request.text().await;
            match text {
                Ok(html) => {
                    if html.contains("Соединение не установлено") {
                        return Err(errors::ErrorCode::IbiServersDown.get_description());
                    }

                    Ok(lists::parse_basic_list("teacher".to_owned(), html))
                }
                Err(err) => {
                    return Err(errors::ErrorCode::IbiBadResponse.get_description());
                }
            }
        }
        Err(e) => {
            return Err(errors::ErrorCode::FailedToRetrieve.get_description());
        }
    }
}

pub async fn get_groups(level: &str) -> Result<Vec<lists::BasicItem>, String> {
    let request =
        reqwest::get("http://inet.ibi.spb.ru/raspisan/menu.php?tmenu=12&cod=".to_owned() + &level)
            .await;

    match request {
        Ok(request) => {
            let text = request.text().await;
            match text {
                Ok(html) => {
                    if html.contains("Соединение не установлено") {
                        return Err(errors::ErrorCode::IbiServersDown.get_description());
                    }

                    Ok(lists::parse_basic_list("group".to_owned(), html))
                }
                Err(err) => {
                    return Err(errors::ErrorCode::IbiBadResponse.get_description());
                }
            }
        }
        Err(e) => {
            return Err(errors::ErrorCode::FailedToRetrieve.get_description());
        }
    }
}

pub async fn get_levels() -> Result<Vec<lists::BasicItem>, String> {
    let request = reqwest::get("http://inet.ibi.spb.ru/raspisan/menu.php?tmenu=1").await;

    match request {
        Ok(request) => {
            let text = request.text().await;
            match text {
                Ok(html) => {
                    if html.contains("Соединение не установлено") {
                        return Err(errors::ErrorCode::IbiServersDown.get_description());
                    }

                    Ok(lists::parse_basic_list("ucstep".to_owned(), html))
                }
                Err(err) => {
                    return Err(errors::ErrorCode::IbiBadResponse.get_description());
                }
            }
        }
        Err(e) => {
            return Err(errors::ErrorCode::FailedToRetrieve.get_description());
        }
    }
}
