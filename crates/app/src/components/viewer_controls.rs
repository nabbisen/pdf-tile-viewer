//! Viewer toolbar: scale, pages-per-row, page numbers, jump-to-page
//! (RFC 008 §5 ViewerToolbar).

use dioxus::prelude::*;

use domain::layout::{MAX_FIXED_PAGES_PER_ROW, PagesPerRowMode, ViewerScale};

/// Viewer toolbar props.
#[component]
pub fn ViewerControls(
    page_count: usize,
    scale: Signal<f32>,
    mode: Signal<PagesPerRowMode>,
    show_page_numbers: Signal<bool>,
    /// Callback(page_index 0-based) to request scroll-to-page.
    on_jump: Callback<usize>,
) -> Element {
    let jump_input = use_signal(String::new);
    let jump_error = use_signal(|| Option::<String>::None);

    rsx! {
        div { class: "viewer-controls",

            // ── Scale ──────────────────────────────────────────────────────
            div { class: "control-group",
                label { class: "control-label", "Scale" }
                button {
                    class: "ghost icon-btn",
                    title: "Zoom out",
                    disabled: *scale.read() <= ViewerScale::MIN,
                    onclick: move |_| {
                        let v = (*scale.read() - 0.2).max(ViewerScale::MIN);
                        scale.set((v * 10.0).round() / 10.0);
                    },
                    "−"
                }
                input {
                    r#type: "range",
                    class: "scale-slider",
                    min: "{(ViewerScale::MIN * 10.0) as i32}",
                    max: "{(ViewerScale::MAX * 10.0) as i32}",
                    step: "1",
                    value: "{(*scale.read() * 10.0).round() as i32}",
                    oninput: move |evt| {
                        if let Ok(v) = evt.value().parse::<f32>() {
                            scale.set((v / 10.0).clamp(ViewerScale::MIN, ViewerScale::MAX));
                        }
                    },
                }
                button {
                    class: "ghost icon-btn",
                    title: "Zoom in",
                    disabled: *scale.read() >= ViewerScale::MAX,
                    onclick: move |_| {
                        let v = (*scale.read() + 0.2).min(ViewerScale::MAX);
                        scale.set((v * 10.0).round() / 10.0);
                    },
                    "+"
                }
                span { class: "muted scale-label",
                    "{(*scale.read() * 100.0).round() as i32}%"
                }
            }

            // ── Pages per row ──────────────────────────────────────────────
            div { class: "control-group",
                label { class: "control-label", "Cols" }
                button {
                    class: if matches!(*mode.read(), PagesPerRowMode::Auto) { "ghost active" } else { "ghost" },
                    onclick: move |_| mode.set(PagesPerRowMode::Auto),
                    "Auto"
                }
                input {
                    r#type: "number",
                    class: "pages-per-row-input",
                    min: "1",
                    max: "{MAX_FIXED_PAGES_PER_ROW}",
                    value: {
                        match *mode.read() {
                            PagesPerRowMode::Fixed(n) => format!("{n}"),
                            PagesPerRowMode::Auto => "5".to_string(),
                        }
                    },
                    oninput: move |evt| {
                        if let Ok(n) = evt.value().parse::<u16>() {
                            let clamped = n.clamp(1, MAX_FIXED_PAGES_PER_ROW);
                            mode.set(PagesPerRowMode::Fixed(clamped));
                        }
                    },
                }
            }

            // ── Page numbers toggle ────────────────────────────────────────
            div { class: "control-group",
                label { class: "control-label",
                    input {
                        r#type: "checkbox",
                        checked: *show_page_numbers.read(),
                        onchange: move |evt| show_page_numbers.set(evt.checked()),
                    }
                    " #"
                }
            }

            // ── Jump to page ───────────────────────────────────────────────
            div { class: "control-group",
                label { class: "control-label", "p." }
                input {
                    r#type: "number",
                    class: "jump-input",
                    min: "1",
                    max: "{page_count}",
                    placeholder: "…",
                    value: "{jump_input.read()}",
                    oninput: {
                        let mut jump_input = jump_input.clone();
                        move |evt| jump_input.set(evt.value())
                    },
                    onkeydown: {
                        let jump_input = jump_input.clone();
                        let mut jump_error = jump_error.clone();
                        let on_jump = on_jump.clone();
                        move |evt: Event<KeyboardData>| {
                            if evt.key() == Key::Enter {
                                handle_jump(
                                    &jump_input.read(),
                                    page_count,
                                    &on_jump,
                                    &mut jump_error,
                                );
                            }
                        }
                    },
                }
                button {
                    class: "ghost",
                    onclick: {
                        let mut jump_error = jump_error.clone();
                        let on_jump = on_jump.clone();
                        move |_| {
                            handle_jump(
                                &jump_input.read(),
                                page_count,
                                &on_jump,
                                &mut jump_error,
                            );
                        }
                    },
                    "Go"
                }
                if let Some(err) = &*jump_error.read() {
                    span { class: "muted jump-error", "{err}" }
                }
            }
        }
    }
}

/// Validate and fire the jump callback (RFC 008 §9.3: one-based input).
fn handle_jump(
    raw: &str,
    page_count: usize,
    on_jump: &Callback<usize>,
    jump_error: &mut Signal<Option<String>>,
) {
    match raw.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= page_count => {
            jump_error.set(None);
            on_jump.call(n - 1); // convert to 0-based PageIndex
        }
        Ok(_) => jump_error.set(Some(format!("1–{page_count}"))),
        Err(_) => jump_error.set(Some("?".to_string())),
    }
}
