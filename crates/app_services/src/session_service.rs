//! Owner of the open document session (RFC 035 § 5.1).
//!
//! One place knows everything that belongs to a session: the three caches and the
//! native document. [`SessionService::close`] closes all of it, so the UI cannot forget a
//! part. The product keeps exactly one document open at a time; [`SessionService::opened`]
//! registers it.

use std::sync::{Arc, Mutex};

use domain::document::{DocumentGeneration, DocumentId, DocumentSession};
use pdf_engine::worker::EngineHandle;

use crate::navigation_service::NavigationService;
use crate::render_service::RenderService;
use crate::text_service::TextLayerService;

type SessionKey = (DocumentId, DocumentGeneration);

#[derive(Clone)]
pub struct SessionService {
    engine: EngineHandle,
    render: RenderService,
    text: TextLayerService,
    navigation: NavigationService,
    active: Arc<Mutex<Option<SessionKey>>>,
}

impl SessionService {
    pub fn new(
        engine: EngineHandle,
        render: RenderService,
        text: TextLayerService,
        navigation: NavigationService,
    ) -> Self {
        SessionService {
            engine,
            render,
            text,
            navigation,
            active: Arc::new(Mutex::new(None)),
        }
    }

    /// Registers the session that is now open and makes its generation the render
    /// cache's active generation.
    ///
    /// A second `opened` while another session is still active is a logic error: it
    /// fails a debug assertion. Release builds close the stale session instead of
    /// leaking its native document.
    pub fn opened(&self, session: &DocumentSession) {
        let key = (session.id, session.generation);
        let previous = self.active.lock().unwrap().replace(key);
        debug_assert!(
            previous.is_none_or(|previous| previous == key),
            "a document session was opened while another one is still active"
        );
        if let Some((id, generation)) = previous.filter(|previous| *previous != key) {
            self.close_caches_and_engine(id, generation);
        }
        self.render.set_active_generation(Some(session.generation));
    }

    /// Closes a session: later results for it are refused by every cache, its cache
    /// entries are evicted, then the engine is told to close the native document.
    ///
    /// Synchronous and fire-and-forget (safe to call from a drop hook): the engine
    /// command is sent when this method runs and executed in order by the worker; the
    /// returned future is dropped. Closing an unknown or already closed session is a
    /// no-op.
    pub fn close(&self, id: DocumentId, generation: DocumentGeneration) {
        {
            let mut active = self.active.lock().unwrap();
            if *active == Some((id, generation)) {
                *active = None;
                self.render.set_active_generation(None);
            }
        }
        self.close_caches_and_engine(id, generation);
    }

    fn close_caches_and_engine(&self, id: DocumentId, generation: DocumentGeneration) {
        self.render.evict_session(id, generation);
        self.text.evict_session(id, generation);
        self.navigation.evict_session(id, generation);
        // Dropping the future is intentional: the command is already sent.
        drop(self.engine.close_document(id));
    }

    /// The session registered by [`SessionService::opened`], if any.
    pub fn active_session(&self) -> Option<(DocumentId, DocumentGeneration)> {
        *self.active.lock().unwrap()
    }
}

#[cfg(test)]
mod tests;
