use actix_web::ResponseError;
use std::io;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IoError(#[from] io::Error),
}

impl ResponseError for Error {}
