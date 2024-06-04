mod data;
mod handlers;

use axum::{
    routing::{get, post, delete},
    Router,
    routing::get_service,
    http::{Request, StatusCode},
};
use tower_http::services::ServeDir;
use handlers::{
    admin::{admin_handler, admin_girls_edit_handler, admin_girls_edit_post_handler, admin_girls_handler, admin_girls_new_handler, admin_girls_new_post_handler, admin_girls_delete_handler, update_girl_status},
    home::home_handler,
    girls::{girls_handler, girl_detail_handler},
    system::system_handler,
};

#[allow(warnings)]
#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(home_handler))
        .route("/girls", get(girls_handler))
        .route("/girl/:id", get(girl_detail_handler))
        .route("/system", get(system_handler))
        .route("/admin", get(admin_handler))
        .route("/admin/girls", get(admin_girls_handler))
        .route("/admin/girls/new", get(admin_girls_new_handler).post(admin_girls_new_post_handler))
        .route("/admin/girls/edit/:id", get(admin_girls_edit_handler).post(admin_girls_edit_post_handler))
        .route("/admin/girls/status/:id", post(update_girl_status))
        .route("/admin/girls/delete/:id", delete(admin_girls_delete_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

}
