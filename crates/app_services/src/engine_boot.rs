//! Engine bootstrap (RFC 003 + RFC 004).
//!
//! Glues `packaging::pdfium_bundle::resolve` (policy: where PDFium may be
//! loaded from) to `pdf_engine::worker::EngineHandle::spawn` (mechanism:
//! bind + dedicated thread).

use std::path::PathBuf;

use packaging::pdfium_bundle::{PdfiumLoadMode, PdfiumLoaderConfig, PdfiumResolveError, resolve};
use pdf_engine::loader::PdfiumLoadError;
use pdf_engine::worker::{EngineHandle, EngineThread};

/// Environment variable allowing developers/tests to point at a local
/// PDFium build (e.g. `ci/.pdfium`). Ignored in production mode by
/// `packaging::pdfium_bundle::resolve` (RFC 016 §6).
pub const PDFIUM_DIR_ENV: &str = "PDF_TILE_VIEWER_PDFIUM_DIR";

#[derive(Clone, Debug)]
pub enum EngineBootError {
    Resolve(PdfiumResolveError),
    Bind(String),
}

impl std::fmt::Display for EngineBootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineBootError::Resolve(PdfiumResolveError::NotFound { searched }) => {
                write!(f, "PDFium library not found; searched: {searched:?}")
            }
            EngineBootError::Resolve(PdfiumResolveError::DevFallbackDisabled) => {
                f.write_str("development PDFium path is disabled in this mode")
            }
            EngineBootError::Bind(msg) => write!(f, "failed to bind PDFium: {msg}"),
        }
    }
}

/// Build the loader config for the current process.
///
/// `resource_root` is the app-controlled resources directory (next to the
/// executable in packaged builds). In development/test mode the
/// `PDF_TILE_VIEWER_PDFIUM_DIR` env var is honoured as an explicit path.
pub fn loader_config(mode: PdfiumLoadMode, resource_root: PathBuf) -> PdfiumLoaderConfig {
    let dev = matches!(mode, PdfiumLoadMode::Development | PdfiumLoadMode::Test);
    PdfiumLoaderConfig {
        mode,
        allow_dev_fallback: dev,
        bundled_resource_root: resource_root,
        explicit_dev_path: if dev {
            std::env::var_os(PDFIUM_DIR_ENV).map(PathBuf::from)
        } else {
            None
        },
    }
}

/// Resolve PDFium per policy and spawn the serialized engine worker.
pub fn boot_engine(
    config: &PdfiumLoaderConfig,
) -> Result<(EngineHandle, EngineThread), EngineBootError> {
    let resolution = resolve(config).map_err(EngineBootError::Resolve)?;
    EngineHandle::spawn(resolution.library_dir).map_err(|e| match e {
        PdfiumLoadError::LibraryNotFound { searched } => {
            EngineBootError::Resolve(PdfiumResolveError::NotFound { searched })
        }
        PdfiumLoadError::BindFailed(msg) => EngineBootError::Bind(msg),
    })
}

/// Resource root for the running app (RFC 003 §7).
pub fn default_resource_root() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| resource_root_for_exe(&exe))
        .unwrap_or_else(|| PathBuf::from("resources"))
}

pub fn resource_root_for_exe(exe: &std::path::Path) -> Option<PathBuf> {
    let exe_dir = exe.parent()?;

    if exe_dir.file_name().is_some_and(|name| name == "MacOS")
        && let Some(contents_dir) = exe_dir
            .parent()
            .filter(|dir| dir.file_name().is_some_and(|name| name == "Contents"))
    {
        return Some(contents_dir.join("Resources"));
    }

    if exe_dir.file_name().is_some_and(|name| name == "bin")
        && let Some(package_root) = exe_dir.parent()
    {
        return Some(package_root.join("resources"));
    }

    Some(exe_dir.join("resources"))
}

#[cfg(test)]
mod tests;
