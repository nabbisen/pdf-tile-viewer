//! The viewer's toolbar (back, outline, search, reveal, zen): a pure move out of `viewer.rs`
//! (RFC 033 final sync). Behavior, ids and names are unchanged; the focus rules of RFC 033 depend
//! on the element ids (`viewer-outline-button`, `viewer-search-button`, `viewer-zen-button`).

use dioxus::prelude::*;

use crate::i18n::{Locale, MessageKey, t};
use crate::state::Phase;

#[component]
pub(super) fn ViewerToolbar(
    display_name: String,
    /// Whether the document has a local path (shows the reveal-in-file-manager button).
    can_reveal: bool,
    mut show_outline: Signal<bool>,
    mut show_search: Signal<bool>,
    mut zen_mode: Signal<bool>,
    reveal_error: Signal<Option<MessageKey>>,
    on_reveal: Callback<()>,
    mut phase: Signal<Phase>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    rsx! {
        header { class: "viewer-header",
            button { class: "ghost", onclick: move |_| phase.set(Phase::Dashboard),
                {t(locale(), MessageKey::ViewerBackToDashboard)}
            }
            h1 { class: "viewer-title", "{display_name}" }
            button {
                id: "viewer-outline-button",
                class: if *show_outline.read() { "ghost icon-btn active" } else { "ghost icon-btn" },
                title: t(locale(), MessageKey::OutlinePanelLabel),
                "aria-label": t(locale(), MessageKey::OutlinePanelLabel),
                "aria-expanded": if *show_outline.read() { "true" } else { "false" },
                onclick: move |_| show_outline.toggle(),
                "☰"
            }
            button {
                id: "viewer-search-button",
                class: if *show_search.read() { "ghost icon-btn active" } else { "ghost icon-btn" },
                title: t(locale(), MessageKey::SearchToggleLabel),
                "aria-label": t(locale(), MessageKey::SearchToggleLabel),
                "aria-expanded": if *show_search.read() { "true" } else { "false" },
                onclick: move |_| show_search.toggle(),
                "🔍"
            }
            if can_reveal {
                button {
                    class: "ghost icon-btn",
                    title: t(locale(), MessageKey::RevealInFileManager),
                    "aria-label": t(locale(), MessageKey::RevealInFileManager),
                    onclick: move |_| on_reveal.call(()),
                    "📂"
                }
            }
            // a live region that exists before its text changes (RFC 037 § 5.6.3)
            span { role: "alert",
                if let Some(key) = *reveal_error.read() {
                    span { class: "toast-error", {t(locale(), key)} }
                }
            }
            button {
                id: "viewer-zen-button",
                class: "ghost icon-btn",
                title: t(locale(), MessageKey::ZenModeEnter),
                "aria-label": t(locale(), MessageKey::ZenModeEnter),
                onclick: move |_| zen_mode.set(true),
                "⊞"
            }
        }
    }
}
