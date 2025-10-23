use crate::analyzer::Task;
use crate::analyzer::{
    VideoAnalyzerConfig, VideoAnalyzerMode, VideoAnalyzerModeDesc, VideoAnalyzerOutput,
};
use crate::handlers::utils;
use actix_multipart::form::{MultipartForm, json::Json as MpJson, tempfile::TempFile};
use actix_web::error::Error;
use actix_web::web::{self, ServiceConfig};
use actix_web::{HttpResponse, Responder, post};
use log;
use mime;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use time::OffsetDateTime;
use tokio::sync::oneshot;

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
    analyzer: web::Data<mpsc::Sender<Task>>,
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

    // Checks if the video format is supported.
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

    // Constructs analysis task. We need to complete the analysis config and setup a oneshot channel
    // for receiving analysis resutls. All the stuff is then wrapped into a `Task` instance.
    let mdata = form.metadata.into_inner();
    let mut config = VideoAnalyzerConfig::new(form.file.file.path());
    config.analyze_mode(mdata.mode).video_name(video_name);
    let (tx, rx) = oneshot::channel();
    let task = Task::new(config, tx);

    // Sends the task to the analyzer.
    log::debug!("sending analysis task to the analyzer");
    if task.spawn(&analyzer).is_err() {
        log::debug!(
            "failed to send task to the analyzer, indicating that the receiving-half might have been dropped"
        );
        return Ok(HttpResponse::InternalServerError().body("internal communication broken"));
    }

    // Awaits the analysis results and then constructs the response.
    if let Ok(output) = rx.await {
        let output = output?;
        let res = UploadResponse::new(file_name, mdata.mode, output);
        Ok(HttpResponse::Ok().json(res))
    } else {
        log::debug!(
            "failed to receive analysis results from the analyzer, indicating that the sending-half might have been dropped"
        );
        Ok(HttpResponse::InternalServerError().body("internal communication broken"))
    }
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(upload_video);
}
