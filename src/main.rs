use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use dotenv::dotenv;

mod api;
mod handlers;
mod middlewares;
mod models;

use api::routes::{book_event, test};
use handlers::mongo::MongoDB;
use middlewares::auth::CheckLoginFactory;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = match std::env::var("PORT") {
        Ok(port) => port.parse().unwrap_or_default(),
        Err(_) => 8080,
    };

    let env = std::env::var("ENV").unwrap_or("development".to_string());
    if env == "development" {
        dotenv().ok();
        env_logger::init();
    }

    let mongo = MongoDB::init().await;
    let mongo_data = Data::new(mongo);

    println!("Starting the Booking Machine server in ENV '{env}' on PORT {port}!");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(CheckLoginFactory)
            .app_data(mongo_data.clone())
            .service(book_event)
            .service(test)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
