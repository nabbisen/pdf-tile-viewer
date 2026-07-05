//! Serialized engine worker (RFC 004 §8, ADR §3.4).
//!
//! PDFium is not thread-safe, so all engine calls are executed on one
//! dedicated OS thread that owns the [`PdfEngine`]. UI/services interact
//! only through [`EngineHandle`], which sends commands over a channel and
//! returns runtime-agnostic `oneshot` futures (no Tokio dependency, works
//! under the Dioxus desktop runtime and plain `block_on`-style tests).
//!
//! Native PDFium handles never cross this boundary; every reply is built
//! from `domain` types and carries `document_id` + `generation` so callers
//! can discard stale results (Appendix A §8).

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::JoinHandle;

use domain::document::{DocumentError, DocumentId, DocumentSession};
use domain::render::{RenderError, RenderPageRequest, RenderedPageImage};
use domain::search::{SearchError, SearchHighlightSet, SearchRequest, SearchResultSet};
use domain::text::{PageTextLayer, TextLayerError, TextLayerRequest};
use futures_channel::oneshot;

use crate::engine::PdfEngine;
use crate::loader::{self, PdfiumLoadError, PdfiumLoadReport};
use crate::render;
use crate::search;
use crate::text_layer;

/// Commands executed sequentially on the engine thread.
enum Command {
    OpenDocument {
        path: PathBuf,
        reply: oneshot::Sender<Result<DocumentSession, DocumentError>>,
    },
    CloseDocument {
        id: DocumentId,
        reply: oneshot::Sender<bool>,
    },
    RenderPage {
        request: RenderPageRequest,
        reply: oneshot::Sender<Result<RenderedPageImage, RenderError>>,
    },
    SearchDocument {
        request: SearchRequest,
        reply: oneshot::Sender<Result<SearchResultSet, SearchError>>,
    },
    SearchDocumentWithHighlights {
        request: SearchRequest,
        reply: oneshot::Sender<Result<(SearchResultSet, SearchHighlightSet), SearchError>>,
    },
    ExtractPageTextLayer {
        request: TextLayerRequest,
        reply: oneshot::Sender<Result<PageTextLayer, TextLayerError>>,
    },
    Shutdown,
}

/// Error returned when the engine thread is gone (channel closed).
///
/// Callers map this onto their own `EngineUnavailable`-style variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EngineGone;

impl std::fmt::Display for EngineGone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PDF engine worker is not available")
    }
}

impl std::error::Error for EngineGone {}

/// Cloneable handle to the serialized engine worker.
///
/// Dropping the last clone does not stop the thread; call
/// [`EngineHandle::shutdown`] (or let process exit reap it) — the vertical
/// slice keeps one handle alive for the app lifetime (RFC 005).
#[derive(Clone)]
pub struct EngineHandle {
    sender: mpsc::Sender<Command>,
    pub load_report: PdfiumLoadReport,
    /// Identity token — two clones of the same handle share this Arc.
    identity: std::sync::Arc<()>,
}

impl PartialEq for EngineHandle {
    /// Two handles are equal when they share the same identity Arc.
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.identity, &other.identity)
    }
}

