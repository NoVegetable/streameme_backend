use crate::inference::InferenceOutput;
use log;
use serde::Serialize;
use serde_json;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::fs;
use tokio::process::Command;

#[derive(Debug, Copy, Clone, Deserialize_repr)]
#[repr(u8)]
pub enum VideoAnalyzerMode {
    Binary = 0,
    Multi = 1,
}

#[derive(Debug, Serialize)]
#[repr(transparent)]
pub struct VideoAnalyzerModeDesc(String);

impl VideoAnalyzerModeDesc {
    #[inline]
    pub fn new(mode: VideoAnalyzerMode) -> Self {
        use VideoAnalyzerMode::*;
        Self(match mode {
            Binary => String::from("binary"),
            Multi => String::from("multi"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct VideoAnalyzerConfig {
    video_path: PathBuf,
    analyze_mode: VideoAnalyzerMode,
}

impl VideoAnalyzerConfig {
    #[inline]
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            video_path: PathBuf::from(path.as_ref()),
            analyze_mode: VideoAnalyzerMode::Multi,
        }
    }

    #[inline]
    pub fn analyze_mode(&mut self, analyze_mode: VideoAnalyzerMode) -> &mut Self {
        self.analyze_mode = analyze_mode;
        self
    }

    #[inline]
    pub fn build(&self) -> VideoAnalyzer {
        VideoAnalyzer {
            config: self.clone(),
        }
    }
}

pub struct VideoAnalyzer {
    config: VideoAnalyzerConfig,
}

impl VideoAnalyzer {
    pub async fn run(self) -> io::Result<VideoAnalyzerOutput> {
        let out_dir = TempDir::new_in(".")?;

        let command_dir = fs::canonicalize("../streameme_inference").await?;
        log::info!("executing inference.py under {:?}", command_dir);

        let output = Command::new("./.venv/bin/python")
            .current_dir(command_dir)
            .arg("inference.py")
            .arg("--video_path")
            .arg(&self.config.video_path)
            .arg("--video_name")
            .arg("video")
            .arg("--output_dir")
            .arg(out_dir.path())
            .output()
            .await?;

        if output.status.success() {
            log::info!("inference script exited successfully");
            let mut inference_out_path = PathBuf::new();
            inference_out_path.push(out_dir.path());
            inference_out_path.push("suggestions.json");
            let inference_out_str = fs::read_to_string(&inference_out_path).await?;
            let inference_output: InferenceOutput = serde_json::from_str(&inference_out_str)?;

            Ok(VideoAnalyzerOutput::from(inference_output))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!(
                "inference script exited within error; dumping stderr: {}",
                stderr
            );

            Ok(VideoAnalyzerOutput::default())
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize_repr)]
#[repr(u8)]
pub enum MemeType {
    Happiness = 0,
    Love = 1,
    Anger = 2,
    Sorrow = 3,
    Hate = 4,
    Surprise = 5,
}

#[derive(Debug, Serialize)]
#[repr(transparent)]
pub struct MemeTypeDesc(String);

impl MemeTypeDesc {
    #[inline]
    pub fn new(meme_type: MemeType) -> Self {
        use MemeType::*;

        match meme_type {
            Happiness => Self(String::from("happiness")),
            Love => Self(String::from("love")),
            Anger => Self(String::from("anger")),
            Sorrow => Self(String::from("sorrow")),
            Hate => Self(String::from("hate")),
            Surprise => Self(String::from("surprise")),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VideoAnalyzerSuggestion {
    start: u32,
    end: u32,
    meme_type: MemeType,
    meme_type_desc: MemeTypeDesc,
}

impl VideoAnalyzerSuggestion {
    #[inline]
    pub fn new(start: u32, end: u32, meme_type: MemeType) -> Self {
        Self {
            start,
            end,
            meme_type,
            meme_type_desc: MemeTypeDesc::new(meme_type),
        }
    }
}

#[derive(Debug, Serialize)]
#[repr(transparent)]
pub struct VideoAnalyzerOutput(Option<Vec<VideoAnalyzerSuggestion>>);

impl Default for VideoAnalyzerOutput {
    fn default() -> Self {
        Self(None)
    }
}

impl From<Vec<VideoAnalyzerSuggestion>> for VideoAnalyzerOutput {
    fn from(suggestions: Vec<VideoAnalyzerSuggestion>) -> Self {
        Self(Some(suggestions))
    }
}

impl From<InferenceOutput> for VideoAnalyzerOutput {
    fn from(output: InferenceOutput) -> Self {
        let suggestions: Vec<VideoAnalyzerSuggestion> = output
            .into_inner()
            .into_iter()
            .filter_map(|unit| {
                let meme_type = match unit.suggestion.as_str() {
                    "happiness" => MemeType::Happiness,
                    "love" => MemeType::Love,
                    "anger" => MemeType::Anger,
                    "sorrow" => MemeType::Sorrow,
                    "hate" => MemeType::Hate,
                    "surprise" => MemeType::Surprise,
                    _ => return None,
                };
                Some(VideoAnalyzerSuggestion::new(
                    unit.start, unit.end, meme_type,
                ))
            })
            .collect();
        Self::from(suggestions)
    }
}
