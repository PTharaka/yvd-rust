use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use std::process::{Child, Stdio};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use yvd_rust::command::build_yt_dlp_args;
use yvd_rust::{DownloadOptions, DownloadStatus, DownloadTask};

#[derive(Default)]
struct AppState {
    processes: Arc<Mutex<HashMap<String, Arc<Mutex<Child>>>>>,
    progress: Arc<Mutex<HashMap<String, DownloadProgress>>>,
}

#[derive(Clone, Default, Serialize)]
struct DownloadProgress {
    speed: Option<String>,
    eta: Option<String>,
}

#[derive(Serialize)]
struct QueueItem {
    id: String,
    url: String,
    status: DownloadStatus,
    progress_percent: f64,
    speed: Option<String>,
    eta: Option<String>,
    error: Option<String>,
}

fn queue_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let directory = app.path().app_data_dir().map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join("queue.state"))
}

fn settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let directory = app.path().app_data_dir().map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join("settings.conf"))
}

fn load_settings(app: &tauri::AppHandle) -> Result<yvd_rust::settings::Settings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(yvd_rust::settings::Settings::default());
    }
    yvd_rust::settings::load(&path).map_err(|error| error.to_string())
}

fn load_tasks(app: &tauri::AppHandle) -> Result<(std::path::PathBuf, Vec<DownloadTask>), String> {
    let path = queue_path(app)?;
    let tasks = yvd_rust::persistence::load_queue(&path).map_err(|error| error.to_string())?;
    Ok((path, tasks))
}

fn save_tasks(path: &std::path::Path, tasks: &[DownloadTask]) -> Result<(), String> {
    yvd_rust::persistence::save_queue(path, tasks).map_err(|error| error.to_string())
}

fn parse_progress(line: &str) -> Option<(f64, Option<String>, Option<String>)> {
    let percent_end = line.find('%')?;
    let percent_start = line[..percent_end]
        .rfind(|character: char| !character.is_ascii_digit() && character != '.')
        .map_or(0, |index| index + 1);
    let percent = line[percent_start..percent_end].parse().ok()?;
    let speed = line
        .split_whitespace()
        .find(|value| value.ends_with("/s"))
        .map(str::to_string);
    let eta = line
        .split_whitespace()
        .position(|value| value == "ETA")
        .and_then(|index| line.split_whitespace().nth(index + 1))
        .map(str::to_string);
    Some((percent, speed, eta))
}

fn find_task_mut<'a>(tasks: &'a mut [DownloadTask], id: &str) -> Result<&'a mut DownloadTask, String> {
    tasks
        .iter_mut()
        .find(|task| task.id == id)
        .ok_or_else(|| format!("Download {id} was not found."))
}

#[tauri::command]
fn queue_status(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<QueueItem>, String> {
    let (_, tasks) = load_tasks(&app)?;
    Ok(tasks
        .into_iter()
        .map(|task| {
            let live_progress = state
                .progress
                .lock()
                .ok()
                .and_then(|progress| progress.get(&task.id).cloned())
                .unwrap_or_default();
            QueueItem {
                speed: live_progress.speed,
                eta: live_progress.eta,
                id: task.id,
                url: task.options.url,
                status: task.status,
                progress_percent: task.progress_percent,
                error: task.error,
            }
        })
        .collect())
}

#[tauri::command]
fn queue_add(app: tauri::AppHandle, url: String) -> Result<String, String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("A YouTube URL is required.".to_string());
    }
    let (path, mut tasks) = load_tasks(&app)?;
    let settings = load_settings(&app)?;
    let id = format!("download-{}", uuid::Uuid::new_v4());
    let mut options = DownloadOptions::new(url, settings.output_dir);
    options.quality = settings.quality;
    options.audio_only = settings.audio_only;
    options.use_browser_cookies = settings.use_browser_cookies;
    tasks.push(DownloadTask::new(&id, options));
    save_tasks(&path, &tasks)?;
    Ok(id)
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<yvd_rust::settings::Settings, String> {
    load_settings(&app)
}

#[tauri::command]
fn save_settings(
    app: tauri::AppHandle,
    settings: yvd_rust::settings::Settings,
) -> Result<(), String> {
    settings.validate().map_err(|error| error.to_string())?;
    let path = settings_path(&app)?;
    yvd_rust::settings::save(&path, &settings).map_err(|error| error.to_string())
}

