//! Platform integration (RFC 002 §6, §9).
//!
//! - Native "open PDF" picker behind the `native-dialogs` feature (rfd).
//! - Reveal-in-file-manager via the platform's standard opener; we reveal
//!   the *containing directory*, never execute the document itself.

use std::path::Path;
#[cfg(feature = "native-dialogs")]
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum PlatformError {
    NoParentDirectory,
    LaunchFailed(String),
}

/// Open the file manager at the document's containing directory.
pub fn reveal_in_file_manager(path: &Path) -> Result<(), PlatformError> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or(PlatformError::NoParentDirectory)?;
    open_directory(dir)
}

fn open_directory(dir: &Path) -> Result<(), PlatformError> {
    let (program, args): (&str, Vec<&std::ffi::OsStr>) = if cfg!(target_os = "windows") {
        ("explorer", vec![dir.as_os_str()])
    } else if cfg!(target_os = "macos") {
        ("open", vec![dir.as_os_str()])
    } else {
        ("xdg-open", vec![dir.as_os_str()])
    };
    std::process::Command::new(program)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|e| PlatformError::LaunchFailed(e.to_string()))
}

/// Async native file picker filtered to PDF (RFC 002 §6).
#[cfg(feature = "native-dialogs")]
pub async fn pick_pdf_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("PDF documents", &["pdf"])
        .set_title("Open PDF")
        .pick_file()
        .await
        .map(|handle| handle.path().to_path_buf())
}
