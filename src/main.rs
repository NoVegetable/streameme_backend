use actix_cors::Cors;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::{App, HttpServer, middleware};
use clap::{Arg, Command, value_parser};
use env_logger::Env;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use streameme_backend::handlers;
use tempfile::TempDir;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = Command::new("streameme_backend")
        .arg(
            Arg::new("port")
                .help("The port to listen on")
                .short('p')
                .long("port")
                .value_parser(value_parser!(u16))
                .default_value("9090"),
        )
        .get_matches();

    let port = *matches.get_one::<u16>("port").unwrap();

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
    .bind((Ipv4Addr::UNSPECIFIED, port))?
    .run();

    log::info!("start HTTP server at http://localhost:{}", port);

    server.await
}
