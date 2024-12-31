use actix_web::{http, post, web, HttpRequest, HttpResponse};
use serde_json;
use serde_yml;
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_manifest);
}

#[derive(Default, serde::Deserialize)]
struct Metadata {
    #[serde(default)]
    orders: Vec<Order>,
}

#[serde_with::serde_as]
#[derive(serde::Deserialize)]
struct Order {
    item: String,
    #[serde_as(deserialize_as = "serde_with::DefaultOnError")]
    #[serde(default)]
    quantity: Option<usize>,
}

#[post("/5/manifest")]
async fn post_manifest(req: HttpRequest, text: String) -> HttpResponse {
    let content_type = req.headers().get(http::header::CONTENT_TYPE);

    let Some(package) = (match content_type.map(|c| c.to_str()) {
        Some(Ok("application/toml")) => {
            toml::from_str::<cargo_manifest::Manifest<Metadata>>(&text).ok()
        }
        Some(Ok("application/json")) => {
            serde_json::from_str::<cargo_manifest::Manifest<Metadata>>(&text).ok()
        }
        Some(Ok("application/yaml")) => {
            serde_yml::from_str::<cargo_manifest::Manifest<Metadata>>(&text).ok()
        }
        _ => return HttpResponse::UnsupportedMediaType().finish(),
    })
        .and_then(|m| m.package) else {
        return HttpResponse::BadRequest().body("Invalid manifest");
    };

    if !package
        .keywords
        .and_then(|k| k.as_local())
        .map(|k| k.contains(&"Christmas 2024".to_string()))
        .unwrap_or_default()
    {
        return HttpResponse::BadRequest().body("Magic keyword not provided");
    }

    let Some(orders) = package.metadata.map(|m| {
        m.orders
            .into_iter()
            .filter(|o| o.quantity.is_some())
            .map(|o| format!("{}: {}", o.item, o.quantity.unwrap()))
            .collect::<Vec<_>>()
    }) else {
        return HttpResponse::NoContent().finish();
    };

    if orders.is_empty() {
        return HttpResponse::NoContent().finish();
    }

    HttpResponse::Ok().body(orders.join("\n"))
}