use crate::model::{DownloadOptions, DownloadStatus, DownloadTask};
use std::fs;
use std::io;
use std::path::Path;

pub fn save_queue(path: &Path, tasks: &[DownloadTask]) -> io::Result<()> {
    let mut content = String::from("# YVD queue state\n");
    for task in tasks {
        content.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            task.id,
            task.status.as_str(),
            task.options.output_dir.display(),
            task.options.url,
            task.progress_percent,
            task.error
                .as_deref()
                .unwrap_or("")
                .replace(['\t', '\r', '\n'], " "),
            task.options.quality.map_or_else(String::new, |value| value.to_string()),
            task.options.audio_only,
            task.options.use_browser_cookies,
            task.options.max_retries
        ));
    }

    fs::write(path, content)
}

pub fn load_queue(path: &Path) -> io::Result<Vec<DownloadTask>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut tasks = Vec::new();
    for line in fs::read_to_string(path)?
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let fields: Vec<_> = line.splitn(10, '\t').collect();
        if fields.len() < 4 {
            continue;
        }
        let Some(status) = DownloadStatus::parse(fields[1]) else {
            continue;
        };
        let mut task = DownloadTask::new(fields[0], DownloadOptions::new(fields[3], fields[2]));
        task.status = status;
        task.progress_percent = fields
            .get(4)
            .and_then(|value| value.parse().ok())
            .unwrap_or(0.0);
        task.error = fields
            .get(5)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string());
        task.options.quality = fields.get(6).and_then(|value| value.parse().ok());
        task.options.audio_only = fields.get(7).is_some_and(|value| *value == "true");
        task.options.use_browser_cookies = fields.get(8).is_some_and(|value| *value == "true");
        task.options.max_retries = fields
            .get(9)
            .and_then(|value| value.parse().ok())
            .unwrap_or(task.options.max_retries);
        tasks.push(task);
    }
    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn round_trips_progress_and_error() {
        let path = std::env::temp_dir().join(format!(
            "yvd-queue-{}.state",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after the Unix epoch")
                .as_nanos()
        ));
        let mut task = DownloadTask::new(
            "download-1",
            DownloadOptions::new("https://example.com/video", "downloads"),
        );
        task.progress_percent = 42.5;
        task.error = Some("network failure".to_string());

        save_queue(&path, &[task]).expect("queue should save");
        let loaded = load_queue(&path).expect("queue should load");
        std::fs::remove_file(&path).expect("temporary queue should be removed");

        assert_eq!(loaded[0].progress_percent, 42.5);
        assert_eq!(loaded[0].error.as_deref(), Some("network failure"));
    }
}
