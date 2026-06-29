use std::fs;
use std::path::PathBuf;

use crate::pdfium_bundle::*;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ptv-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn bundled_layout_matches_rfc_003_resource_structure() {
    let root = PathBuf::from("/opt/app/resources");
    let dir = bundled_library_dir(&root);
    // resources/pdfium/<platform>/ (RFC 003 §7)
    assert!(dir.starts_with(root.join("pdfium")));
    assert!(dir.ends_with(platform_segment()));
}

#[test]
fn production_mode_never_uses_dev_paths() {
    // RFC 016 §6: production loads only from the app-controlled root.
    let dev = temp_dir("devlib");
    fs::write(dev.join(library_file_name()), b"fake").unwrap();
    let config = PdfiumLoaderConfig {
        mode: PdfiumLoadMode::ProductionBundled,
        allow_dev_fallback: true, // even if misconfigured to true
        bundled_resource_root: temp_dir("empty-root"),
        explicit_dev_path: Some(dev),
    };
    let err = resolve(&config).unwrap_err();
    assert!(matches!(err, PdfiumResolveError::NotFound { .. }));
}

#[test]
fn development_mode_uses_explicit_dev_path_when_allowed() {
    let dev = temp_dir("devlib2");
    fs::write(dev.join(library_file_name()), b"fake").unwrap();
    let config = PdfiumLoaderConfig {
        mode: PdfiumLoadMode::Development,
        allow_dev_fallback: true,
        bundled_resource_root: temp_dir("empty-root2"),
        explicit_dev_path: Some(dev.clone()),
    };
    let res = resolve(&config).unwrap();
    assert_eq!(res.library_dir, dev);
    assert_eq!(res.source, PdfiumLoadSource::ExplicitDevelopmentPath);
}

#[test]
fn dev_fallback_disabled_is_an_explicit_error() {
    let config = PdfiumLoaderConfig {
        mode: PdfiumLoadMode::Development,
        allow_dev_fallback: false,
        bundled_resource_root: temp_dir("empty-root3"),
        explicit_dev_path: Some(temp_dir("devlib3")),
    };
    assert_eq!(
        resolve(&config).unwrap_err(),
        PdfiumResolveError::DevFallbackDisabled
    );
}

#[test]
fn missing_library_reports_searched_paths_for_diagnostics() {
    // RFC 003 §9: failed load must produce a clear, recoverable error.
    let config = PdfiumLoaderConfig {
        mode: PdfiumLoadMode::ProductionBundled,
        allow_dev_fallback: false,
        bundled_resource_root: temp_dir("empty-root4"),
        explicit_dev_path: None,
    };
    match resolve(&config).unwrap_err() {
        PdfiumResolveError::NotFound { searched } => assert!(!searched.is_empty()),
        other => panic!("unexpected: {other:?}"),
    }
}
