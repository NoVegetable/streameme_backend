pub mod upload;

use actix_web::web::ServiceConfig;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.configure(upload::config);
}
