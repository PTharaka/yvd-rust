use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Downloading => "downloading",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "queued" => Self::Queued,
            "downloading" => Self::Downloading,
            "paused" => Self::Paused,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadOptions {
    pub url: String,
    pub output_dir: PathBuf,
    pub quality: Option<u16>,
    pub audio_only: bool,
    pub use_browser_cookies: bool,
}

impl DownloadOptions {
    pub fn new(url: impl Into<String>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            url: url.into(),
            output_dir: output_dir.into(),
            quality: None,
            audio_only: false,
            use_browser_cookies: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub options: DownloadOptions,
    pub status: DownloadStatus,
    pub progress_percent: f64,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub title: Option<String>,
    pub channel: Option<String>,
    pub duration_seconds: Option<u64>,
    pub thumbnail: Option<String>,
    pub item_count: Option<usize>,
}

impl DownloadTask {
    pub fn new(id: impl Into<String>, options: DownloadOptions) -> Self {
        Self {
            id: id.into(),
            options,
            status: DownloadStatus::Queued,
            progress_percent: 0.0,
            error: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum DownloadEvent {
    Log {
        task_id: String,
        message: String,
    },
    Progress {
        task_id: String,
        percent: f64,
    },
    StatusChanged {
        task_id: String,
        status: DownloadStatus,
    },
}
