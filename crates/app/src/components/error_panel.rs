//! Blocking diagnostic panel for engine-level failures (RFC 003 §9).

use dioxus::prelude::*;

#[component]
pub fn ErrorPanel(title: String, body: String) -> Element {
    rsx! {
        aside { class: "error-panel", role: "alert",
            h2 { "{title}" }
            p { "{body}" }
        }
    }
}
