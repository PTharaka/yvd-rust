use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub output_dir: PathBuf,
    pub quality: Option<u16>,
    pub audio_only: bool,
    pub use_browser_cookies: bool,
    pub max_retries: u32,
}

impl Settings {
    pub fn validate(&self) -> io::Result<()> {
        if self.output_dir.as_os_str().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Output directory cannot be empty.",
            ));
        }
        if let Some(quality) = self.quality {
            if quality == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Quality must be greater than zero.",
                ));
            }
        }
        Ok(())
    }
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
    settings.validate()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads_settings() {
        let path = std::env::temp_dir().join("yvd-settings-test.conf");
        let settings = Settings {
            output_dir: PathBuf::from("/tmp/videos"),
            quality: Some(1080),
            audio_only: true,
            use_browser_cookies: true,
            max_retries: 5,
        };

        save(&path, &settings).expect("settings should save");
        let loaded = load(&path).expect("settings should load");
        std::fs::remove_file(&path).expect("temporary settings should be removed");

        assert_eq!(loaded.output_dir, settings.output_dir);
        assert_eq!(loaded.quality, settings.quality);
        assert!(loaded.audio_only);
        assert!(loaded.use_browser_cookies);
        assert_eq!(loaded.max_retries, 5);
    }

    #[test]
    fn rejects_empty_output_directory() {
        let settings = Settings {
            output_dir: PathBuf::new(),
            ..Settings::default()
        };

        assert!(settings.validate().is_err());
    }
}
