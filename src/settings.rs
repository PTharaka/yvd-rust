use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Settings {
    pub output_dir: PathBuf,
    pub quality: Option<u16>,
    pub audio_only: bool,
    pub use_browser_cookies: bool,
    pub max_retries: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("downloads"),
            quality: None,
            audio_only: false,
            use_browser_cookies: false,
            max_retries: 2,
        }
    }
}

pub fn save(path: &Path, settings: &Settings) -> io::Result<()> {
    let content = format!(
        "output_dir={}\nquality={}\naudio_only={}\nuse_browser_cookies={}\nmax_retries={}\n",
        settings.output_dir.to_string_lossy(),
        settings
            .quality
            .map_or_else(|| "best".into(), |v| v.to_string()),
        settings.audio_only,
        settings.use_browser_cookies,
        settings.max_retries
    );
    std::fs::write(path, content)
}

pub fn load(path: &Path) -> io::Result<Settings> {
    let mut settings = Settings::default();
    for line in std::fs::read_to_string(path)?.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "output_dir" => settings.output_dir = PathBuf::from(value),
            "quality" if value != "best" => settings.quality = value.parse().ok(),
            "audio_only" => settings.audio_only = value == "true",
            "use_browser_cookies" => settings.use_browser_cookies = value == "true",
            "max_retries" => settings.max_retries = value.parse().unwrap_or(settings.max_retries),
            _ => {}
        }
    }
    Ok(settings)
}
