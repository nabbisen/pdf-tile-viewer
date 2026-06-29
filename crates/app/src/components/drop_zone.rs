//! Reusable PDF drop zone (RFC 002 §5.2).
//!
//! Accepts a single dropped PDF; rejects multiple files with a toast.
//! Calls `on_drop(path)` with the validated path; error handling
//! (magic-header check etc.) is performed by the caller via
//! `document_service::validate_candidate`.

use std::path::PathBuf;

use dioxus::html::HasFileData;
use dioxus::prelude::*;

use crate::i18n::{Locale, MessageKey, t};

#[component]
pub fn DropZone(
    on_drop: Callback<PathBuf>,
    on_error: Callback<MessageKey>,
    children: Element,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let mut is_over = use_signal(|| false);

    rsx! {
        div {
            class: if *is_over.read() { "drop-zone drag-over" } else { "drop-zone" },
            ondragover: move |evt| {
                evt.prevent_default();
                is_over.set(true);
            },
            ondragleave: move |_| is_over.set(false),
            ondrop: move |evt: Event<DragData>| {
                evt.prevent_default();
                is_over.set(false);
                let files = (&*evt.data()).files();
                match files.len() {
                    0 => {} // no files — ignore silently
                    1 => {
                        let path = files[0].path();
                        on_drop.call(path);
                    }
                    _ => on_error.call(MessageKey::ErrMultipleFilesDropped),
                }
            },
            {children}
            p { class: "drop-hint muted", {t(locale(), MessageKey::DropZoneHint)} }
        }
    }
}
