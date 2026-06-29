//! Viewer (RFC 005 vertical slice): document header + page-1 preview.
//! The tile grid (RFC 006) and lazy render queue (RFC 007) replace the
//! single preview in M4.
//!
//! Privacy: the header shows `display_name`, never the full path, unless
//! the (default-off) privacy setting allows it (RFC 016 §7).

use dioxus::prelude::*;

use crate::i18n::{Locale, MessageKey, t};
use crate::state::{OpenDocumentView, Phase};

#[component]
pub fn Viewer(view: OpenDocumentView, phase: Signal<Phase>) -> Element {
    let locale: Memo<Locale> = use_context();
    let page_count = view.session.pages.len();
    // One-based display numbering at the UI edge only (Appendix A §3).
    let first_display_number = view
        .session
        .pages
        .first()
        .map(|p| p.page_index.display_number())
        .unwrap_or(1);

    rsx! {
        main { class: "viewer",
            header { class: "viewer-header",
                button {
                    class: "ghost",
                    onclick: move |_| phase.set(Phase::Dashboard),
                    {t(locale(), MessageKey::ViewerBackToDashboard)}
                }
                h1 { "{view.session.display_name}" }
                span { class: "muted",
                    {t(locale(), MessageKey::ViewerPageCountLabel)}
                    ": {page_count}"
                }
            }

            section { class: "page-preview",
                h2 { {t(locale(), MessageKey::ViewerPageOnePreview)} }
                if let Some(uri) = view.page_one_data_uri.as_ref() {
                    figure {
                        img {
                            class: "page-image",
                            alt: "Page {first_display_number}",
                            src: "{uri}",
                        }
                    }
                } else if let Some(err) = view.render_error.as_ref() {
                    p { class: "toast-error",
                        {t(locale(), MessageKey::ErrRenderFailed)}
                        " ({err})"
                    }
                } else {
                    p { class: "muted", {t(locale(), MessageKey::RenderingPage)} }
                }
            }
        }
    }
}
