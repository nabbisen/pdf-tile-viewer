//! Root component (RFC 001 shell, M5: settings persistence + drag-drop).

use std::path::PathBuf;

use dioxus::document::{self, Title};
use dioxus::prelude::*;

use app_services::document_service::{self, OpenDocumentOutcome};
use app_services::history_service::SessionHistory;
use app_services::navigation_service::NavigationService;
use app_services::render_service::RenderService;
use app_services::session_service::SessionService;
use app_services::settings_service::SettingsStore;
use app_services::text_service::{DEFAULT_TEXT_LAYER_CACHE_BUDGET_BYTES, TextLayerService};
use domain::document::DocumentPassword;

use crate::components::error_panel::ErrorPanel;
use crate::components::password_prompt::PasswordPrompt;
use crate::i18n::{self, Locale, MessageKey, t};
use crate::page_meta;
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

    // The services share one engine; `SessionService` owns the session lifecycle (RFC 035).
    let session_service: Option<SessionService> = state::engine().map(|engine| {
        let budget_bytes = settings
            .peek()
            .advanced
            .render_cache_budget_mb
            .map(|mb| (mb as usize) * 1024 * 1024)
            .unwrap_or(app_services::render_service::DEFAULT_CACHE_BUDGET_BYTES);
        let render = use_hook(|| RenderService::new(engine.clone(), budget_bytes));
        let navigation = use_hook(|| NavigationService::new(engine.clone()));
        let text = use_hook(|| {
            TextLayerService::new(engine.clone(), DEFAULT_TEXT_LAYER_CACHE_BUDGET_BYTES)
        });
        let session = use_hook(|| {
            SessionService::new(
                engine.clone(),
                render.clone(),
                text.clone(),
                navigation.clone(),
            )
        });
        use_context_provider(|| render);
        use_context_provider(|| navigation);
        use_context_provider(|| text);
        use_context_provider(|| session.clone());
        session
    });

    // Provide settings store for downstream save callbacks.
    use_context_provider(|| settings_store);

    // One title for the window and for the web document (RFC 016 § 7: full path only when opted in).
    let window_title = use_memo(move || {
        let app_title = t(locale(), MessageKey::AppTitle);
        match &*phase.read() {
            Phase::Viewer(view) => {
                let name = if settings.read().privacy.show_full_path_in_title {
                    match &view.session.source {
                        domain::document::DocumentSource::LocalFile { path, .. } => {
                            path.display().to_string()
                        }
                    }
                } else {
                    view.session.display_name.clone()
                };
                page_meta::title_text(app_title, Some(&name))
            }
            _ => page_meta::title_text(app_title, None),
        }
    });
    // `<html lang>` and `document.title` follow the locale and the open document.
    use_effect(move || {
        let script = page_meta::meta_script(page_meta::lang_tag(locale()), &window_title.read());
        let _ = document::eval(&script);
    });

    let body = match &*phase.read() {
        Phase::Dashboard => {
            let open_picker = open_action(
                phase,
                history,
                last_error,
                password_prompt,
                session_service.clone(),
                None,
            );
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
                        let session_service2 = session_service.clone();
                        Callback::new(move |path: PathBuf| {
                            open_action(
                                phase2,
                                history2,
                                last_error2,
                                password_prompt2,
                                session_service2.clone(),
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
        Title { "{window_title.read()}" }
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
                        let session_service = session_service.clone();
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
                            let session_service = session_service.clone();
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
                                    // The prompt was cancelled or replaced meanwhile: a document
                                    // that did open must not stay open in the engine.
                                    if let (Ok(OpenDocumentOutcome::Opened(session)), Some(service)) =
                                        (&result, &session_service)
                                    {
                                        service.close(session.id, session.generation);
                                    }
                                    return;
                                }
                                match result {
                                    Ok(OpenDocumentOutcome::Opened(session)) => {
                                        if let Some(service) = &session_service {
                                            service.opened(&session);
                                        }
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
    session_service: Option<SessionService>,
    path: Option<PathBuf>,
) -> Callback<()> {
    Callback::new(move |_| {
        let path_opt = path.clone();
        let session_service = session_service.clone();
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
                    if let Some(service) = &session_service {
                        service.opened(&session);
                    }
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
