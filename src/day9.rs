use std::time::Duration;

use actix_web::{http, post, web, HttpRequest, HttpResponse};
use shuttle_runtime::tokio::sync::Mutex;

lazy_static::lazy_static!(
    static ref STATE: web::Data<State> = web::Data::default();
);

struct State {
    rate_limiter: Mutex<leaky_bucket::RateLimiter>,
}

fn rate_limiter() -> leaky_bucket::RateLimiter {
    leaky_bucket::Builder::default()
        .initial(5)
        .interval(Duration::from_secs(1))
        .max(5)
        .build()
}

impl Default for State {
    fn default() -> Self {
        let rate_limiter = Mutex::new(rate_limiter());
        Self { rate_limiter }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_milk)
        .service(post_refill)
        .app_data(STATE.clone());
}

#[derive(serde::Deserialize)]
struct Body {
    liters: Option<f32>,
    gallons: Option<f32>,
    litres: Option<f32>,
    pints: Option<f32>,
}

#[post("/9/milk")]
async fn post_milk(
    req: HttpRequest,
    state: web::Data<State>,
    body: Option<web::Json<Body>>,
) -> HttpResponse {
    let content_type = req.headers().get(http::header::CONTENT_TYPE);

    let json = matches!(
        content_type.map(|c| c.to_str()),
        Some(Ok("application/json"))
    );

    if !state.rate_limiter.lock().await.try_acquire(1) {
        HttpResponse::TooManyRequests().body("No milk available\n")
    } else if !json {
        HttpResponse::Ok().body("Milk withdrawn\n")
    } else {
        match body.map(
            |web::Json(Body {
                           liters,
                           gallons,
                           litres,
                           pints,
                       })| (liters, gallons, litres, pints),
        ) {
            Some((Some(liters), None, None, None)) => {
                let gallons = liters / 3.785_411_8;

                let v = serde_json::json!({"gallons": gallons});

                HttpResponse::Ok().json(v)
            }
            Some((None, Some(gallons), None, None)) => {
                let liters = gallons * 3.785_411_8;

                let v = serde_json::json!({"liters": liters});

                HttpResponse::Ok().json(v)
            }
            Some((None, None, Some(litres), None)) => {
                let pints = litres * 1.759_754;

                let v = serde_json::json!({"pints": pints});

                HttpResponse::Ok().json(v)
            }
            Some((None, None, None, Some(pints))) => {
                let litres = pints / 1.759_754;

                let v = serde_json::json!({"litres": litres});

                HttpResponse::Ok().json(v)
            }
            _ => HttpResponse::BadRequest().finish(),
        }
    }
}

#[post("/9/refill")]
async fn post_refill(state: web::Data<State>) -> HttpResponse {
    *state.rate_limiter.lock().await = rate_limiter();

    HttpResponse::Ok().finish()
}