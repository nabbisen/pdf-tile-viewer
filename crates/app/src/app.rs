//! Root component (RFC 001 shell, M5: settings persistence + drag-drop).

use std::path::PathBuf;

use dioxus::document::Title;
use dioxus::prelude::*;

use app_services::document_service::{self, OpenDocumentOutcome};
use app_services::history_service::SessionHistory;
use app_services::render_service::RenderService;
use app_services::settings_service::SettingsStore;
use app_services::text_service::{DEFAULT_TEXT_LAYER_CACHE_BUDGET_BYTES, TextLayerService};
use domain::document::DocumentPassword;

use crate::components::error_panel::ErrorPanel;
use crate::components::password_prompt::PasswordPrompt;
use crate::i18n::{self, Locale, MessageKey, t};
use crate::screens::dashboard::Dashboard;
use crate::screens::viewer::Viewer;
use crate::state::{self, OpenDocumentView, PasswordPromptState, Phase};

const STYLE: &str = include_str!("../assets/main.css");

#[component]
pub fn App() -> Element {
    let settings_store = use_hook(SettingsStore::at_default_location);
    let settings = use_signal(|| settings_store.load().0);
    let locale = use_memo(move || Locale::resolve(settings.read().ui.locale.as_deref()));
    let phase = use_signal(Phase::default);
    let history = use_signal(SessionHistory::default);
    let last_error: Signal<Option<MessageKey>> = use_signal(|| None);
    let password_prompt: Signal<Option<PasswordPromptState>> = use_signal(|| None);

    use_context_provider(|| settings);
    use_context_provider(|| locale);

    if let Some(engine) = state::engine() {
        let budget_bytes = settings
            .peek()
            .advanced
            .render_cache_budget_mb
            .map(|mb| (mb as usize) * 1024 * 1024)
            .unwrap_or(app_services::render_service::DEFAULT_CACHE_BUDGET_BYTES);
        use_context_provider(|| RenderService::new(engine.clone(), budget_bytes));
        use_context_provider(|| {
            TextLayerService::new(engine, DEFAULT_TEXT_LAYER_CACHE_BUDGET_BYTES)
        });
    }

    // Provide settings store for downstream save callbacks.
    use_context_provider(|| settings_store);

    let body = match &*phase.read() {
        Phase::Dashboard => {
            let open_picker = open_action(phase, history, last_error, password_prompt, None);
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
                        let password_prompt2 = password_prompt;
                        Callback::new(move |path: PathBuf| {
                            open_action(
                                phase2,
                                history2,
                                last_error2,
                                password_prompt2,
                                Some(path),
                            ).call(());
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
                let app_title = t(locale(), MessageKey::AppTitle);
                let title = if settings.read().privacy.show_full_path_in_title {
                    match &view.session.source {
                        domain::document::DocumentSource::LocalFile { path, .. } => {
                            format!("{} — {app_title}", path.display())
                        }
                    }
                } else {
                    format!("{} — {app_title}", view.session.display_name)
                };
                rsx! { Title { "{title}" } }
            }
            _ => rsx! { Title { {t(locale(), MessageKey::AppTitle)} } },
        }}
        div { class: "app-shell",
            if let Some(boot_error) = state::engine_boot_error() {
                {
                    let body = if cfg!(debug_assertions) {
                        format!(
                            "{}\n\n{}\n\n{}: {boot_error}",
                            t(locale(), MessageKey::EngineUnavailableBody),
                            t(locale(), MessageKey::EngineUnavailableDevelopmentHelp),
                            t(locale(), MessageKey::DiagnosticDetailsLabel),
                        )
                    } else {
                        format!(
                            "{}\n\n{}",
                            t(locale(), MessageKey::EngineUnavailableBody),
                            t(locale(), MessageKey::EngineUnavailablePackagedHelp),
                        )
                    };
                    rsx! {
                        ErrorPanel {
                            title: t(locale(), MessageKey::EngineUnavailableTitle).to_string(),
                            body,
                        }
                    }
                }
            }
            {body}
            if let Some(prompt) = password_prompt.read().clone() {
                PasswordPrompt {
                    rejected: prompt.rejected,
                    submitting: prompt.submitting,
                    on_cancel: {
                        let mut password_prompt = password_prompt;
                        Callback::new(move |_| {
                            password_prompt.set(None);
                        })
                    },
                    on_submit: {
                        let mut phase = phase;
                        let mut history = history;
                        let mut last_error = last_error;
                        let mut password_prompt = password_prompt;
                        Callback::new(move |password: String| {
                            let Some(current) = password_prompt.peek().clone() else {
                                return;
                            };
                            let path = current.path.clone();
                            password_prompt.set(Some(PasswordPromptState {
                                path: path.clone(),
                                rejected: false,
                                submitting: true,
                            }));
                            spawn(async move {
                                let Some(engine) = state::engine() else {
                                    last_error.set(Some(MessageKey::EngineUnavailableTitle));
                                    password_prompt.set(None);
                                    return;
                                };
                                let result = document_service::open_document_with_password(
                                    &engine,
                                    &path,
                                    DocumentPassword::new(password),
                                )
                                .await;
                                let still_current = password_prompt
                                    .peek()
                                    .as_ref()
                                    .is_some_and(|state| state.path == path);
                                if !still_current {
                                    return;
                                }
                                match result {
                                    Ok(OpenDocumentOutcome::Opened(session)) => {
                                        history.write().record_opened(&session);
                                        phase.set(Phase::Viewer(OpenDocumentView { session }));
                                        last_error.set(None);
                                        password_prompt.set(None);
                                    }
                                    Ok(OpenDocumentOutcome::PasswordRequired(ctx)) => {
                                        password_prompt.set(Some(PasswordPromptState {
                                            path: ctx.path,
                                            rejected: true,
                                            submitting: false,
                                        }));
                                    }
                                    Err(e) => {
                                        last_error.set(Some(i18n::open_error_key(&e)));
                                        password_prompt.set(None);
                                    }
                                }
                            });
                        })
                    },
                }
            }
        }
    }
}

/// Build an open-document action.  If `path` is given, skip the picker.
fn open_action(
    mut phase: Signal<Phase>,
    mut history: Signal<SessionHistory>,
    mut last_error: Signal<Option<MessageKey>>,
    mut password_prompt: Signal<Option<PasswordPromptState>>,
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
            password_prompt.set(None);
            let previous_phase = phase.peek().clone();
            phase.set(Phase::Opening);

            match document_service::open_document(&engine, &resolved_path).await {
                Ok(OpenDocumentOutcome::Opened(session)) => {
                    history.write().record_opened(&session);
                    phase.set(Phase::Viewer(OpenDocumentView { session }));
                }
                Ok(OpenDocumentOutcome::PasswordRequired(ctx)) => {
                    phase.set(previous_phase);
                    password_prompt.set(Some(PasswordPromptState {
                        path: ctx.path,
                        rejected: false,
                        submitting: false,
                    }));
                }
                Err(e) => {
                    last_error.set(Some(i18n::open_error_key(&e)));
                    phase.set(previous_phase);
                }
            }
        });
    })
}