impl EngineHandle {
    /// Spawn the engine thread, binding PDFium from `library_dir`.
    ///
    /// Binding happens on the worker thread itself so that every PDFium
    /// call — including `FPDF_InitLibrary` via the first bind — stays on
    /// one thread.
    pub fn spawn(library_dir: PathBuf) -> Result<(Self, EngineThread), PdfiumLoadError> {
        let (sender, receiver) = mpsc::channel::<Command>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<PdfiumLoadReport, PdfiumLoadError>>();

        let join = std::thread::Builder::new()
            .name("pdf-engine".to_string())
            .spawn(move || {
                let (pdfium, report) = match loader::bind_from_dir(&library_dir) {
                    Ok(ok) => ok,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                let _ = ready_tx.send(Ok(report));
                run_loop(PdfEngine::new(pdfium), receiver);
            })
            .expect("failed to spawn pdf-engine thread");

        let load_report = ready_rx
            .recv()
            .map_err(|_| PdfiumLoadError::BindFailed("engine thread died".to_string()))??;

        Ok((
            EngineHandle {
                sender,
                load_report,
                identity: std::sync::Arc::new(()),
            },
            EngineThread { join },
        ))
    }

    pub fn open_document(
        &self,
        path: PathBuf,
    ) -> impl std::future::Future<
        Output = Result<Result<DocumentSession, DocumentError>, EngineGone>,
    > + use<> {
        let (reply, rx) = oneshot::channel();
        let sent = self.sender.send(Command::OpenDocument { path, reply });
        async move {
            sent.map_err(|_| EngineGone)?;
            rx.await.map_err(|_| EngineGone)
        }
    }

    pub fn close_document(
        &self,
        id: DocumentId,
    ) -> impl std::future::Future<Output = Result<bool, EngineGone>> + use<> {
        let (reply, rx) = oneshot::channel();
        let sent = self.sender.send(Command::CloseDocument { id, reply });
        async move {
            sent.map_err(|_| EngineGone)?;
            rx.await.map_err(|_| EngineGone)
        }
    }

    pub fn render_page(
        &self,
        request: RenderPageRequest,
    ) -> impl std::future::Future<
        Output = Result<Result<RenderedPageImage, RenderError>, EngineGone>,
    > + use<> {
        let (reply, rx) = oneshot::channel();
        let sent = self.sender.send(Command::RenderPage { request, reply });
        async move {
            sent.map_err(|_| EngineGone)?;
            rx.await.map_err(|_| EngineGone)
        }
    }

    pub fn search_document(
        &self,
        request: SearchRequest,
    ) -> impl std::future::Future<Output = Result<Result<SearchResultSet, SearchError>, EngineGone>>
    + use<> {
        let (reply, rx) = oneshot::channel();
        let sent = self.sender.send(Command::SearchDocument { request, reply });
        async move {
            sent.map_err(|_| EngineGone)?;
            rx.await.map_err(|_| EngineGone)
        }
    }

    pub fn search_document_with_highlights(
        &self,
        request: SearchRequest,
    ) -> impl std::future::Future<
        Output = Result<Result<(SearchResultSet, SearchHighlightSet), SearchError>, EngineGone>,
    > + use<> {
        let (reply, rx) = oneshot::channel();
        let sent = self
            .sender
            .send(Command::SearchDocumentWithHighlights { request, reply });
        async move {
            sent.map_err(|_| EngineGone)?;
            rx.await.map_err(|_| EngineGone)
        }
    }

    pub fn extract_page_text_layer(
        &self,
        request: TextLayerRequest,
    ) -> impl std::future::Future<Output = Result<Result<PageTextLayer, TextLayerError>, EngineGone>>
    + use<> {
        let (reply, rx) = oneshot::channel();
        let sent = self
            .sender
            .send(Command::ExtractPageTextLayer { request, reply });
        async move {
            sent.map_err(|_| EngineGone)?;
            rx.await.map_err(|_| EngineGone)
        }
    }

    /// Ask the worker to exit after draining queued commands.
    pub fn shutdown(&self) {
        let _ = self.sender.send(Command::Shutdown);
    }
}

/// Owner of the engine thread; join on app shutdown for clean teardown.
pub struct EngineThread {
    join: JoinHandle<()>,
}

impl EngineThread {
    pub fn join(self) {
        let _ = self.join.join();
    }
}

fn run_loop(mut engine: PdfEngine, receiver: mpsc::Receiver<Command>) {
    while let Ok(command) = receiver.recv() {
        match command {
            Command::OpenDocument { path, reply } => {
                let _ = reply.send(engine.open_document(&path));
            }
            Command::CloseDocument { id, reply } => {
                let _ = reply.send(engine.close_document(id));
            }
            Command::RenderPage { request, reply } => {
                let _ = reply.send(render::render_page(&engine, &request));
            }
            Command::SearchDocument { request, reply } => {
                let _ = reply.send(search::search_document(&engine, &request));
            }
            Command::SearchDocumentWithHighlights { request, reply } => {
                let _ = reply.send(search::search_document_with_highlights(&engine, &request));
            }
            Command::ExtractPageTextLayer { request, reply } => {
                let _ = reply.send(text_layer::extract_page_text_layer(&engine, &request));
            }
            Command::Shutdown => break,
        }
    }
    // `engine` drops here: sessions first, then the Pdfium bindings.
}
