use crate::analyzer::{
    VideoAnalyzerConfig, VideoAnalyzerMode, VideoAnalyzerModeDesc, VideoAnalyzerOutput,
};
use crate::handlers::utils;
use actix_multipart::form::{MultipartForm, json::Json as MpJson, tempfile::TempFile};
use actix_web::error::Error;
use actix_web::web::ServiceConfig;
use actix_web::{HttpResponse, Responder, post};
use log;
use mime;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

const SUPPORTED_VIDEO_FORMATS: [&str; 3] = ["mp4", "avi", "mov"];

#[derive(Debug, Deserialize)]
struct UploadFormMetadata {
    mode: VideoAnalyzerMode,
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
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
    fn new(
        file_name: &str,
        analyze_mode: VideoAnalyzerMode,
        suggestions: VideoAnalyzerOutput,
    ) -> Self {
        Self {
            file_name: file_name.to_owned(),
            analyze_time: OffsetDateTime::now_utc(),
            analyze_mode: VideoAnalyzerModeDesc::new(analyze_mode),
            suggestions,
        }
    }
}

#[post("/upload")]
async fn upload_video(
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder, Error> {
    let Some(file_name) = form.file.file_name.as_ref() else {
        return Ok(HttpResponse::BadRequest().body("file name is missing"));
    };

    log::info!(
        "file received: \"{}\", size: {} bytes, content type: {}",
        file_name,
        form.file.size,
        form.file
            .content_type
            .unwrap_or(mime::APPLICATION_OCTET_STREAM)
            .essence_str()
    );

    let video_name = if let (Some(video_name), Some(ext)) = utils::split_file_name(file_name)
        && SUPPORTED_VIDEO_FORMATS.contains(&ext.to_str().unwrap())
    {
        video_name.to_str().unwrap()
    } else {
        return Ok(HttpResponse::BadRequest().body(format!(
            "supported video formats are: {}",
            SUPPORTED_VIDEO_FORMATS.join(", ")
        )));
    };

    let mdata = form.metadata.into_inner();
    let analyzer = VideoAnalyzerConfig::new(form.file.file.path())
        .analyze_mode(mdata.mode)
        .video_name(video_name)
        .build();
    let output = analyzer.run().await?;

    let res = UploadResponse::new(file_name, mdata.mode, output);

    Ok(HttpResponse::Ok().json(res))
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(upload_video);
}