#[tauri::command]
fn metadata_preview(url: String) -> Result<yvd_rust::model::VideoMetadata, String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("A video or playlist URL is required.".to_string());
    }
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("Enter a valid HTTP or HTTPS URL.".to_string());
    }
    yvd_rust::metadata::probe(std::path::Path::new("yt-dlp"), url)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn start_download(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let (path, mut tasks) = load_tasks(&app)?;
    let task = find_task_mut(&mut tasks, &id)?;
    if !matches!(task.status, DownloadStatus::Queued | DownloadStatus::Paused | DownloadStatus::Failed) {
        return Err(format!("Download {} cannot be started from {}.", id, task.status.as_str()));
    }
    let options = task.options.clone();
    task.status = DownloadStatus::Downloading;
    task.error = None;
    save_tasks(&path, &tasks)?;

    let yt_dlp = std::path::PathBuf::from("yt-dlp");
    let ffmpeg = std::path::PathBuf::from("ffmpeg");
    let args = build_yt_dlp_args(&options, &yt_dlp, &ffmpeg);
    let mut child = match std::process::Command::new(&args[0])
        .args(&args[1..])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            let (_, mut tasks) = load_tasks(&app)?;
            let task = find_task_mut(&mut tasks, &id)?;
            task.status = DownloadStatus::Failed;
            task.error = Some(format!("Could not start yt-dlp: {error}"));
            save_tasks(&path, &tasks)?;
            return Err(error.to_string());
        }
    };
    let stderr = child.stderr.take();
    let child = Arc::new(Mutex::new(child));
    state.processes.lock().map_err(|_| "Process state is unavailable.".to_string())?.insert(id.clone(), Arc::clone(&child));

    let app = app.clone();
    let processes = Arc::clone(&state.processes);
    let progress = Arc::clone(&state.progress);
    let reader_progress = Arc::clone(&progress);
    let progress_app = app.clone();
    let progress_id = id.clone();
    if let Some(stderr) = stderr {
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Some((percent, speed, eta)) = parse_progress(&line) {
                    if let Ok((path, mut tasks)) = load_tasks(&progress_app) {
                        if let Ok(task) = find_task_mut(&mut tasks, &progress_id) {
                            task.progress_percent = percent.clamp(0.0, 100.0);
                            let _ = save_tasks(&path, &tasks);
                        }
                    }
                    if let Ok(mut values) = reader_progress.lock() {
                        values.insert(progress_id.clone(), DownloadProgress { speed, eta });
                    }
                }
            }
        });
    }
    std::thread::spawn(move || {
        let result = child.lock().ok().and_then(|mut process| process.wait().ok());
        let _ = (|| -> Result<(), String> {
            let (path, mut tasks) = load_tasks(&app)?;
            let task = find_task_mut(&mut tasks, &id)?;
            if task.status == DownloadStatus::Downloading {
                task.status = if result.map(|status| status.success()).unwrap_or(false) {
                    DownloadStatus::Completed
                } else {
                    task.error = Some("The download process failed.".to_string());
                    DownloadStatus::Failed
                };
            }
            save_tasks(&path, &tasks)?;
            processes.lock().map_err(|_| "Process state is unavailable.".to_string())?.remove(&id);
            progress.lock().map_err(|_| "Progress state is unavailable.".to_string())?.remove(&id);
            Ok(())
        })();
    });
    Ok(())
}

#[tauri::command]
fn pause_download(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    stop_download(app, state, id, DownloadStatus::Paused)
}

#[tauri::command]
fn cancel_download(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    stop_download(app, state, id, DownloadStatus::Cancelled)
}

fn stop_download(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
    status: DownloadStatus,
) -> Result<(), String> {
    let process = state.processes.lock().map_err(|_| "Process state is unavailable.".to_string())?.remove(&id);
    let Some(process) = process else {
        return Err(format!("Download {id} is not running."));
    };
    process.lock().map_err(|_| "Process state is unavailable.".to_string())?.kill().map_err(|error| error.to_string())?;
    let (path, mut tasks) = load_tasks(&app)?;
    find_task_mut(&mut tasks, &id)?.status = status;
    save_tasks(&path, &tasks)
}

#[tauri::command]
fn retry_download(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let (path, mut tasks) = load_tasks(&app)?;
    let task = find_task_mut(&mut tasks, &id)?;
    if task.status != DownloadStatus::Failed {
        return Err("Only failed downloads can be retried.".to_string());
    }
    task.status = DownloadStatus::Queued;
    task.error = None;
    save_tasks(&path, &tasks)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            queue_status,
            queue_add,
            get_settings,
            save_settings,
            metadata_preview,
            start_download,
            pause_download,
            cancel_download,
            retry_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}

fn main() {
    run();
}
