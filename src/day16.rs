use actix_web::{cookie, get, http, post, web, HttpRequest, HttpResponse};

const SECRET: &[u8] = b"penguin";

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_wrap)
        .service(get_unwrap)
        .service(post_decode);
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Claims {
    payload: serde_json::Value,
    exp: usize,
}

#[post("/16/wrap")]
async fn post_wrap(payload: web::Json<serde_json::Value>) -> HttpResponse {
    let payload = payload.into_inner();
    let exp = (chrono::Utc::now() + chrono::Days::new(1)).timestamp() as usize;

    let claims = Claims { payload, exp };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(SECRET),
    )
        .unwrap();

    let cookie = cookie::CookieBuilder::new("gift", token)
        .finish()
        .to_string();

    HttpResponse::Ok()
        .insert_header((http::header::SET_COOKIE, cookie))
        .finish()
}

#[get("/16/unwrap")]
async fn get_unwrap(req: HttpRequest) -> HttpResponse {
    let Some(token) = req.cookie("gift") else {
        return HttpResponse::BadRequest().finish();
    };

    let claims = jsonwebtoken::decode::<Claims>(
        token.value(),
        &jsonwebtoken::DecodingKey::from_secret(SECRET),
        &jsonwebtoken::Validation::default(),
    )
        .unwrap()
        .claims;

    HttpResponse::Ok().json(claims.payload)
}

#[post("/16/decode")]
async fn post_decode(jwt: String) -> HttpResponse {
    let mut validation = jsonwebtoken::Validation::default();
    validation.algorithms = vec![
        jsonwebtoken::Algorithm::RS256,
        jsonwebtoken::Algorithm::RS512,
    ];
    validation.set_required_spec_claims::<String>(&[]);

    let claims = match jsonwebtoken::decode::<serde_json::Value>(
        &jwt,
        &jsonwebtoken::DecodingKey::from_rsa_pem(include_bytes!("../day16_santa_public_key.pem"))
            .unwrap(),
        &validation,
    ) {
        Ok(token_data) => token_data.claims,
        Err(e) => match e.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                return HttpResponse::Unauthorized().finish()
            }
            _ => return HttpResponse::BadRequest().finish(),
        },
    };

    HttpResponse::Ok().json(claims)
}