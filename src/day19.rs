use std::{collections::HashMap, sync::LazyLock};

use actix_web::{delete, get, post, put, web, HttpResponse};
use rand::{distributions::Alphanumeric, Rng};
use shuttle_runtime::tokio::sync::Mutex;

static STATE: LazyLock<web::Data<State>> = LazyLock::new(Default::default);

#[derive(Default)]
struct State {
    tokens: Mutex<HashMap<String, i64>>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct Quote {
    id: uuid::Uuid,
    author: String,
    quote: String,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i32,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    let state = STATE.clone();

    cfg.service(post_reset)
        .service(get_cite)
        .service(delete_remove)
        .service(put_undo)
        .service(post_draft)
        .service(get_list)
        .app_data(state);
}

#[post("/19/reset")]
async fn post_reset(pool: web::Data<sqlx::PgPool>) -> HttpResponse {
    sqlx::query("DELETE FROM quotes")
        .execute(pool.as_ref())
        .await
        .unwrap();

    HttpResponse::Ok().finish()
}

#[get("/19/cite/{id}")]
async fn get_cite(path: web::Path<uuid::Uuid>, pool: web::Data<sqlx::PgPool>) -> HttpResponse {
    let uuid = path.into_inner();

    // Replace query_as! with query_as and bind the parameter manually
    let quote: Result<Quote, sqlx::Error> = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
        .bind(uuid)  // Bind the uuid parameter
        .fetch_one(pool.as_ref())  // Fetch a single result
        .await;

    match quote {
        Ok(quote) => HttpResponse::Ok().json(quote),  // Return 200 OK with the quote if found
        Err(_) => HttpResponse::NotFound().finish(),  // Return 404 if no quote found
    }
}


#[delete("/19/remove/{id}")]
async fn delete_remove(path: web::Path<uuid::Uuid>, pool: web::Data<sqlx::PgPool>) -> HttpResponse {
    let uuid = path.into_inner();

    let quote: Result<Quote, sqlx::Error> = sqlx::query_as::<_, Quote>("DELETE FROM quotes WHERE id = $1 RETURNING *")
        .bind(uuid)  // Bind the uuid parameter
        .fetch_one(pool.as_ref())  // Fetch the returned row
        .await;

    match quote {
        Ok(quote) => HttpResponse::Ok().json(quote),  // Return 200 OK with the deleted quote
        Err(_) => HttpResponse::NotFound().finish(),  // Return 404 if no quote found
    }
}

#[derive(serde::Deserialize)]
struct Info {
    author: String,
    quote: String,
}

#[put("/19/undo/{id}")]
async fn put_undo(
    path: web::Path<uuid::Uuid>,
    info: web::Json<Info>,
    pool: web::Data<sqlx::PgPool>,
) -> HttpResponse {
    let uuid = path.into_inner();
    let Info { author, quote } = info.0;

    let quote: Result<Quote, sqlx::Error> = sqlx::query_as::<_, Quote>(
        "UPDATE quotes SET author = $2, quote = $3, version = version + 1 WHERE id = $1 RETURNING *"
    )
        .bind(uuid)   // Bind the uuid parameter
        .bind(author) // Bind the author parameter
        .bind(quote)  // Bind the quote parameter
        .fetch_one(pool.as_ref())  // Fetch a single result
        .await;

    match quote {
        Ok(quote) => HttpResponse::Ok().json(quote),  // Return 200 OK with the updated quote
        Err(_) => HttpResponse::NotFound().finish(),  // Return 404 if no quote found
    }
}

#[post("/19/draft")]
async fn post_draft(info: web::Json<Info>, pool: web::Data<sqlx::PgPool>) -> HttpResponse {
    let uuid = uuid::Uuid::new_v4();
    let Info { author, quote } = info.0;

    let quote: Quote = sqlx::query_as::<_, Quote>(
        "INSERT INTO quotes(id, author, quote) VALUES($1, $2, $3) RETURNING *"
    )
        .bind(uuid)    // Bind the UUID
        .bind(author)  // Bind the author
        .bind(quote)   // Bind the quote
        .fetch_one(pool.as_ref())  // Fetch the result
        .await
        .unwrap();  // Handle error (you may want better error handling here)

    HttpResponse::Created().json(quote)  // Return 201 Created with the new quote
}

#[derive(serde::Deserialize)]
struct Token {
    token: Option<String>,
}

#[derive(serde::Serialize)]
struct List {
    quotes: Vec<Quote>,
    page: i64,
    next_token: Option<String>,
}

#[get("/19/list")]
async fn get_list(
    web::Query(Token { token }): web::Query<Token>,
    state: web::Data<State>,
    pool: web::Data<sqlx::PgPool>,
) -> HttpResponse {
    let page = if let Some(token) = token {
        let Some(page) = state.tokens.lock().await.remove(&token) else {
            return HttpResponse::BadRequest().finish();
        };

        page
    } else {
        0
    };

    let count = sqlx::query!("SELECT COUNT(*) FROM quotes")
        .fetch_one(pool.as_ref())
        .await
        .unwrap()
        .count
        .unwrap();

    let quotes = sqlx::query_as!(
        Quote,
        "SELECT * FROM quotes ORDER BY created_at LIMIT 3 OFFSET $1",
        page * 3,
    )
        .fetch_all(pool.as_ref())
        .await
        .unwrap();

    let page = page + 1;

    let next_token = if count > page * 3 {
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        state.tokens.lock().await.insert(token.clone(), page);

        Some(token)
    } else {
        None
    };

    let list = List {
        quotes,
        page,
        next_token,
    };

    HttpResponse::Ok().json(list)
}