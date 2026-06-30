use crate::settings::*;

#[test]
fn defaults_match_rfc_008_and_rfc_016_privacy_defaults() {
    let s = AppSettingsV1::default();
    assert_eq!(s.schema_version, SETTINGS_SCHEMA_VERSION);
    assert_eq!(s.viewer.default_scale, 1.0);
    assert_eq!(s.viewer.pages_per_row, PagesPerRowPreference::Auto);
    assert!(!s.viewer.show_page_numbers);
    assert_eq!(s.viewer.zoom_overlay_scale, 2.7);
    // RFC 016 §7: safe-by-default privacy.
    assert!(!s.privacy.show_full_path_in_title);
    assert!(!s.privacy.persist_recent_files);
    // RFC 017 §7: locale follows system by default.
    assert!(s.ui.locale.is_none());
}

#[test]
fn unknown_or_missing_fields_fall_back_safely() {
    // RFC 008 §11: invalid settings file falls back safely.
    let parsed: AppSettingsV1 = serde_json::from_str(r#"{"schema_version": 1}"#).unwrap();
    assert_eq!(parsed, AppSettingsV1::default());
}

#[test]
fn out_of_range_values_are_clamped_by_sanitize() {
    let mut s = AppSettingsV1::default();
    s.viewer.default_scale = 99.0;
    s.viewer.pages_per_row = PagesPerRowPreference::Fixed(200);
    s.viewer.zoom_overlay_alpha = Some(7.0);
    let s = s.sanitized();
    assert_eq!(s.viewer.default_scale, 5.0);
    assert_eq!(s.viewer.pages_per_row, PagesPerRowPreference::Fixed(24));
    assert_eq!(s.viewer.zoom_overlay_alpha, Some(1.0));
}

#[test]
fn settings_round_trip_through_json() {
    let s = AppSettingsV1::default();
    let json = serde_json::to_string_pretty(&s).unwrap();
    let back: AppSettingsV1 = serde_json::from_str(&json).unwrap();
    assert_eq!(s, back);
}
