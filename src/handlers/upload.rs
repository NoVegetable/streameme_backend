use crate::analyzer::{VideoAnalyzerConfig, VideoAnalyzerMode};
use actix_multipart::form::{MultipartForm, json::Json as MpJson, tempfile::TempFile};
use actix_web::error::Error;
use actix_web::web::ServiceConfig;
use actix_web::{HttpResponse, Responder, post};
use log;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UploadFormMetadata {
    mode: VideoAnalyzerMode,
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    file: TempFile,
    metadata: MpJson<UploadFormMetadata>,
}

#[post("/upload")]
pub async fn upload_video(
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder, Error> {
    let file_name = match form.file.file_name.as_ref() {
        Some(f) => f,
        None => {
            return Ok(HttpResponse::BadRequest().body("file name is missing"));
        }
    };

    log::info!(
        "receive file: name = {:?}, size = {:?} bytes",
        file_name,
        form.file.size,
    );

    // TODO: Analyze the received video with our model.
    // This can be done by invoking another process that
    // executes an evaluation script.
    let mdata = form.metadata.into_inner();
    let analyzer = VideoAnalyzerConfig::new(form.file.file.path())
        .analyze_mode(mdata.mode)
        .build();
    let output = analyzer.run().await?;

    // TODO: Access the analysis results and build response.
    // Build abstraction upon both the analysis results and
    // the response body, both should be serializable type.

    Ok(HttpResponse::Ok().json(output))
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(upload_video);
}
