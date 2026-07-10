//! Packaged release artifact smoke helper (RFC 018).
//!
//! This binary validates an extracted release archive without launching the GUI:
//! it checks the expected archive layout, derives the production resource root
//! from `bin/<app>`, resolves bundled PDFium in production mode, and binds it.

use std::path::Path;

use app_services::engine_boot::{boot_engine, loader_config, resource_root_for_exe};
use packaging::pdfium_bundle::{PdfiumLoadMode, PdfiumLoadSource, bundled_library_dir};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: package_artifact_smoke <extracted-artifact-root>");
        std::process::exit(2);
    }

    if let Err(err) = run(Path::new(&args[1])) {
        eprintln!("package artifact smoke failed: {err}");
        std::process::exit(1);
    }
}

fn run(root: &Path) -> Result<(), String> {
    if !root.is_dir() {
        return Err(format!(
            "artifact root is not a directory: {}",
            root.display()
        ));
    }

    let bin = root.join("bin").join(app_binary_name());
    require_file(&bin)?;
    require_file(&root.join("LICENSE"))?;
    require_file(&root.join("NOTICE"))?;
    require_file(&root.join("README.md"))?;
    if !root.join("notes.txt").is_file() && !root.join("CHANGELOG.md").is_file() {
        return Err("expected notes.txt or CHANGELOG.md in artifact root".to_string());
    }

    let resource_root = resource_root_for_exe(&bin)
        .ok_or_else(|| format!("could not derive resource root from {}", bin.display()))?;
    let expected_resource_root = root.join("resources");
    if resource_root != expected_resource_root {
        return Err(format!(
            "resource root mismatch: got {}, expected {}",
            resource_root.display(),
            expected_resource_root.display()
        ));
    }

    let library_dir = bundled_library_dir(&resource_root);
    require_file(&library_dir.join(packaging::pdfium_bundle::library_file_name()))?;

    let config = loader_config(PdfiumLoadMode::ProductionBundled, resource_root);
    let resolution = packaging::pdfium_bundle::resolve(&config)
        .map_err(|err| format!("production PDFium resolve failed: {err:?}"))?;
    if resolution.source != PdfiumLoadSource::BundledResource {
        return Err(format!("unexpected PDFium source: {:?}", resolution.source));
    }

    let (engine, thread) = boot_engine(&config).map_err(|err| err.to_string())?;
    engine.shutdown();
    thread.join();

    println!(
        "package artifact smoke passed: {}",
        root.file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| root.display().to_string().into())
    );
    Ok(())
}

fn require_file(path: &Path) -> Result<(), String> {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("required file is missing: {}", path.display()))
    }
}

fn app_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "pdf-tile-viewer.exe"
    } else {
        "pdf-tile-viewer"
    }
}
