use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware};
use env_logger::Env;
use std::net::Ipv4Addr;

use streameme_backend::handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("info"));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                // FIXME: This is not secure, it should be fixed this later.
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .wrap(middleware::Logger::default())
            .configure(handlers::config)
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run();

    log::info!("start HTTP server at http://localhost:8080");

    server.await
}
