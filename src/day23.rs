use actix_files as fs;
use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use tera::escape_html;
use ammonia::{clean, clean_text};
use html_escape::{encode_safe, encode_text};
use futures::{StreamExt, TryStreamExt};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use toml;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(fs::Files::new("/assets", "./assets"))
        .service(light_star)
        .service(present)
        .service(ornament)
        .service(lockfile);
}

#[get("/23/star")]
async fn light_star() -> impl Responder {
    // Static content - no dynamic user input, so no XSS risk here.
    HttpResponse::Ok().body(r#"<div id="star" class="lit"></div>"#)
}

#[get("/23/present/{color}")]
async fn present(req: HttpRequest) -> impl Responder {
    // Escape the color parameter to prevent XSS
    let color = escape_html(req.match_info().get("color").unwrap_or_default());

    // Cycle through colors safely
    let next_color = match &*color {
        "red" => "blue",
        "blue" => "purple",
        "purple" => "red",
        _ => {
            return HttpResponse::ImATeapot().finish(); // Invalid color returns 418
        }
    };

    // Escape the next color to prevent issues in the URL context
    let next_color = escape_html(next_color);

    // Construct the HTML response with escaped dynamic content
    let res = format!(
        r#"<div class="present {color}" hx-get="/23/present/{next_color}" hx-swap="outerHTML"><div class="ribbon"></div><div class="ribbon"></div><div class="ribbon"></div><div class="ribbon"></div></div>"#,
        color = color,
        next_color = next_color
    );

    // Return the response
    HttpResponse::Ok().body(res)
}

#[get("/23/ornament/{state}/{n}")]
async fn ornament(req: HttpRequest) -> impl Responder {
    // Get raw parameters
    let state = req.match_info().get("state").unwrap_or_default();
    let raw_n = req.match_info().get("n").unwrap_or_default();

    // URL decode the n parameter first
    let decoded_n = urlencoding::decode(raw_n)
        .unwrap_or(std::borrow::Cow::from(""))
        .into_owned();

    // Validate state before escaping
    let next_state = match state {
        "on" => "off",
        "off" => "on",
        _ => return HttpResponse::ImATeapot().finish(),
    };

    // Build class based on raw state
    let class = if state == "on" {
        "ornament on"
    } else {
        "ornament"
    };

    // HTML escape the decoded parameter
    let escaped_n = encode_safe(&decoded_n);

    // Format HTML with escaped parameters
    let html = format!(
        r#"<div class="{class}" id="ornament{escaped_n}" hx-trigger="load delay:2s once" hx-get="/23/ornament/{next_state}/{escaped_n}" hx-swap="outerHTML"></div>"#,
        class = class,
        escaped_n = escaped_n,
        next_state = next_state
    );

    HttpResponse::Ok().body(html)
}

#[derive(Deserialize)]
struct Package {
    checksum: String,
}

impl Package {
    fn cal(&self) -> Option<(String, u8, u8)> {
        if self.checksum.len() < 10 || !self.checksum.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let color = &self.checksum[..6];
        let top = u8::from_str_radix(&self.checksum[6..8], 16).ok()?;
        let left = u8::from_str_radix(&self.checksum[8..10], 16).ok()?;
        Some((format!("#{}", color), top, left))
    }
}

#[post("/23/lockfile")]
async fn lockfile(mut payload: Multipart) -> impl Responder {
    let mut htmls = Vec::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        if field.name() == Some("lockfile") {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(chunk) => data.extend_from_slice(&chunk),
                    Err(_) => return HttpResponse::BadRequest().finish(),
                }
            }

            let data = match String::from_utf8(data) {
                Ok(data) => data,
                Err(_) => return HttpResponse::BadRequest().finish(),
            };

            let payload: toml::Value = match toml::from_str(&data) {
                Ok(payload) => payload,
                Err(_) => return HttpResponse::BadRequest().finish(),
            };

            let packages = match payload.get("package").and_then(|p| p.as_array()) {
                Some(packages) => packages,
                None => return HttpResponse::BadRequest().finish(),
            };

            for package in packages {
                if let Some(checksum) = package.get("checksum").and_then(|c| c.as_str()) {
                    let pkg = Package { checksum: checksum.to_string() };
                    match pkg.cal() {
                        Some(d) => htmls.push(d),
                        None => return HttpResponse::UnprocessableEntity().finish(),
                    }
                }
            }
        }
    }

    if htmls.is_empty() {
        return HttpResponse::BadRequest().finish();
    }

    let html = htmls
        .into_iter()
        .map(|(color, top, left)| {
            format!(r#"<div style="background-color:{color};top:{top}px;left:{left}px;"></div>"#)
        })
        .collect::<String>();

    HttpResponse::Ok().body(html)
}