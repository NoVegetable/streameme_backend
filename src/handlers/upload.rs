use crate::analyzer::task::TaskConfig;
use crate::analyzer::{
    VideoAnalyzerBuffer, VideoAnalyzerMode, VideoAnalyzerModeDesc, VideoAnalyzerOutput,
};
use crate::handlers::utils;
use actix_multipart::form::{MultipartForm, json::Json as MpJson, tempfile::TempFile};
use actix_web::error::Error;
use actix_web::web::{self, ServiceConfig};
use actix_web::{HttpResponse, Responder, post};
use log;
use mime;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// Makes `OffsetDateTime` serialized to a format that can be parsed by JS Date.
// Reference: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date#date_time_string_format
time::serde::format_description!(
    js_format,
    OffsetDateTime,
    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
);

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
    #[serde(with = "js_format")]
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
    analyzer: web::Data<VideoAnalyzerBuffer>,
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
    let task = TaskConfig::new(form.file.file.path())
        .analyze_mode(mdata.mode)
        .video_name(video_name)
        .build();

    // Sends the task to the analyzer.
    log::debug!("sending analysis task to the analyzer");
    let Ok(handle) = task.spawn(&analyzer) else {
        log::debug!(
            "failed to send task to the analyzer, indicating that the receiving-half might have been dropped"
        );
        return Ok(HttpResponse::InternalServerError().body("internal communication broken"));
    };

    // Awaits the analysis results and then constructs the response.
    if let Ok(output) = handle.recv().await {
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
