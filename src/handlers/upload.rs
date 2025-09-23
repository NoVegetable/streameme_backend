use crate::analyzer::{
    VideoAnalyzerConfig, VideoAnalyzerMode, VideoAnalyzerModeDesc, VideoAnalyzerOutput,
};
use actix_multipart::form::{MultipartForm, json::Json as MpJson, tempfile::TempFile};
use actix_web::error::Error;
use actix_web::web::ServiceConfig;
use actix_web::{HttpResponse, Responder, post};
use log;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Deserialize)]
struct UploadFormMetadata {
    mode: VideoAnalyzerMode,
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "2GB")]
    file: TempFile,
    metadata: MpJson<UploadFormMetadata>,
}

#[derive(Debug, Serialize)]
struct UploadResponse {
    file_name: String,
    analyze_time: OffsetDateTime,
    analyze_mode: VideoAnalyzerModeDesc,
    suggestions: VideoAnalyzerOutput,
}

impl UploadResponse {
    pub fn new(
        file_name: &String,
        analyze_mode: VideoAnalyzerMode,
        suggestions: VideoAnalyzerOutput,
    ) -> Self {
        Self {
            file_name: file_name.clone(),
            analyze_time: OffsetDateTime::now_utc(),
            analyze_mode: VideoAnalyzerModeDesc::new(analyze_mode),
            suggestions,
        }
    }
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

    let mdata = form.metadata.into_inner();
    let analyzer = VideoAnalyzerConfig::new(form.file.file.path())
        .analyze_mode(mdata.mode)
        .build();
    let output = analyzer.run().await?;

    let res = UploadResponse::new(file_name, mdata.mode, output);

    Ok(HttpResponse::Ok().json(res))
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(upload_video);
}
