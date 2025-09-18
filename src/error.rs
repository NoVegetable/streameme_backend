use actix_web::ResponseError;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl ResponseError for Error {}
