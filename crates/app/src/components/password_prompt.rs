//! Password prompt overlay for password-protected PDFs (RFC 025).

use dioxus::prelude::*;

use crate::i18n::{Locale, MessageKey, t};

#[component]
pub fn PasswordPrompt(
    rejected: bool,
    submitting: bool,
    on_submit: Callback<String>,
    on_cancel: Callback<()>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let mut password = use_signal(String::new);

    let submit = Callback::new(move |_: ()| {
        if submitting {
            return;
        }
        on_submit.call(password.read().clone());
        password.set(String::new());
    });

    rsx! {
        div {
            class: "modal-backdrop",
            role: "presentation",
            onkeydown: {
                move |e: Event<KeyboardData>| {
                    if e.key() == Key::Escape {
                        on_cancel.call(());
                    }
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
                    onkeydown: {
                        move |e: Event<KeyboardData>| {
                            if e.key() == Key::Enter {
                                submit.call(());
                            }
                        }
                    },
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
