use actix_cors::Cors;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::{App, HttpServer, middleware};
use env_logger::Env;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use streameme_backend::handlers;
use tempfile::TempDir;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls())?;
    builder.set_private_key_file("key.pem", SslFiletype::PEM)?;
    builder.set_certificate_chain_file("cert.pem")?;

    env_logger::init_from_env(Env::new().default_filter_or("info"));

    let tmp_dir = Arc::new(Mutex::new(TempDir::new_in(".")?));

    let tmp_dir_2 = tmp_dir.clone();
    let server = HttpServer::new(move || {
        let tmp_dir = tmp_dir_2.lock().unwrap();
        let path = tmp_dir.path();
        App::new()
            .wrap(
                // FIXME: This is not secure, it should be fixed this later.
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .wrap(middleware::Logger::default())
            .app_data(TempFileConfig::default().directory(path))
            .configure(handlers::config)
    })
    .bind_openssl((Ipv4Addr::UNSPECIFIED, 9090), builder)?
    .run();

    log::info!("start HTTP server at http://localhost:9090");

    server.await
}
