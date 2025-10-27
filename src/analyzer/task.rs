use super::{VideoAnalyzerBuffer, VideoAnalyzerMode, VideoAnalyzerOutput};
use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub struct TaskConfig {
    video_name: String,
    video_path: PathBuf,
    analyze_mode: VideoAnalyzerMode,
}

impl TaskConfig {
    #[inline]
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            video_name: String::from("video"),
            video_path: PathBuf::from(path.as_ref()),
            analyze_mode: VideoAnalyzerMode::Multi,
        }
    }

    #[inline]
    pub fn video_name(&mut self, video_name: &str) -> &mut Self {
        video_name.clone_into(&mut self.video_name);
        self
    }

    #[inline]
    pub fn analyze_mode(&mut self, analyze_mode: VideoAnalyzerMode) -> &mut Self {
        self.analyze_mode = analyze_mode;
        self
    }

    #[inline]
    pub fn build(&self) -> Task {
        Task::new(self.clone())
    }
}

/// An analysis task.
pub struct Task {
    config: TaskConfig,
}

impl Task {
    /// Creates a new [`Task`].
    #[inline]
    pub fn new(config: TaskConfig) -> Self {
        Self { config }
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
        &self.config.video_path
    }

    #[inline]
    pub(super) fn video_name(&self) -> &str {
        &self.config.video_name
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
