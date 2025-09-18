use crate::error::Error;
use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::web::ServiceConfig;
use actix_web::{HttpResponse, Responder, post};
use log;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    file: TempFile,
}

#[post("/upload")]
pub async fn upload_video(
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder, Error> {
    let file_name = form.file.file_name.as_ref().unwrap();
    log::info!(
        "temp file name: {:?}, size: {:?} bytes",
        file_name,
        form.file.size,
    );

    if let Err(e) = fs::create_dir("uploads")
        && e.kind() != io::ErrorKind::AlreadyExists
    {
        log::error!("failed to create `uploads` directory");
        return Ok(HttpResponse::InternalServerError().finish());
    }
    let path: PathBuf = ["uploads", file_name].iter().collect();
    if let Err(_) = form.file.file.persist(&path) {
        log::warn!("uploaded file dropped without updating");
        Ok(HttpResponse::InternalServerError().finish())
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(upload_video);
}
