use packaging::pdfium_bundle::PdfiumLoadMode;

use crate::engine_boot::{loader_config, resource_root_for_exe};

#[test]
fn production_config_never_allows_dev_fallback() {
    // RFC 016 §6: production loads only from the bundled resource root.
    let config = loader_config(
        PdfiumLoadMode::ProductionBundled,
        "/opt/app/resources".into(),
    );
    assert!(!config.allow_dev_fallback);
    assert_eq!(config.explicit_dev_path, None);
}

#[test]
fn development_config_allows_explicit_dev_path() {
    let config = loader_config(PdfiumLoadMode::Development, "resources".into());
    assert!(config.allow_dev_fallback);
}

#[test]
fn plain_executable_uses_adjacent_resources_dir() {
    let root = resource_root_for_exe("/opt/pdf-tile-viewer/pdf-tile-viewer".as_ref()).unwrap();
    assert_eq!(
        root,
        std::path::PathBuf::from("/opt/pdf-tile-viewer/resources")
    );
}

#[test]
fn archive_bin_layout_uses_package_root_resources_dir() {
    let root = resource_root_for_exe("/opt/pdf-tile-viewer/bin/pdf-tile-viewer".as_ref()).unwrap();
    assert_eq!(
        root,
        std::path::PathBuf::from("/opt/pdf-tile-viewer/resources")
    );
}

#[test]
fn macos_app_bundle_uses_contents_resources_dir() {
    let root = resource_root_for_exe(
        "/Applications/PDF Tile Viewer.app/Contents/MacOS/pdf-tile-viewer".as_ref(),
    )
    .unwrap();
    assert_eq!(
        root,
        std::path::PathBuf::from("/Applications/PDF Tile Viewer.app/Contents/Resources")
    );
}
