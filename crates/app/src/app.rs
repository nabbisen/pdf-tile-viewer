//! Root component (RFC 001 shell, M5: settings persistence + drag-drop).

use std::path::PathBuf;

use dioxus::document::Title;
use dioxus::prelude::*;

use app_services::document_service;
use app_services::history_service::SessionHistory;
use app_services::render_service::RenderService;
use app_services::settings_service::SettingsStore;

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
        let budget_bytes = settings
            .peek()
            .advanced
            .render_cache_budget_mb
            .map(|mb| (mb as usize) * 1024 * 1024)
            .unwrap_or(app_services::render_service::DEFAULT_CACHE_BUDGET_BYTES);
        use_context_provider(|| RenderService::new(engine, budget_bytes));
    }

    // Provide settings store for downstream save callbacks.
    use_context_provider(|| settings_store);

    let body = match &*phase.read() {
        Phase::Dashboard => {
            let open_picker = open_action(phase, history, last_error, None);
            let last_error_clone = last_error;
            rsx! {
                Dashboard {
                    history,
                    last_error: last_error_clone,
                    on_open: open_picker,
                    on_open_path: {
                        let phase2 = phase;
                        let history2 = history;
                        let last_error2 = last_error;
                        Callback::new(move |path: PathBuf| {
                            open_action(phase2, history2, last_error2, Some(path)).call(());
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
        // Window title: privacy.show_full_path_in_title controls the path (RFC 016 §7).
        {match &*phase.read() {
            Phase::Viewer(view) => {
                let title = if settings.read().privacy.show_full_path_in_title {
                    match &view.session.source {
                        domain::document::DocumentSource::LocalFile { path, .. } => {
                            format!("{} — PDF Tile Viewer", path.display())
                        }
                    }
                } else {
                    format!("{} — PDF Tile Viewer", view.session.display_name)
                };
                rsx! { Title { "{title}" } }
            }
            _ => rsx! { Title { "PDF Tile Viewer" } },
        }}
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
    mut phase: Signal<Phase>,
    mut history: Signal<SessionHistory>,
    mut last_error: Signal<Option<MessageKey>>,
    path: Option<PathBuf>,
) -> Callback<()> {
    Callback::new(move |_| {
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

            phase.set(Phase::Viewer(OpenDocumentView { session }));
        });
    })
}
