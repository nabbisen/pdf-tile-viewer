//! Root component (RFC 001 shell + RFC 005/M4 tile viewer).
//!
//! Boot order: engine is already bound by `main`; we create the
//! `RenderService` (RFC 007) here and provide it via context so every
//! nested component can request renders without threading handles through
//! props.

use base64::Engine as _;
use dioxus::prelude::*;

use app_services::document_service;
use app_services::history_service::SessionHistory;
use app_services::render_service::{DEFAULT_CACHE_BUDGET_BYTES, RenderService};
use app_services::settings_service::SettingsStore;
use domain::document::PageIndex;
use domain::render::{
    RenderFlags, RenderOutputFormat, RenderPageRequest, RenderedImagePayload, ScaleBucket,
};
use domain::settings::AppSettingsV1;

use crate::components::error_panel::ErrorPanel;
use crate::i18n::{self, Locale, MessageKey, t};
use crate::screens::dashboard::Dashboard;
use crate::screens::viewer::Viewer;
use crate::state::{self, OpenDocumentView, Phase};

const STYLE: &str = include_str!("../assets/main.css");

#[component]
pub fn App() -> Element {
    let settings_store = use_hook(SettingsStore::at_default_location);
    let settings = use_signal(|| settings_store.load().0);
    let locale = use_memo(move || Locale::resolve(settings.read().ui.locale.as_deref()));
    let phase = use_signal(Phase::default);
    let history = use_signal(SessionHistory::default);
    let last_error = use_signal(|| Option::<MessageKey>::None);

    use_context_provider(|| settings);
    use_context_provider(|| locale);

    // Provide the render service; all descendant components share it.
    if let Some(engine) = state::engine() {
        let budget = settings
            .peek()
            .advanced
            .render_cache_budget_mb
            .map(|mb| (mb as usize).saturating_mul(1024 * 1024))
            .unwrap_or(DEFAULT_CACHE_BUDGET_BYTES);
        use_context_provider(|| RenderService::with_default_budget(engine));
        let _ = budget; // will be plumbed in RFC 008 settings panel
    }

    let body = match &*phase.read() {
        Phase::Dashboard => rsx! {
            Dashboard {
                history,
                on_open: open_document_action(settings, phase, history, last_error),
            }
        },
        Phase::Opening => rsx! {
            main { class: "centered",
                p { class: "muted", {t(locale(), MessageKey::OpeningDocument)} }
            }
        },
        Phase::Viewer(view) => rsx! {
            Viewer { view: view.clone(), phase }
        },
    };

    rsx! {
        style { {STYLE} }
        div { class: "app-shell",
            if let Some(boot_error) = state::engine_boot_error() {
                ErrorPanel {
                    title: t(locale(), MessageKey::EngineUnavailableTitle).to_string(),
                    body: format!("{} ({boot_error})", t(locale(), MessageKey::EngineUnavailableBody)),
                }
            }
            if let Some(key) = *last_error.read() {
                p { class: "toast-error", {t(locale(), key)} }
            }
            {body}
        }
    }
}

fn open_document_action(
    settings: Signal<AppSettingsV1>,
    mut phase: Signal<Phase>,
    mut history: Signal<SessionHistory>,
    mut last_error: Signal<Option<MessageKey>>,
) -> Callback<()> {
    Callback::new(move |_| {
        let default_scale = settings.read().viewer.default_scale;
        spawn(async move {
            let Some(path) = app_services::platform::pick_pdf_file().await else {
                return;
            };
            let Some(engine) = state::engine() else {
                last_error.set(Some(MessageKey::EngineUnavailableTitle));
                return;
            };

            last_error.set(None);
            phase.set(Phase::Opening);

            let session = match document_service::open_document(&engine, &path).await {
                Ok(session) => session,
                Err(e) => {
                    last_error.set(Some(i18n::open_error_key(&e)));
                    phase.set(Phase::Dashboard);
                    return;
                }
            };
            history.write().record_opened(&session);

            let mut view = OpenDocumentView {
                session: session.clone(),
                page_one_data_uri: None,
                render_error: None,
            };

            // Render page 1 preview for the Viewer initial header image
            // (superseded by the tile grid in M4, but retained as fallback).
            let request = RenderPageRequest {
                document_id: session.id,
                generation: session.generation,
                page_index: PageIndex(0),
                scale_bucket: ScaleBucket::from_scale(default_scale),
                flags: RenderFlags::default(),
                format: RenderOutputFormat::Png,
            };
            match engine.render_page(request).await {
                Ok(Ok(image)) => {
                    if let RenderedImagePayload::Bytes(png) = image.payload {
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
                        view.page_one_data_uri = Some(format!("data:image/png;base64,{b64}"));
                    }
                }
                Ok(Err(e)) => view.render_error = Some(format!("{e:?}")),
                Err(_) => view.render_error = Some("engine unavailable".to_string()),
            }

            phase.set(Phase::Viewer(view));
        });
    })
}
