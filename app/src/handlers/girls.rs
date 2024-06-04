use askama::Template;
use axum::{
    extract::Path,
    response::Html,
};
use crate::data::{Girl, load_girls_data};
use std::path::Path as StdPath;

#[derive(Template)]
#[template(path = "girls.html")]
struct GirlsTemplate<'a> {
    lang: &'a str,
    girls: Vec<Girl>,
}

#[derive(Template)]
#[template(path = "girl_detail.html")]
struct GirlDetailTemplate<'a> {
    lang: &'a str,
    girl: Girl,
}

pub async fn girls_handler() -> Html<String> {
    let girls = load_girls_data(StdPath::new("data/girls.json"));
    let template = GirlsTemplate { lang: "en", girls };
    Html(template.render().unwrap())
}

pub async fn girl_detail_handler(Path(id): Path<String>) -> Html<String> {
    let girls = load_girls_data(StdPath::new("data/girls.json"));
    let girl = girls.iter().find(|g| g.id == id).cloned().expect("Girl not found");
    let template = GirlDetailTemplate { lang: "en", girl };
    Html(template.render().unwrap())
}
