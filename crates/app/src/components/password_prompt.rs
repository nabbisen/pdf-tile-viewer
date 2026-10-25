//! Password prompt overlay for password-protected PDFs (RFC 025).

use dioxus::prelude::*;

use domain::shortcuts::{Layer, ShortcutAction};

use crate::components::modal;
use crate::i18n::{Locale, MessageKey, t};
use crate::keys::editable_field_keys;

#[component]
pub fn PasswordPrompt(
    rejected: bool,
    submitting: bool,
    on_submit: Callback<String>,
    on_cancel: Callback<()>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let mut password = use_signal(String::new);
    // Modal focus contract (RFC 033 § 5.5): focus the field, trap Tab, make the page inert.
    modal::use_modal("password-dialog-root", "password-input", "viewer-root");

    let submit = Callback::new(move |_: ()| {
        if submitting {
            return;
        }
        on_submit.call(password.read().clone());
        password.set(String::new());
    });

    rsx! {
        div {
            id: "password-dialog-root",
            class: "modal-backdrop",
            role: "presentation",
            // The prompt owns the keyboard: nothing it receives reaches the page behind it.
            onkeydown: move |e: Event<KeyboardData>| {
                if let Some(ShortcutAction::Dismiss(_)) =
                    modal::modal_key(&e, "password-dialog-root", Layer::Dialog)
                {
                    on_cancel.call(());
                }
            },
            section {
                class: "password-dialog",
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "password-dialog-title",
                h2 {
                    id: "password-dialog-title",
                    {t(locale(), MessageKey::PasswordPromptTitle)}
                }
                p { class: "muted", {t(locale(), MessageKey::PasswordPromptBody)} }
                label {
                    class: "password-label",
                    r#for: "password-input",
                    {t(locale(), MessageKey::PasswordPromptField)}
                }
                input {
                    id: "password-input",
                    class: "password-input",
                    r#type: "password",
                    value: "{password.read()}",
                    disabled: submitting,
                    oninput: move |e| password.set(e.value()),
                    onkeydown: editable_field_keys(on_cancel, Some(submit)),
                }
                if rejected {
                    p {
                        class: "password-error",
                        role: "alert",
                        {t(locale(), MessageKey::PasswordPromptRejected)}
                    }
                }
                div { class: "password-actions",
                    button {
                        class: "ghost",
                        disabled: submitting,
                        onclick: move |_| on_cancel.call(()),
                        {t(locale(), MessageKey::PasswordPromptCancel)}
                    }
                    button {
                        class: "primary",
                        disabled: submitting,
                        onclick: {
                            move |_| submit.call(())
                        },
                        {t(locale(), MessageKey::PasswordPromptOpen)}
                    }
                }
            }
        }
    }
}
