
// https://stackoverflow.com/questions/56714619/including-a-file-from-another-that-is-not-main-rs-nor-lib-rs
mod time_provider;
mod models;
mod storage;
mod handlers;

use dotenv::dotenv;
// https://actix.rs/
// very fast framework: https://www.techempower.com/benchmarks/#section=data-r19
use actix_web::{web, App, HttpServer};

use crate::time_provider::{SystemTimeProvider, TimeProvider};
use crate::models::{Config, Storage, Service};
use crate::storage::{invalid, postgres};
use crate::handlers::{not_found, add_board, list_boards, get_board, delete_board};


fn build_service (time_provider: Box<dyn TimeProvider>) -> Service {
    let config = Config::from_env();
    println!("config {:?}", config);

    // https://stackoverflow.com/questions/25383488/how-to-match-a-string-against-string-literals-in-rust
    let storage: Box<dyn Storage> = match config.provider.as_str() {
        "postgres" => match postgres::PostgresStorage::from_env(time_provider) {
            Err(why) => Box::new(invalid::InvalidStorage { error: format!("Invalid postgres storage provider! {}", why) }),
            Ok(storage) => Box::new(storage),
        },
        _ => Box::new(invalid::InvalidStorage { error: format!("Invalid or no storage provider given! '{}'", config.provider) })
    };

    println!("created storage: {}", storage.name());

    Service {
        time_provider: time_provider,
        config: config,
        storage: storage,
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // https://stackoverflow.com/questions/28219519/are-polymorphic-variables-allowed
    let time_provider: Box<dyn TimeProvider> = Box::new(SystemTimeProvider {});

    HttpServer::new(|| {
        App::new()
            .data(build_service(time_provider))
            .service(
                web::scope("/api")
                    .route("boards", web::post().to(add_board))
                    .route("boards", web::get().to(list_boards))
                    .route("boards/{id}", web::get().to(get_board))
                    .route("boards/{id}", web::delete().to(delete_board))
            )
            // https://github.com/actix/actix-website/blob/master/content/docs/url-dispatch.md
            .default_service(
                // https://docs.rs/actix-web/2.0.0/actix_web/struct.App.html#method.service
                web::route().to(not_found)
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
