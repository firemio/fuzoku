use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "system.html")]
struct SystemTemplate<'a> {
    lang: &'a str,
}

pub async fn system_handler() -> Html<String> {
    let template = SystemTemplate { lang: "en" };
    Html(template.render().unwrap())
}
