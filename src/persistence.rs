use crate::model::{DownloadOptions, DownloadStatus, DownloadTask};
use std::fs;
use std::io;
use std::path::Path;

pub fn save_queue(path: &Path, tasks: &[DownloadTask]) -> io::Result<()> {
    let mut content = String::from("# YVD queue state\n");
    for task in tasks {
        content.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            task.id,
            task.status.as_str(),
            task.options.output_dir.display(),
            task.options.url
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
        let fields: Vec<_> = line.splitn(4, '\t').collect();
        if fields.len() != 4 {
            continue;
        }
        let Some(status) = DownloadStatus::parse(fields[1]) else {
            continue;
        };
        let mut task = DownloadTask::new(fields[0], DownloadOptions::new(fields[3], fields[2]));
        task.status = status;
        tasks.push(task);
    }
    Ok(tasks)
}
