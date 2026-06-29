//! Root component (RFC 001 shell, M5: settings persistence + drag-drop).

use std::path::PathBuf;

use base64::Engine as _;
use dioxus::prelude::*;

use app_services::document_service;
use app_services::history_service::SessionHistory;
use app_services::render_service::RenderService;
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
    let last_error: Signal<Option<MessageKey>> = use_signal(|| None);

    use_context_provider(|| settings);
    use_context_provider(|| locale);

    if let Some(engine) = state::engine() {
        use_context_provider(|| RenderService::with_default_budget(engine));
    }

    // Provide settings store for downstream save callbacks.
    use_context_provider(|| settings_store);

    let body = match &*phase.read() {
        Phase::Dashboard => {
            let open_picker = open_action(settings, phase, history, last_error, None);
            let last_error_clone = last_error;
            rsx! {
                Dashboard {
                    history,
                    last_error: last_error_clone,
                    on_open: open_picker,
                    on_open_path: {
                        let settings2 = settings;
                        let phase2 = phase;
                        let history2 = history;
                        let last_error2 = last_error;
                        Callback::new(move |path: PathBuf| {
                            open_action(settings2, phase2, history2, last_error2, Some(path)).call(());
                        })
                    },
                }
            }
        }
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
                    body: format!(
                        "{} ({boot_error})",
                        t(locale(), MessageKey::EngineUnavailableBody)
                    ),
                }
            }
            {body}
        }
    }
}

/// Build an open-document action.  If `path` is given, skip the picker.
fn open_action(
    settings: Signal<AppSettingsV1>,
    mut phase: Signal<Phase>,
    mut history: Signal<SessionHistory>,
    mut last_error: Signal<Option<MessageKey>>,
    path: Option<PathBuf>,
) -> Callback<()> {
    Callback::new(move |_| {
        let default_scale = settings.read().viewer.default_scale;
        let path_opt = path.clone();
        spawn(async move {
            let resolved = match path_opt {
                Some(p) => Some(p),
                None => app_services::platform::pick_pdf_file().await,
            };
            let Some(resolved_path) = resolved else {
                return; // cancelled
            };
            let Some(engine) = state::engine() else {
                last_error.set(Some(MessageKey::EngineUnavailableTitle));
                return;
            };

            last_error.set(None);
            phase.set(Phase::Opening);

            let session = match document_service::open_document(&engine, &resolved_path).await {
                Ok(s) => s,
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

            // Render page 1 as preview (used while tile grid initialises).
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
                Err(_) => {} // engine gone
            }

            phase.set(Phase::Viewer(view));
        });
    })
}
