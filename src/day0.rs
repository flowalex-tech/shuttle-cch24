use actix_web::{get, web, Responder};
use actix_web::http::StatusCode;
use actix_web::web::Redirect;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(hello_world)
        .service(seek);
}

// Hello bird
#[get("/")]
async fn hello_world() -> &'static str {
    "Hello, bird!"
}

// 302 Found redirection
#[get("/-1/seek")]
async fn seek() -> impl Responder {
    Redirect::to("https://www.youtube.com/watch?v=9Gc4QTqslN4").using_status_code(StatusCode::FOUND)
}