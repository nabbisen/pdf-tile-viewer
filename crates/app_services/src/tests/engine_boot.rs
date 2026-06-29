use packaging::pdfium_bundle::PdfiumLoadMode;

use crate::engine_boot::loader_config;

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
