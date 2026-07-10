use domain::settings::AppSettingsV1;

use crate::settings_service::{SettingsLoadOutcome, SettingsStore};

fn temp_settings_path(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ptv-settings-{}-{tag}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("settings.json")
}

#[test]
fn missing_file_yields_defaults() {
    let store = SettingsStore::at_path(temp_settings_path("missing"));
    let (settings, outcome) = store.load();
    assert_eq!(settings, AppSettingsV1::default());
    assert_eq!(outcome, SettingsLoadOutcome::DefaultsMissingFile);
}

#[test]
fn save_then_load_round_trips() {
    let store = SettingsStore::at_path(temp_settings_path("roundtrip"));
    let mut settings = AppSettingsV1::default();
    settings.viewer.default_scale = 1.5;
    settings.ui.locale = Some("ja".to_string());
    store.save(&settings).unwrap();

    let (loaded, outcome) = store.load();
    assert_eq!(outcome, SettingsLoadOutcome::Loaded);
    assert_eq!(loaded.viewer.default_scale, 1.5);
    assert_eq!(loaded.ui.locale.as_deref(), Some("ja"));
}

#[test]
fn corrupt_file_is_backed_up_and_replaced_by_defaults() {
    // RFC 008 §11: invalid settings must never block startup; the broken
    // file is preserved as a backup for inspection.
    let path = temp_settings_path("corrupt");
    std::fs::write(&path, "{ this is not json").unwrap();
    let store = SettingsStore::at_path(path.clone());

    let (settings, outcome) = store.load();
    assert_eq!(settings, AppSettingsV1::default());
    assert_eq!(outcome, SettingsLoadOutcome::DefaultsAfterBackup);
    assert!(path.with_extension("json.bak").is_file());
    assert!(!path.exists());
}

#[test]
fn out_of_range_values_are_sanitized_on_load() {
    let path = temp_settings_path("sanitize");
    std::fs::write(
        &path,
        r#"{ "schema_version": 1, "viewer": { "default_scale": 99.0 } }"#,
    )
    .unwrap();
    let store = SettingsStore::at_path(path);
    let (settings, outcome) = store.load();
    assert_eq!(outcome, SettingsLoadOutcome::Loaded);
    assert_eq!(settings.viewer.default_scale, 5.0); // clamped to ViewerScale MAX
}

#[test]
fn unknown_and_missing_fields_fall_back_per_field() {
    // serde(default) everywhere: forward/backward compatible settings.
    let path = temp_settings_path("partial");
    std::fs::write(&path, r#"{ "schema_version": 1, "future_field": true }"#).unwrap();
    let store = SettingsStore::at_path(path);
    let (settings, outcome) = store.load();
    assert_eq!(outcome, SettingsLoadOutcome::Loaded);
    assert_eq!(settings, AppSettingsV1::default());
}
