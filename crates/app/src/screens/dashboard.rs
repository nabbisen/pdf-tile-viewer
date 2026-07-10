//! Dashboard screen (RFC 009 §5): open PDF card with drop zone,
//! session history list, and error recovery (RFC 002 §5.2).

use std::path::PathBuf;

use dioxus::prelude::*;

use app_services::history_service::SessionHistory;

use crate::components::drop_zone::DropZone;
use crate::i18n::{Locale, MessageKey, t};

#[component]
pub fn Dashboard(
    history: Signal<SessionHistory>,
    on_open: Callback<()>,
    on_open_path: Callback<PathBuf>,
    last_error: Signal<Option<MessageKey>>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let history_read = history.read();

    rsx! {
        main { class: "dashboard centered",
            h1 { {t(locale(), MessageKey::AppTitle)} }
            // Error toast
            if let Some(key) = *last_error.read() {
                p { class: "toast-error", role: "alert",
                    {t(locale(), key)}
                }
            }

            DropZone {
                on_drop: move |path: PathBuf| on_open_path.call(path),
                on_error: move |key: MessageKey| {
                    let mut err = last_error;
                    err.set(Some(key));
                },
                h2 { {t(locale(), MessageKey::DashboardHeading)} }
                button {
                    class: "primary",
                    "aria-label": t(locale(), MessageKey::OpenPdfButton),
                    onclick: move |_| on_open.call(()),
                    {t(locale(), MessageKey::OpenPdfButton)}
                }
            }

            section { class: "history",
                h3 { {t(locale(), MessageKey::RecentSessionsHeading)} }
                if history_read.entries().is_empty() {
                    p { class: "muted", {t(locale(), MessageKey::RecentSessionsEmpty)} }
                } else {
                    ul {
                        for entry in history_read.entries() {
                            {
                                let reopen_label = format!(
                                    "{} {}",
                                    t(locale(), MessageKey::ReopenDocument),
                                    entry.display_name
                                );
                                let page_count_label = format!(
                                    " — {}{}",
                                    entry.page_count,
                                    t(locale(), MessageKey::PageCountSuffix)
                                );
                                rsx! {
                                    li {
                                        key: "{entry.document_id.0}",
                                        button {
                                            class: "ghost history-btn",
                                            "aria-label": "{reopen_label}",
                                            onclick: {
                                                let path = entry.path.clone();
                                                let on_open_path = on_open_path;
                                                move |_| {
                                                    if let Some(p) = &path {
                                                        on_open_path.call(p.clone());
                                                    }
                                                }
                                            },
                                            span { class: "history-name", "{entry.display_name}" }
                                            span { class: "muted history-pages", "{page_count_label}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
