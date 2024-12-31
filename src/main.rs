mod day0;
mod day2;
mod day5;
mod day9;
mod day12;
mod day16;
mod day19;
mod day23;

use actix_web::{error, web, HttpResponse};
use shuttle_actix_web::ShuttleActixWeb;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(local_uri = "postgresql://postgres@localhost:5432")] pool: sqlx::PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut web::ServiceConfig) + Send + Clone + 'static> {
    sqlx::migrate!().run(&pool).await.unwrap();

    let config = move |cfg: &mut web::ServiceConfig| {
        cfg.configure(day0::configure)
            .configure(day2::configure)
            .configure(day5::configure)
            .configure(day9::configure)
            .configure(day12::configure)
            .configure(day16::configure)
            .configure(day19::configure)
            .configure(day23::configure)
            .app_data(web::Data::new(pool))
            .app_data(web::PathConfig::default().error_handler(|err, _| {
                error::InternalError::from_response(err, HttpResponse::BadRequest().into()).into()
            }));
    };

    Ok(config.into())
}