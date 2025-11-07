use scraper::{Html, Selector};

pub struct BasicItem {
    pub id: String,
    pub label: String,
}

pub fn parse_basic_list(id: String, html: String) -> Vec<BasicItem> {
    let fragment = Html::parse_fragment(&html);
    let levels_selector = Selector::parse(&format!("#{} > option", id)).unwrap();

    let mut result: Vec<BasicItem> = Vec::new();

    for element in fragment.select(&levels_selector) {
        let id = element.value().attr("value");

        match id {
            Some(id_str) => result.push(BasicItem {
                id: id_str.to_string(),
                label: element.inner_html(),
            }),
            None => (),
        }
    }

    result
}
