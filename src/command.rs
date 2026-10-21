use crate::model::DownloadOptions;
use std::path::Path;

pub fn build_yt_dlp_args(options: &DownloadOptions, yt_dlp: &Path, ffmpeg: &Path) -> Vec<String> {
    let mut args = vec![
        yt_dlp.to_string_lossy().into_owned(),
        "--force-ipv4".into(),
        "--concurrent-fragments".into(),
        "4".into(),
        "--retries".into(),
        options.max_retries.to_string(),
        "--fragment-retries".into(),
        "10".into(),
        "--continue".into(),
        "--no-overwrites".into(),
        "--ffmpeg-location".into(),
        ffmpeg
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_string_lossy()
            .into_owned(),
        "-o".into(),
        format!(
            "{}",
            options
                .output_dir
                .join("%(title)s.%(ext)s")
                .to_string_lossy()
        ),
    ];

    if options.use_browser_cookies {
        args.extend(["--cookies-from-browser".into(), "chrome".into()]);
    }

    if options.audio_only {
        args.extend([
            "-x".into(),
            "--audio-format".into(),
            "mp3".into(),
            "-f".into(),
            "bestaudio/best".into(),
        ]);
    } else if let Some(height) = options.quality {
        args.extend([
            "--merge-output-format".into(),
            "mp4".into(),
            "-S".into(),
            format!("res:{height},br,fps"),
            "-f".into(),
            format!("bestvideo[height<={height}]+bestaudio/best[height<={height}]/best"),
        ]);
    } else {
        args.extend([
            "--merge-output-format".into(),
            "mp4".into(),
            "-S".into(),
            "res,br,fps".into(),
            "-f".into(),
            "bestvideo+bestaudio/best".into(),
        ]);
    }

    args.push(options.url.clone());
    args
}
