use askama::Template;
use axum::{response::Html, extract::Query};
use crate::data::{Girl, load_girls_data};
use std::path::Path;
use std::collections::HashMap;
use fluent_bundle::{FluentBundle, FluentResource, FluentArgs, FluentValue};
use unic_langid::LanguageIdentifier;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    lang: &'a str,
    standby_girls: Vec<Girl>,
    girls: Vec<Girl>,
    greeting: String,
}



pub async fn home_handler(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    println!("route: HOME");

    let mut rng = thread_rng();
    let lang = params.get("lang").map(|s| s.as_str()).unwrap_or("en");
    let girls = load_girls_data(Path::new("data/girls.json"));

    let mut girls = girls;
    girls.shuffle(&mut rng);
    // Fluentのセットアップ
    let ftl_string = match lang {
        "ja" => include_str!("../../locales/ja.ftl"),
        "zh" => include_str!("../../locales/zh.ftl"),
        _ => include_str!("../../locales/en.ftl"),
    };
    let res = FluentResource::try_new(ftl_string.to_string()).expect("Failed to parse FTL.");
    let langid: LanguageIdentifier = lang.parse().expect("Failed to parse language identifier.");
    let mut bundle = FluentBundle::new(vec![langid]);
    bundle.add_resource(res).expect("Failed to add FTL resources to the bundle.");

    // 最初の女の子のstatusを取得
    let first_girl_status = girls.get(0).map(|girl| girl.status.clone()).unwrap_or_default();
    println!("First girl status: {}", first_girl_status);

    // FluentArgsを作成
    let mut args = FluentArgs::new();
    args.set("status", FluentValue::from(first_girl_status));


    // standbyのフィルタリング
    let standby_girls: Vec<Girl> = girls.iter().cloned().filter(|girl| girl.status == "STANDBY").collect();
    let mut standby_girls = standby_girls;
    standby_girls.shuffle(&mut rng);


    //==========================================
    // ========     locatiozation       ========
    //==========================================
    let greeting = bundle.get_message("greeting")
        .and_then(|msg| {
            let mut errors = vec![];
            Some(bundle.format_pattern(msg.value()?, None, &mut errors).to_string())
        })
        .unwrap_or_else(|| "Hello".to_string());

    // "STANDBY"のローカライズ
    let standby_status = bundle.get_message("standby_status")
        .and_then(|msg| {
            let mut errors = vec![];
            Some(bundle.format_pattern(msg.value()?, None, &mut errors).to_string())
        })
        .unwrap_or_else(|| "STANDBY".to_string());

    // standby_girlsのstatusを適用
    for girl in &mut standby_girls {
        if girl.status == "STANDBY" {
            girl.status = standby_status.clone();
        }
    }
    //==========================================


    let mut standby_girls = standby_girls;
    standby_girls.shuffle(&mut rng);

    let template = HomeTemplate { lang, standby_girls, girls, greeting };
    Html(template.render().unwrap())
}