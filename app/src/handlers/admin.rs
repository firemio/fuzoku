use askama::Template;
use axum::{
    extract::{Multipart, Path, Form, Query, Json},
    response::{Html,Redirect,IntoResponse},
    http::StatusCode,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path as StdPath;
use std::sync::atomic::{AtomicU32, Ordering};
use uuid::Uuid;
use image::GenericImageView;
use image::{codecs::webp, ColorType, DynamicImage, EncodableLayout, ImageEncoder};

use crate::data::{load_girls_data, save_girls_data, Girl};


const DATA_FILE: &str = "data/girls.json";
const UPLOAD_DIR: &str = "static/uploads";
//static NEXT_ID: AtomicU32 = AtomicU32::new(1024);

#[derive(Template)]
#[template(path = "admin.html")]
struct AdminTemplate<'a> {
    lang: &'a str,
}

#[derive(Template)]
#[template(path = "admin_girls.html")]
struct AdminGirlsTemplate<'a> {
    lang: &'a str,
    girls: Vec<Girl>,
}

#[derive(Template)]
#[template(path = "admin_girls_edit.html")]
struct AdminGirlsEditTemplate<'a> {
    lang: &'a str,
    girl: Girl,
}

#[derive(Template)]
#[template(path = "admin_girls_new.html")]
struct AdminGirlsNewTemplate<'a> {
    lang: &'a str,
}

#[derive(Deserialize)]
pub struct GirlForm {
    name: String,
    profile: String,
    schedule: HashMap<String, String>,
}

pub async fn admin_handler() -> Html<String> {
    let template = AdminTemplate { lang: "en" };
    Html(template.render().unwrap())
}

pub async fn admin_girls_handler() -> Html<String> {
    let girls = load_girls_data(StdPath::new(DATA_FILE));
    let template = AdminGirlsTemplate { lang: "en", girls };
    Html(template.render().unwrap())
}

pub async fn admin_girls_edit_handler(Path(id): Path<String>) -> Html<String> {
    let girls = load_girls_data(StdPath::new(DATA_FILE));
    let girl = girls.into_iter().find(|g| g.id == id.to_string()).unwrap();
    println!("{:?}", girl.schedule.get("sun_end"));

    let template = AdminGirlsEditTemplate { lang: "en", girl };
    Html(template.render().unwrap())
}

