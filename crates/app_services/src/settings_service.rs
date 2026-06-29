//! Settings persistence (RFC 008 §10–§11).
//!
//! JSON file under the platform config dir. Failure policy: a missing file
//! yields defaults; a corrupt file is backed up (`settings.json.bak`) and
//! replaced by sanitized defaults — the app never refuses to start over
//! settings. All loaded values pass through `AppSettingsV1::sanitized()`.

use std::path::{Path, PathBuf};

use domain::settings::AppSettingsV1;

/// How the settings were obtained, for diagnostics/toasts (RFC 008 §11).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsLoadOutcome {
    Loaded,
    DefaultsMissingFile,
    DefaultsAfterBackup,
    DefaultsNoConfigDir,
}

#[derive(Clone, Debug)]
pub struct SettingsStore {
    path: Option<PathBuf>,
}

impl SettingsStore {
    /// Store at the platform default location
    /// (`<config_dir>/pdf-tile-viewer/settings.json`).
    pub fn at_default_location() -> Self {
        SettingsStore {
            path: packaging::app_dirs::settings_file(),
        }
    }

    /// Store at an explicit path (tests, portable mode).
    pub fn at_path(path: PathBuf) -> Self {
        SettingsStore { path: Some(path) }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Load settings, never failing (RFC 008 §11).
    pub fn load(&self) -> (AppSettingsV1, SettingsLoadOutcome) {
        let Some(path) = &self.path else {
            return (
                AppSettingsV1::default(),
                SettingsLoadOutcome::DefaultsNoConfigDir,
            );
        };
        let raw = match std::fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return (
                    AppSettingsV1::default(),
                    SettingsLoadOutcome::DefaultsMissingFile,
                );
            }
            Err(_) => {
                return self.backup_and_default(path);
            }
        };
        match serde_json::from_str::<AppSettingsV1>(&raw) {
            Ok(parsed) => (parsed.sanitized(), SettingsLoadOutcome::Loaded),
            Err(_) => self.backup_and_default(path),
        }
    }

    fn backup_and_default(&self, path: &Path) -> (AppSettingsV1, SettingsLoadOutcome) {
        let backup = path.with_extension("json.bak");
        let _ = std::fs::rename(path, &backup);
        (
            AppSettingsV1::default(),
            SettingsLoadOutcome::DefaultsAfterBackup,
        )
    }

    /// Persist settings atomically (temp file + rename), creating the
    /// config directory on demand.
    pub fn save(&self, settings: &AppSettingsV1) -> std::io::Result<()> {
        let Some(path) = &self.path else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no config directory available",
            ));
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(&settings.clone().sanitized())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}
