//! Engine smoke tests (RFC 015 §5): open → geometry → render → search,
//! end-to-end through the serialized worker, against generated fixtures.
//!
//! Requires a real PDFium dynamic library. Set `PDF_TILE_VIEWER_PDFIUM_DIR`
//! to its directory (e.g. `ci/.pdfium`, populated by `ci/fetch-pdfium.sh`);
//! without it the tests skip with a notice instead of failing, so plain
//! `cargo test` stays green on machines without PDFium (RFC 015 §7).
//!
//! Implementation note: PDFium calls `FPDF_InitLibraryWithConfig` once per
//! process. All tests share a single `EngineHandle` via a `LazyLock`, and
//! the associated `EngineThread` is forgotten so `FPDF_DestroyLibrary` is
//! never called mid-process.

use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex, MutexGuard};

use pdf_engine::worker::EngineHandle;

const PDFIUM_DIR_ENV: &str = "PDF_TILE_VIEWER_PDFIUM_DIR";

fn pdfium_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(PDFIUM_DIR_ENV) {
        return Some(PathBuf::from(dir));
    }
    let repo_default = workspace_root().join("ci").join(".pdfium");
    repo_default.is_dir().then_some(repo_default)
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crate lives at <root>/crates/pdf_engine")
        .to_path_buf()
}

fn fixture(name: &str) -> PathBuf {
    workspace_root().join("fixtures").join(name)
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    futures_executor::block_on(future)
}

/// Shared PDFium engine for the whole test process.
///
/// PDFium may only be initialized once per OS process; the EngineThread is
/// forgotten so FPDF_DestroyLibrary is never called between tests.
static ENGINE: LazyLock<Option<EngineHandle>> = LazyLock::new(|| {
    let dir = pdfium_dir()?;
    let (handle, thread) = EngineHandle::spawn(dir).expect("PDFium bind failed");
    std::mem::forget(thread);
    Some(handle)
});

static ENGINE_LOCK: Mutex<()> = Mutex::new(());

struct LockedEngine {
    handle: &'static EngineHandle,
    _guard: MutexGuard<'static, ()>,
}

impl Deref for LockedEngine {
    type Target = EngineHandle;

    fn deref(&self) -> &Self::Target {
        self.handle
    }
}

macro_rules! require_engine {
    () => {{
        let guard = ENGINE_LOCK.lock().expect("engine smoke lock poisoned");
        match ENGINE.as_ref() {
            Some(h) => LockedEngine {
                handle: h,
                _guard: guard,
            },
            None => {
                eprintln!(
                    "SKIP: set {PDFIUM_DIR_ENV} (or run ci/fetch-pdfium.sh) \
                     to run engine smoke tests"
                );
                return;
            }
        }
    }};
}

mod document;
mod encryption;
mod lifecycle;
mod navigation;
mod render;
mod search;
mod text;
