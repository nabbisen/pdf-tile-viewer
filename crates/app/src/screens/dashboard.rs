//! Dashboard (RFC 009, vertical-slice scope): open button + session
//! history list. Full history UX (cards, pinned items) arrives in M5.

use dioxus::prelude::*;

use app_services::history_service::SessionHistory;

use crate::i18n::{Locale, MessageKey, t};

#[component]
pub fn Dashboard(history: Signal<SessionHistory>, on_open: Callback<()>) -> Element {
    let locale: Memo<Locale> = use_context();
    let history_read = history.read();

    rsx! {
        main { class: "dashboard centered",
            h1 { {t(locale(), MessageKey::AppTitle)} }
            p { class: "muted", {t(locale(), MessageKey::DashboardHint)} }
            h2 { {t(locale(), MessageKey::DashboardHeading)} }
            button {
                class: "primary",
                onclick: move |_| on_open.call(()),
                {t(locale(), MessageKey::OpenPdfButton)}
            }

            section { class: "history",
                h3 { {t(locale(), MessageKey::RecentSessionsHeading)} }
                if history_read.entries().is_empty() {
                    p { class: "muted", {t(locale(), MessageKey::RecentSessionsEmpty)} }
                } else {
                    ul {
                        for entry in history_read.entries() {
                            li { key: "{entry.document_id.0}",
                                span { class: "history-name", "{entry.display_name}" }
                                span { class: "muted", " — {entry.page_count}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
