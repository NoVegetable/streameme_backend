use actix_cors::Cors;
use actix_multipart::form::MultipartFormConfig;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::{App, HttpServer, middleware, web};
use clap::{Arg, Command, value_parser};
use env_logger::Env;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use streameme_backend::analyzer::VideoAnalyzer;
use streameme_backend::handlers;
use tempfile::TempDir;

const UPLOAD_SIZE_LIMIT: usize = 2 * 1024 * 1024 * 1024; // 2 GiB

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("info"));

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

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        VideoAnalyzer::new(rx).run();
    });
    let analyzer = web::Data::new(tx);

    let tmp_dir = Arc::new(TempDir::new_in(".")?);
    let tmp_dir_2 = tmp_dir.clone();
    HttpServer::new(move || {
        let path = tmp_dir_2.path();
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
            .app_data(MultipartFormConfig::default().total_limit(UPLOAD_SIZE_LIMIT))
            .app_data(web::Data::clone(&analyzer))
            .configure(handlers::config)
    })
    .bind((Ipv4Addr::UNSPECIFIED, port))?
    .run()
    .await
}
