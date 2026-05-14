mod config;
mod controllers;
mod forms;
mod models;
mod repositories;
mod routes;
mod services;
mod views;

use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};
use config::Config;
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Could not connect to PostgreSQL. Check DATABASE_URL in .env.");

    let state = AppState { db: pool };
    let session_key = Key::from(config.session_secret.as_bytes());
    let bind_address = config.bind_address();

    println!("RustToGold running at http://{}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                session_key.clone(),
            ))
            .service(Files::new("/static", "./static"))
            .configure(routes::configure)
            .default_service(web::route().to(controllers::not_found))
    })
    .bind(bind_address)?
    .run()
    .await
}