pub async fn admin_girls_edit_post_handler(Path(id): Path<String>, mut multipart: Multipart) -> Result<impl IntoResponse, StatusCode> {
    let mut girl_form: Option<GirlForm> = None;
    let mut thumbnail_path: Option<String> = None;
    let mut schedule: HashMap<String, String> = HashMap::new();

    //==== multi post proc
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        eprintln!("Error reading field: {:?}", e);
        StatusCode::BAD_REQUEST
    })? {
        let name = field.name().ok_or_else(|| {
            eprintln!("Field name is missing");
            StatusCode::BAD_REQUEST
        })?.to_string();
        let data = field.bytes().await.map_err(|e| {
            eprintln!("Error reading field bytes: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;

        if name.starts_with("schedule_") {
            let day = name.strip_prefix("schedule_").ok_or_else(|| {
                eprintln!("Error stripping prefix from field name");
                StatusCode::BAD_REQUEST
            })?.to_string();
            let value = std::str::from_utf8(&data).map_err(|e| {
                eprintln!("Error converting field data to string: {:?}", e);
                StatusCode::BAD_REQUEST
            })?.to_string();
            //println!("======== {}{}",day,value);
            schedule.insert(day, value);
        } else if name == "name" || name == "profile" {
            let value = std::str::from_utf8(&data).map_err(|e| {
                eprintln!("Error converting field data to string: {:?}", e);
                StatusCode::BAD_REQUEST
            })?.to_string();
            match name.as_str() {
                "name" => {
                    if let Some(ref mut form) = girl_form {
                        form.name = value;
                    } else {
                        girl_form = Some(GirlForm {
                            name: value,
                            profile: String::new(),
                            schedule: HashMap::new(),
                        });
                    }
                }
                "profile" => {
                    if let Some(ref mut form) = girl_form {
                        form.profile = value;
                    } else {
                        girl_form = Some(GirlForm {
                            name: String::new(),
                            profile: value,
                            schedule: HashMap::new(),
                        });
                    }
                }
                _ => {}
            }
        } else if name == "thumbnail" {
            if !data.is_empty() {
                let uuid = Uuid::new_v4();
                let file_extension = match infer::get(&data) {
                    Some(info) => info.extension(),
                    None => return Err(StatusCode::BAD_REQUEST),
                };

                let file_name = format!("{}.{}", uuid, file_extension);
                let file_path = format!("{}/{}", UPLOAD_DIR, file_name);
                let file_name2 = format!("{}.webp", uuid);
                let file_path2 = format!("{}/{}", UPLOAD_DIR, file_name2);

                let mut file = File::create(&file_path).map_err(|e| {
                    eprintln!("Error creating file: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                file.write_all(&data).map_err(|e| {
                    eprintln!("Error writing to file: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                let img = image::open(&file_path).map_err(|e| {
                    eprintln!("Error opening image: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                if let Err(e) = std::fs::remove_file(&file_path) {
                    eprintln!("Error deleting file: {:?}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                let (width, height) = img.dimensions();
                let aspect_ratio = height as f64 / width as f64;
                let new_height = (400.0 * aspect_ratio) as u32;
                let resized_img = img.resize_exact(400, new_height, image::imageops::FilterType::Lanczos3);
                // ここで品質を設定して保存
                let file = File::create(&file_path2).map_err(|e| {
                    eprintln!("Error creating file: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                let mut encoder = image::codecs::webp::WebPEncoder::new(file);
                encoder.encode(&resized_img.to_rgba8(), resized_img.width(), resized_img.height(), image::ColorType::Rgba8).map_err(|e| {
                    eprintln!("Error saving resized image: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                thumbnail_path = Some(file_name2);
            }
        }
    }

    let form = girl_form.ok_or_else(|| {
        eprintln!("Form data is missing");
        StatusCode::BAD_REQUEST
    })?;

    let mut girls = load_girls_data(StdPath::new(DATA_FILE));
    if let Some(girl) = girls.iter_mut().find(|g| g.id == id.to_string()) {
        girl.name = form.name;
        girl.profile = form.profile;

        let thumbnail = thumbnail_path.unwrap_or_else(|| girl.thumbnail.clone());

        if girl.thumbnail != thumbnail {
            let old_thumbnail_path = format!("{}/{}", UPLOAD_DIR, girl.thumbnail);
            if let Err(e) = std::fs::remove_file(&old_thumbnail_path) {
                eprintln!("Error deleting old thumbnail: {:?}", e);
            }
        }
        girl.thumbnail = thumbnail;
    
    }


        println!("=== save json ===");
        save_girls_data(StdPath::new(DATA_FILE), &girls);

//    Ok(Html(format!("Girl with ID {} has been updated successfully.", id)))
    Ok(Redirect::to("/admin/girls"))

}



pub async fn admin_girls_new_handler() -> Html<String> {
    let template = AdminGirlsNewTemplate { lang: "en" };
    Html(template.render().unwrap())
}


//pub async fn admin_girls_new_post_handler(Form(girl_form): Form<GirlForm>) -> Result<Html<String>, StatusCode> {
pub async fn admin_girls_new_post_handler(mut multipart: Multipart) -> Result<impl IntoResponse, StatusCode> {
    println!("### STEP1");
    let mut girl_form: Option<GirlForm> = None;
    let mut thumbnail_path: Option<String> = None;
    let mut schedule: HashMap<String, String> = HashMap::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        println!("### step1-1 ###");
        let name = field.name().ok_or_else(|| {
            eprintln!("Field name is missing");
            StatusCode::BAD_REQUEST
        })?.to_string();
        let data = field.bytes().await.map_err(|e| {
            eprintln!("Error reading field bytes: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;

        println!("### step1-2 ###");
        if name.starts_with("schedule_") {
            let day = name.strip_prefix("schedule_").ok_or_else(|| {
                eprintln!("Error stripping prefix from field name");
                StatusCode::BAD_REQUEST
            })?.to_string();
            let value = std::str::from_utf8(&data).map_err(|e| {
                eprintln!("Error converting field data to string: {:?}", e);
                StatusCode::BAD_REQUEST
            })?.to_string();

            schedule.insert(day, value);
        } else if name == "name" || name == "profile" {
            let value = std::str::from_utf8(&data).map_err(|e| {
                eprintln!("Error converting field data to string: {:?}", e);
                StatusCode::BAD_REQUEST
            })?.to_string();
            match name.as_str() {
                "name" => {
                    if let Some(ref mut form) = girl_form {
                        form.name = value;
                    } else {
                        girl_form = Some(GirlForm {
                            name: value.clone(),
                            profile: String::new(),
                            schedule: HashMap::new(),
                        });
                    }
                }
                "profile" => {
                    if let Some(ref mut form) = girl_form {
                        form.profile = value;
                    } else {
                        girl_form = Some(GirlForm {
                            name: String::new(),
                            profile: String::new(),
                            schedule: HashMap::new(),
                        });
                    }
                }
                _ => {}
            }
        } else if name == "thumbnail" {
            if data.is_empty() {
            }else{
                println!("UP01");
                let uuid = Uuid::new_v4();
                let file_extension = match infer::get(&data) {
                    Some(info) => info.extension(),
                    None => return Err(StatusCode::BAD_REQUEST),
                };

                let file_name = format!("{}.{}", uuid, file_extension);
                let file_path = format!("{}/{}", UPLOAD_DIR, file_name);
                let file_name2 = format!("{}.webp", uuid);
                let file_path2 = format!("{}/{}", UPLOAD_DIR, file_name2);

                let mut file = File::create(&file_path).map_err(|e| {
                    eprintln!("Error creating file: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                file.write_all(&data).map_err(|e| {
                    eprintln!("Error writing to file: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                let img = image::open(&file_path).map_err(|e| {
                    eprintln!("Error opening image: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                if let Err(e) = std::fs::remove_file(&file_path) {
                    eprintln!("Error deleting file: {:?}", e);
                }
                let (width, height) = img.dimensions();
                let aspect_ratio = height as f64 / width as f64;
                let new_height = (400.0 * aspect_ratio) as u32;
                let resized_img = img.resize_exact(400, new_height, image::imageops::FilterType::Lanczos3);
                // ここで品質を設定して保存
                let file = File::create(&file_path2).map_err(|e| {
                    eprintln!("Error creating file: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                let mut encoder = image::codecs::webp::WebPEncoder::new(file);
                encoder.encode(&resized_img.to_rgba8(), resized_img.width(), resized_img.height(), image::ColorType::Rgba8).map_err(|e| {
                    eprintln!("Error saving resized image: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

                thumbnail_path = Some(file_name2);

            }
        }
        println!("### step1-3 ###");

    }

    println!("### STEP2");

    let form = girl_form.ok_or_else(|| {
        eprintln!("Form data is missing");
        StatusCode::BAD_REQUEST
    })?;
    let thumbnail = thumbnail_path;

    let mut girls = load_girls_data(StdPath::new(DATA_FILE));
    let id = Uuid::new_v4().to_string();

    // スケジュールのデフォルト値を設定
    let default_schedule = vec![
        ("sun_start", ""), ("sun_end", ""),
        ("mon_start", ""), ("mon_end", ""),
        ("tue_start", ""), ("tue_end", ""),
        ("wed_start", ""), ("wed_end", ""),
        ("thu_start", ""), ("thu_end", ""),
        ("fri_start", ""), ("fri_end", ""),
        ("sat_start", ""), ("sat_end", ""),
    ];
    
    let new_girl = Girl {
        id,
        name: form.name,
        profile: form.profile,
        thumbnail: thumbnail.unwrap_or_default(),
        schedule: default_schedule.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        status: "STANDBY".to_string(),
        photos: vec![],
    };

    girls.push(new_girl);
    save_girls_data(StdPath::new(DATA_FILE), &girls);

    // /admin/girlsにリダイレクト
    Ok(Redirect::to("/admin/girls"))
}

pub async fn admin_girls_delete_handler(Path(id): Path<String>) -> StatusCode {
    use std::fs;

    let girls = load_girls_data(StdPath::new(DATA_FILE));
    if let Some(girl) = girls.iter().find(|g| g.id == id) {
        let thumbnail_path = format!("{}/{}", UPLOAD_DIR, girl.thumbnail);
        if let Err(e) = fs::remove_file(&thumbnail_path) {
            eprintln!("サムネイルファイルの削除中にエラーが発生しました: {:?}", e);
        }
    }
    let mut girls = load_girls_data(StdPath::new(DATA_FILE));
    girls.retain(|g| g.id != id);
    save_girls_data(StdPath::new(DATA_FILE), &girls);
    StatusCode::OK
}




#[derive(Deserialize)]
pub struct StatusUpdate {
    status: String,
}

pub async fn update_girl_status(Path(id): Path<String>, Json(payload): Json<StatusUpdate>) -> Result<impl IntoResponse, StatusCode> {
    let mut girls = load_girls_data(StdPath::new(DATA_FILE));
    if let Some(girl) = girls.iter_mut().find(|g| g.id == id) {
        girl.status = payload.status;
        save_girls_data(StdPath::new(DATA_FILE), &girls);
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
