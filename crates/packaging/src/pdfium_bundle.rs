//! Bundled PDFium resource layout and resolution (RFC 003 §7–§9).
//!
//! Production loads only from the app-controlled resource root; it never
//! searches the current working directory or `PATH` (RFC 016 §6).

use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfiumLoadMode {
    ProductionBundled,
    Development,
    Test,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfiumLoadSource {
    BundledResource,
    ExtractedAppCache,
    ExplicitDevelopmentPath,
    TestFixturePath,
}

#[derive(Clone, Debug)]
pub struct PdfiumLoaderConfig {
    pub mode: PdfiumLoadMode,
    pub allow_dev_fallback: bool,
    pub bundled_resource_root: PathBuf,
    pub explicit_dev_path: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct PdfiumResolution {
    pub library_dir: PathBuf,
    pub source: PdfiumLoadSource,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PdfiumResolveError {
    NotFound { searched: Vec<PathBuf> },
    DevFallbackDisabled,
}

/// Platform directory segment under `resources/pdfium/` (RFC 003 §7).
pub fn platform_segment() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows-x86_64"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "macos-aarch64"
        } else {
            "macos-x86_64"
        }
    } else {
        "linux-x86_64"
    }
}

/// Platform library file name.
pub fn library_file_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    }
}

/// `<root>/pdfium/<platform>/` — the directory pdfium-render binds against.
pub fn bundled_library_dir(resource_root: &Path) -> PathBuf {
    resource_root.join("pdfium").join(platform_segment())
}

/// Resolve the PDFium library directory according to RFC 003 §6.
pub fn resolve(config: &PdfiumLoaderConfig) -> Result<PdfiumResolution, PdfiumResolveError> {
    let mut searched = Vec::new();

    if matches!(
        config.mode,
        PdfiumLoadMode::Development | PdfiumLoadMode::Test
    ) {
        if let Some(dev) = &config.explicit_dev_path {
            if !config.allow_dev_fallback {
                return Err(PdfiumResolveError::DevFallbackDisabled);
            }
            let lib = dev.join(library_file_name());
            if lib.is_file() {
                return Ok(PdfiumResolution {
                    library_dir: dev.clone(),
                    source: match config.mode {
                        PdfiumLoadMode::Test => PdfiumLoadSource::TestFixturePath,
                        _ => PdfiumLoadSource::ExplicitDevelopmentPath,
                    },
                });
            }
            searched.push(lib);
        }
    }

    let bundled = bundled_library_dir(&config.bundled_resource_root);
    let lib = bundled.join(library_file_name());
    if lib.is_file() {
        return Ok(PdfiumResolution {
            library_dir: bundled,
            source: PdfiumLoadSource::BundledResource,
        });
    }
    searched.push(lib);

    Err(PdfiumResolveError::NotFound { searched })
}
