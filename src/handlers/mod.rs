pub mod meme;
pub mod results;
pub mod upload;
pub mod video;

use actix_web::web::ServiceConfig;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.configure(upload::config)
        .configure(results::config)
        .configure(video::config)
        .configure(meme::config);
}
