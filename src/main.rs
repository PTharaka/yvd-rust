use std::env;
use std::path::PathBuf;
use yvd_core::{metadata, persistence, DownloadOptions, DownloadTask};

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("metadata") => {
            let Some(url) = args.next() else {
                eprintln!("Usage: yvd-cli metadata <youtube-url>");
                return;
            };
            match metadata::probe(PathBuf::from("yt-dlp").as_path(), &url, false) {
                Ok(value) => println!("{value:?}"),
                Err(error) => eprintln!("Metadata lookup failed: {error}"),
            }
        }
        Some("add") => {
            let Some(url) = args.next() else {
                eprintln!("Usage: yvd-cli add <youtube-url> [output-directory]");
                return;
            };
            let output_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("downloads"));
            let queue_file = PathBuf::from("queue.state");
            let mut tasks = persistence::load_queue(&queue_file).unwrap_or_default();
            let task = DownloadTask::new(
                format!("download-{}", tasks.len() + 1),
                DownloadOptions::new(url, output_dir),
            );
            tasks.push(task);
            if let Err(error) = persistence::save_queue(&queue_file, &tasks) {
                eprintln!("Could not save queue: {error}");
            } else {
                println!("Added download to {}", queue_file.display());
            }
        }
        Some("list") => {
            let queue_file = PathBuf::from("queue.state");
            match persistence::load_queue(&queue_file) {
                Ok(tasks) if tasks.is_empty() => println!("Queue is empty."),
                Ok(tasks) => {
                    for task in tasks {
                        println!(
                            "{}\t{}\t{}",
                            task.id,
                            task.status.as_str(),
                            task.options.url
                        );
                    }
                }
                Err(error) => eprintln!("Could not load queue: {error}"),
            }
        }
        _ => eprintln!("Usage: yvd-cli metadata <url> | add <url> [output-directory] | list"),
    }
}
