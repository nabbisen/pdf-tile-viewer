//! Versioned settings schema (RFC 008), privacy defaults (RFC 016 §7),
//! and UI locale preference (RFC 017 §7).
//!
//! Serialization lives here; file I/O lives in `app_services`.

use serde::{Deserialize, Serialize};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettingsV1 {
    pub schema_version: u32,
    pub ui: UiSettings,
    pub viewer: ViewerSettings,
    pub window: WindowSettings,
    pub privacy: PrivacySettings,
    pub advanced: AdvancedSettings,
}

impl Default for AppSettingsV1 {
    fn default() -> Self {
        AppSettingsV1 {
            schema_version: SETTINGS_SCHEMA_VERSION,
            ui: UiSettings::default(),
            viewer: ViewerSettings::default(),
            window: WindowSettings::default(),
            privacy: PrivacySettings::default(),
            advanced: AdvancedSettings::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UiSettings {
    /// BCP 47-ish tag such as "en" or "ja". `None` follows the system locale.
    pub locale: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewerSettings {
    pub default_scale: f32,
    pub pages_per_row: PagesPerRowPreference,
    pub show_page_numbers: bool,
    pub zoom_overlay_scale: f32,
    pub zoom_overlay_alpha: Option<f32>,
}

impl Default for ViewerSettings {
    fn default() -> Self {
        ViewerSettings {
            default_scale: 1.0,
            pages_per_row: PagesPerRowPreference::Auto,
            show_page_numbers: false,
            zoom_overlay_scale: 2.7,
            zoom_overlay_alpha: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "mode", content = "value", rename_all = "snake_case")]
pub enum PagesPerRowPreference {
    #[default]
    Auto,
    Fixed(u16),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WindowSettings {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub maximized: bool,
}

/// Privacy is safe by default (RFC 016 §7): no full paths in the title,
/// no persisted recent files.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PrivacySettings {
    pub show_full_path_in_title: bool,
    pub persist_recent_files: bool,
    pub include_full_paths_in_diagnostics: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AdvancedSettings {
    pub render_cache_budget_mb: Option<u32>,
    pub diagnostics_enabled: bool,
}

impl AppSettingsV1 {
    /// Clamp out-of-range values instead of failing (RFC 008 §11:
    /// "Invalid settings file falls back safely").
    pub fn sanitized(mut self) -> Self {
        self.schema_version = SETTINGS_SCHEMA_VERSION;
        self.viewer.default_scale = self.viewer.default_scale.clamp(0.2, 5.0);
        self.viewer.zoom_overlay_scale = self.viewer.zoom_overlay_scale.clamp(0.2, 8.0);
        if let Some(alpha) = self.viewer.zoom_overlay_alpha {
            self.viewer.zoom_overlay_alpha = Some(alpha.clamp(0.0, 1.0));
        }
        if let PagesPerRowPreference::Fixed(n) = self.viewer.pages_per_row {
            self.viewer.pages_per_row = PagesPerRowPreference::Fixed(n.clamp(1, 24));
        }
        self
    }
}

#[cfg(test)]
mod tests;
