use crate::model::VideoMetadata;
use serde::Deserialize;
use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct YtDlpMetadata {
    title: Option<String>,
    channel: Option<String>,
    uploader: Option<String>,
    duration: Option<f64>,
    thumbnail: Option<String>,
    entries: Option<Vec<serde_json::Value>>,
}

pub fn probe(yt_dlp: &Path, url: &str, use_browser_cookies: bool) -> io::Result<VideoMetadata> {
    let mut command = Command::new(yt_dlp);
    command.args(["--dump-single-json", "--skip-download", "--no-warnings"]);
    if use_browser_cookies {
        command.args(["--cookies-from-browser", "chrome"]);
    }
    let output = command.arg(url).output()?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(io::Error::other(if error.is_empty() {
            "yt-dlp could not read metadata."
        } else {
            &error
        }));
    }

    let metadata: YtDlpMetadata = serde_json::from_slice(&output.stdout)
        .map_err(|error| io::Error::other(format!("Invalid metadata response: {error}")))?;
    Ok(VideoMetadata {
        title: metadata.title,
        channel: metadata.channel.or(metadata.uploader),
        duration_seconds: metadata.duration.map(|value| value.round() as u64),
        thumbnail: metadata.thumbnail,
        item_count: metadata.entries.map(|entries| entries.len()),
    })
}
