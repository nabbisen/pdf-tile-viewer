//! External-link dialog of the zoom overlay (RFC 026 PR4): copy-only inspection
//! of an external URI, with its own modal focus handling.

use dioxus::document::eval;
use dioxus::prelude::*;

use app_services::uri_policy::{ExternalUriOpenDecision, validate_external_uri_for_open};
use domain::navigation::NavigationResourceLimits;

use crate::i18n::{Locale, MessageKey, t};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ExternalUriDialogState {
    pub(super) raw_uri: String,
    pub(super) copy_status: ExternalUriCopyStatus,
    pub(super) dialog_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ExternalUriCopyStatus {
    Idle,
    Copied,
    Failed,
}

/// Renders the dialog while `external_uri_dialog` holds a state; closing it
/// returns focus to the overlay's close button.
pub(super) fn external_uri_dialog_layer(
    mut external_uri_dialog: Signal<Option<ExternalUriDialogState>>,
) -> Element {
    rsx! {
        if let Some(dialog) = external_uri_dialog.read().clone() {
            ExternalUriDialog {
                raw_uri: dialog.raw_uri,
                copy_status: dialog.copy_status,
                dialog_id: dialog.dialog_id,
                on_copy: Callback::new(move |(dialog_id, raw_uri): (u64, String)| {
                    external_uri_dialog.set(Some(ExternalUriDialogState {
                        raw_uri: raw_uri.clone(),
                        copy_status: ExternalUriCopyStatus::Idle,
                        dialog_id,
                    }));
                    spawn(async move {
                        let next_status = if copy_text_to_clipboard(raw_uri.clone()).await {
                            ExternalUriCopyStatus::Copied
                        } else {
                            ExternalUriCopyStatus::Failed
                        };
                        let still_current = external_uri_dialog
                            .peek()
                            .as_ref()
                            .is_some_and(|state| {
                                state.dialog_id == dialog_id && state.raw_uri == raw_uri
                            });
                        if !still_current {
                            return;
                        }
                        external_uri_dialog.set(Some(ExternalUriDialogState {
                            raw_uri,
                            copy_status: next_status,
                            dialog_id,
                        }));
                    });
                }),
                on_close: Callback::new(move |_| {
                    external_uri_dialog.set(None);
                    focus_zoom_close_button();
                }),
            }
        }
    }
}

#[component]
fn ExternalUriDialog(
    raw_uri: String,
    copy_status: ExternalUriCopyStatus,
    dialog_id: u64,
    on_copy: Callback<(u64, String)>,
    on_close: Callback<()>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let open_decision =
        validate_external_uri_for_open(&raw_uri, &NavigationResourceLimits::default());
    let policy_message = match open_decision {
        ExternalUriOpenDecision::Allowed { .. } => t(locale(), MessageKey::ExternalUriCopyOnlyBody),
        ExternalUriOpenDecision::Rejected(_) => t(locale(), MessageKey::ExternalUriRejectedBody),
    };

    rsx! {
        div {
            class: "modal-backdrop",
            role: "presentation",
            onkeydown: move |evt: Event<KeyboardData>| {
                evt.stop_propagation();
                match evt.key() {
                    Key::Escape => on_close.call(()),
                    Key::Tab => {
                        evt.prevent_default();
                        trap_external_uri_dialog_focus(evt.modifiers().shift());
                    }
                    _ => {}
                }
            },
            section {
                class: "external-uri-dialog",
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "external-uri-dialog-title",
                onmounted: move |_| {
                    focus_external_uri_cancel_button();
                },
                h2 {
                    id: "external-uri-dialog-title",
                    {t(locale(), MessageKey::ExternalUriDialogTitle)}
                }
                p { class: "muted", "{policy_message}" }
                p { class: "muted", {t(locale(), MessageKey::ExternalUriOpenUnavailable)} }
                label {
                    class: "external-uri-label",
                    r#for: "external-uri-value",
                    {t(locale(), MessageKey::ExternalUriTargetLabel)}
                }
                code {
                    id: "external-uri-value",
                    class: "external-uri-value",
                    "{raw_uri}"
                }
                match copy_status {
                    ExternalUriCopyStatus::Idle => rsx! {},
                    ExternalUriCopyStatus::Copied => rsx! {
                        p { class: "external-uri-status", role: "status",
                            {t(locale(), MessageKey::ExternalUriCopyDone)}
                        }
                    },
                    ExternalUriCopyStatus::Failed => rsx! {
                        p { class: "external-uri-status error", role: "alert",
                            {t(locale(), MessageKey::ExternalUriCopyFailed)}
                        }
                    },
                }
                div { class: "external-uri-actions",
                    button {
                        id: "external-uri-cancel",
                        class: "ghost",
                        autofocus: true,
                        onclick: move |_| on_close.call(()),
                        {t(locale(), MessageKey::ExternalUriCancel)}
                    }
                    button {
                        id: "external-uri-copy",
                        class: "primary",
                        onclick: move |_| on_copy.call((dialog_id, raw_uri.clone())),
                        {t(locale(), MessageKey::ExternalUriCopy)}
                    }
                }
            }
        }
    }
}

fn focus_external_uri_cancel_button() {
    let _ = eval(
        "const focusCancel = () => document.getElementById('external-uri-cancel')?.focus();\
         focusCancel();\
         setTimeout(focusCancel, 0);",
    );
}

pub(super) fn focus_zoom_close_button() {
    let _ = eval("setTimeout(() => document.getElementById('zoom-close-button')?.focus(), 0);");
}

fn trap_external_uri_dialog_focus(backward: bool) {
    let step = if backward { "-1" } else { "1" };
    let script = format!(
        "const ids = ['external-uri-cancel', 'external-uri-copy'];\
         const active = document.activeElement && document.activeElement.id;\
         const current = ids.indexOf(active);\
         const next = current < 0 ? 0 : (current + ({step}) + ids.length) % ids.length;\
         document.getElementById(ids[next])?.focus();"
    );
    let _ = eval(&script);
}

async fn copy_text_to_clipboard(text: String) -> bool {
    let Ok(quoted) = serde_json::to_string(&text) else {
        return false;
    };
    let script = format!(
        "const text = {quoted};\
         if (!navigator.clipboard || !navigator.clipboard.writeText) return false;\
         return navigator.clipboard.writeText(text).then(() => true).catch(() => false);"
    );
    eval(&script).join::<bool>().await.unwrap_or(false)
}
