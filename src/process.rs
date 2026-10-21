use crate::command::build_yt_dlp_args;
use crate::model::{DownloadEvent, DownloadOptions, DownloadStatus};
use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};

pub struct DownloadProcess {
    child: Child,
    task_id: String,
}

impl DownloadProcess {
    pub fn start(
        task_id: impl Into<String>,
        options: &DownloadOptions,
        yt_dlp: &Path,
        ffmpeg: &Path,
    ) -> io::Result<Self> {
        let args = build_yt_dlp_args(options, yt_dlp, ffmpeg);
        let child = Command::new(&args[0])
            .args(&args[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(Self {
            child,
            task_id: task_id.into(),
        })
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn pause(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    pub fn cancel(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    pub fn finish(mut self) -> io::Result<DownloadStatus> {
        let status = self.child.wait()?;
        Ok(if status.success() {
            DownloadStatus::Completed
        } else {
            DownloadStatus::Failed
        })
    }

    pub fn status_event(&self, status: DownloadStatus) -> DownloadEvent {
        DownloadEvent::StatusChanged {
            task_id: self.task_id.clone(),
            status,
        }
    }
}
