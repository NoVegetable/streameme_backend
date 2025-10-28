use super::{VideoAnalyzerBuffer, VideoAnalyzerMode, VideoAnalyzerOutput};
use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use tokio::sync::oneshot;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TaskConfig {
    video_path: PathBuf,
    video_name: Option<String>,
    analyze_mode: Option<VideoAnalyzerMode>,
}

impl TaskConfig {
    #[inline]
    pub fn new<P: AsRef<Path> + ?Sized>(path: &P) -> Self {
        Self {
            video_path: PathBuf::from(path.as_ref()),
            video_name: None,
            analyze_mode: None,
        }
    }

    #[inline]
    pub fn video_name(&mut self, video_name: &str) -> &mut Self {
        self.video_name = Some(String::from(video_name));
        self
    }

    #[inline]
    pub fn analyze_mode(&mut self, analyze_mode: VideoAnalyzerMode) -> &mut Self {
        self.analyze_mode = Some(analyze_mode);
        self
    }

    #[inline]
    pub fn build(&self) -> Task {
        Task {
            video_path: self.video_path.clone(),
            video_name: self
                .video_name
                .as_ref()
                .map_or(String::from("_anonymous"), |s| s.clone()),
            analyze_mode: self.analyze_mode.unwrap_or_default(),
        }
    }
}

/// An analysis task.
pub struct Task {
    video_path: PathBuf,
    video_name: String,
    analyze_mode: VideoAnalyzerMode,
}

impl Task {
    /// Creates a new [`Task`].
    #[allow(dead_code)]
    #[inline]
    pub fn new<P: AsRef<Path> + ?Sized>(
        video_path: &P,
        video_name: &str,
        analyze_mode: VideoAnalyzerMode,
    ) -> Self {
        Self {
            video_path: PathBuf::from(video_path.as_ref()),
            video_name: String::from(video_name),
            analyze_mode,
        }
    }

    /// Sends the task to the analyzer using `analyzer`.
    ///
    /// # Errors
    /// An error is returned when failed to send the task to the analyzer. This can occur if the
    /// analyzer has been deallocated already, implying that the wrapped receiver has also been
    /// deallocated.
    #[inline]
    pub fn spawn(
        self,
        analyzer: &VideoAnalyzerBuffer,
    ) -> Result<SpawnedTaskHandle, mpsc::SendError<Self>> {
        let (tx, rx) = oneshot::channel();
        let spawned = SpawnedTask {
            task: self,
            sender: tx,
        };
        spawned
            .spawn(analyzer)
            .map_err(|e| mpsc::SendError(e.0.task))?;
        Ok(SpawnedTaskHandle { receiver: rx })
    }

    #[inline]
    pub(super) fn video_path(&self) -> &Path {
        &self.video_path
    }

    #[inline]
    pub(super) fn video_name(&self) -> &str {
        &self.video_name
    }

    #[inline]
    pub(super) fn analyze_mode(&self) -> VideoAnalyzerMode {
        self.analyze_mode
    }
}

/// An analysis task to be sent to the analyzer. It wraps a [`Task`] inside and uses message
/// passing internally.
pub(super) struct SpawnedTask {
    task: Task,
    sender: oneshot::Sender<io::Result<VideoAnalyzerOutput>>,
}

impl SpawnedTask {
    #[inline]
    fn spawn(self, analyzer: &VideoAnalyzerBuffer) -> Result<(), mpsc::SendError<Self>> {
        analyzer.send(self)
    }

    #[inline]
    pub fn task(&self) -> &Task {
        &self.task
    }

    #[inline]
    pub fn send(
        self,
        output: io::Result<VideoAnalyzerOutput>,
    ) -> Result<(), io::Result<VideoAnalyzerOutput>> {
        self.sender.send(output)
    }
}

/// A handle to the spawned task. This can be used to receive the analysis results.
pub struct SpawnedTaskHandle {
    receiver: oneshot::Receiver<io::Result<VideoAnalyzerOutput>>,
}

impl SpawnedTaskHandle {
    /// Receive analysis results from the analyzer.
    ///
    /// # Errors
    /// An error is returned if the corresponding sender has been dropped. This usually occurs when
    /// the analyzer accidentally drops the sender before sending anything back.
    #[inline]
    pub async fn recv(self) -> Result<io::Result<VideoAnalyzerOutput>, oneshot::error::RecvError> {
        self.receiver.await
    }
}
