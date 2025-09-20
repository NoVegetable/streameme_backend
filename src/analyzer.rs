use log;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Copy, Clone, Deserialize_repr)]
#[repr(u8)]
pub enum VideoAnalyzerMode {
    Binary = 0,
    Multi = 1,
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
        log::info!("video at: {:?}", self.config.video_path);
        Ok(VideoAnalyzerOutput::new(vec![
            VideoAnalyzerSuggestion::new(30, 60, MemeType::Surprise),
        ]))
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
pub struct MemeTypeDesc(String);

impl MemeTypeDesc {
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
pub struct VideoAnalyzerOutput(Option<Vec<VideoAnalyzerSuggestion>>);

impl VideoAnalyzerOutput {
    pub fn new(suggestions: Vec<VideoAnalyzerSuggestion>) -> Self {
        Self(Some(suggestions))
    }
}
