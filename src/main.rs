use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger, web::Data, App, HttpServer};
use dotenv::dotenv;

mod api;
mod handlers;
mod middlewares;
mod models;

use api::routes::{book_event, delete_event};
use handlers::mongo::MongoDB;
use middlewares::auth::CheckLoginFactory;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = match std::env::var("PORT") {
        Ok(port) => port.parse().unwrap_or_default(),
        Err(_) => 8081,
    };

    let env = std::env::var("ENV").unwrap_or("development".to_string());
    if env == "development" {
        dotenv().ok();
        env_logger::init();
    }

    let origin_url = std::env::var("ORIGIN").expect("Origin env variable is required.");

    let mongo = MongoDB::init().await;
    let mongo_data = Data::new(mongo);

    println!("Starting the Booking Machine server in ENV '{env}' on PORT {port}!");
    HttpServer::new(move || {
        App::new()
            .wrap(CheckLoginFactory)
            .wrap(
                Cors::default()
                    .allowed_origin(&origin_url)
                    .allowed_methods(vec!["GET", "POST", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT]),
            )
            .wrap(Logger::default())
            .app_data(mongo_data.clone())
            .service(book_event)
            .service(delete_event)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
