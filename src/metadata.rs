use crate::model::VideoMetadata;
use std::io;
use std::path::Path;
use std::process::Command;

pub fn probe(yt_dlp: &Path, url: &str) -> io::Result<VideoMetadata> {
    let output = Command::new(yt_dlp)
        .args(["--dump-single-json", "--skip-download", url])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim(),
        ));
    }
    let json = String::from_utf8_lossy(&output.stdout);
    Ok(VideoMetadata {
        title: json_string(&json, "title"),
        channel: json_string(&json, "channel").or_else(|| json_string(&json, "uploader")),
        duration_seconds: json_number(&json, "duration"),
        thumbnail: json_string(&json, "thumbnail"),
    })
}

fn json_string(json: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":");
    let value = json.split_once(&marker)?.1.trim_start();
    let value = value.strip_prefix('"')?;
    Some(value.split('"').next()?.replace("\\\"", "\""))
}

fn json_number(json: &str, key: &str) -> Option<u64> {
    let marker = format!("\"{key}\":");
    json.split_once(&marker)?
        .1
        .trim_start()
        .split([',', '}'])
        .next()?
        .parse()
        .ok()
}
