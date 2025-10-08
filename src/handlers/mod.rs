pub mod upload;
pub mod utils;

use actix_web::web::ServiceConfig;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.configure(upload::config);
}
