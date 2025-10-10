use crate::inference::InferenceOutput;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::fs;
use tokio::process::Command;

#[derive(Debug, Copy, Clone, Deserialize_repr)]
#[repr(u8)]
pub(crate) enum VideoAnalyzerMode {
    Binary = 0,
    Multi = 1,
}

#[derive(Debug, Serialize)]
#[repr(transparent)]
pub(crate) struct VideoAnalyzerModeDesc(String);

impl VideoAnalyzerModeDesc {
    #[inline]
    pub(crate) fn new(mode: VideoAnalyzerMode) -> Self {
        use VideoAnalyzerMode::*;
        Self(match mode {
            Binary => String::from("binary"),
            Multi => String::from("multi"),
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VideoAnalyzerConfig {
    video_name: String,
    video_path: PathBuf,
    analyze_mode: VideoAnalyzerMode,
}

impl VideoAnalyzerConfig {
    #[inline]
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            video_name: String::from("video"),
            video_path: PathBuf::from(path.as_ref()),
            analyze_mode: VideoAnalyzerMode::Multi,
        }
    }

    #[inline]
    pub(crate) fn video_name(&mut self, video_name: &str) -> &mut Self {
        video_name.clone_into(&mut self.video_name);
        self
    }

    #[inline]
    pub(crate) fn analyze_mode(&mut self, analyze_mode: VideoAnalyzerMode) -> &mut Self {
        self.analyze_mode = analyze_mode;
        self
    }

    #[inline]
    pub(crate) fn build(&self) -> VideoAnalyzer {
        VideoAnalyzer {
            config: self.clone(),
        }
    }
}

pub(crate) struct VideoAnalyzer {
    config: VideoAnalyzerConfig,
}

impl VideoAnalyzer {
    pub(crate) async fn run(self) -> io::Result<VideoAnalyzerOutput> {
        let out_dir = TempDir::new_in(".")?;
        let command_dir = fs::canonicalize("../streameme_inference").await?;

        log::info!("starting inference procedure");
        log::debug!("executing inference.py under {}", command_dir.display());
        log::debug!(
            "running command: ./.venv/bin/python inference.py --video_path {} --video_name {} --output_dir {}",
            self.config.video_path.display(),
            &self.config.video_name,
            out_dir.path().display()
        );

        let output = Command::new("./.venv/bin/python")
            .current_dir(command_dir)
            .arg("inference.py")
            .arg("--video_path")
            .arg(&self.config.video_path)
            .arg("--video_name")
            .arg(&self.config.video_name)
            .arg("--output_dir")
            .arg(out_dir.path())
            .output()
            .await?;

        if output.status.success() {
            log::info!("inference procedure exited successfully");
            let mut inference_out_path = PathBuf::new();
            inference_out_path.push(out_dir.path());
            inference_out_path.push("suggestions.json");
            log::debug!(
                "parsing inference results from {}",
                inference_out_path.display()
            );
            let inference_out_str = fs::read_to_string(&inference_out_path).await?;
            let inference_output: InferenceOutput = serde_json::from_str(&inference_out_str)?;

            Ok(VideoAnalyzerOutput::from(inference_output))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!(
                "inference procedure exited within error; dumping stderr:\n{}",
                stderr
            );

            Ok(VideoAnalyzerOutput::default())
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize_repr)]
#[repr(u8)]
enum MemeType {
    Happiness = 0,
    Love = 1,
    Anger = 2,
    Sorrow = 3,
    Hate = 4,
    Surprise = 5,
}

#[derive(Debug, Serialize)]
#[repr(transparent)]
struct MemeTypeDesc(String);

impl MemeTypeDesc {
    #[inline]
    fn new(meme_type: MemeType) -> Self {
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
struct VideoAnalyzerSuggestion {
    start: u32,
    end: u32,
    meme_type: MemeType,
    meme_type_desc: MemeTypeDesc,
}

impl VideoAnalyzerSuggestion {
    #[inline]
    fn new(start: u32, end: u32, meme_type: MemeType) -> Self {
        Self {
            start,
            end,
            meme_type,
            meme_type_desc: MemeTypeDesc::new(meme_type),
        }
    }
}

#[derive(Debug, Default, Serialize)]
#[repr(transparent)]
pub(crate) struct VideoAnalyzerOutput(Option<Vec<VideoAnalyzerSuggestion>>);

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
