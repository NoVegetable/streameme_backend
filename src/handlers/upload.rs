use crate::error::Error;
use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::web::ServiceConfig;
use actix_web::{HttpResponse, Responder, post};
use log;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    file: TempFile,
}

#[post("/upload")]
pub async fn upload_video(
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder, Error> {
    let file_name = match form.file.file_name.as_ref() {
        Some(f) => f,
        None => return Ok(HttpResponse::BadRequest().finish()),
    };

    log::info!(
        "receive file: name = {:?}, size = {:?} bytes",
        file_name,
        form.file.size,
    );

    // TODO: Analyze the received video with our model.
    // This can be done by invoking another process that
    // executes an evaluation script.

    // TODO: Access the analysis results and build response.
    // Build abstraction upon both the analysis results and
    // the response body, both should be serializable type.

    Ok(HttpResponse::Ok().finish())
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(upload_video);
}
