//! PDF link layer of the zoom overlay (RFC 026 PR3): hit areas drawn over the
//! page image, keyboard activation, and click activation with a drag threshold.

use dioxus::document::eval;
use dioxus::prelude::*;

use domain::document::PageIndex;
use domain::navigation::NavigationTarget;

use crate::i18n::{Locale, MessageKey, t};

use super::external_uri_dialog::{ExternalUriCopyStatus, ExternalUriDialogState};
use super::util;

const LINK_CLICK_DRAG_THRESHOLD_PX: f64 = 4.0;

/// Link hit areas for the current page; empty when the page has no links.
pub(super) fn link_layer(
    page_link_rects: &[util::ZoomPageLinkRect],
    current_idx: PageIndex,
    page_count: usize,
    locale: Memo<Locale>,
    on_internal_link: Callback<PageIndex>,
    mut external_uri_dialog_seq: Signal<u64>,
    mut external_uri_dialog: Signal<Option<ExternalUriDialogState>>,
) -> Element {
    rsx! {
        if !page_link_rects.is_empty() {
            div { class: "zoom-link-layer",
                for positioned in page_link_rects.iter() {
                    {
                        let link = &positioned.link;
                        let rect = positioned.rect;
                        let key = format!("{}-{}", current_idx.0, link.id.0);
                        let (lx, ly, lw, lh) = (rect.x, rect.y, rect.width, rect.height);
                        match &link.target {
                            NavigationTarget::InternalDestination(destination) => {
                                let target = destination.page_index;
                                let enabled = target.0 < page_count;
                                rsx! {
                                    button {
                                        key: "{key}",
                                        class: "zoom-link-overlay internal",
                                        disabled: !enabled,
                                        title: t(locale(), MessageKey::ZoomLinkInternal),
                                        "aria-label": t(locale(), MessageKey::ZoomLinkInternal),
                                        style: "left:{lx}px; top:{ly}px; width:{lw}px; height:{lh}px;",
                                        onkeydown: move |evt: Event<KeyboardData>| {
                                            match evt.key() {
                                                Key::Enter => {
                                                    evt.stop_propagation();
                                                    on_internal_link.call(target);
                                                }
                                                Key::Character(ref value) if value == " " => {
                                                    evt.prevent_default();
                                                    evt.stop_propagation();
                                                    on_internal_link.call(target);
                                                }
                                                _ => {}
                                            }
                                        },
                                    }
                                }
                            }
                            NavigationTarget::ExternalUri(uri) => {
                                let raw_uri = uri.raw_uri.clone();
                                rsx! {
                                button {
                                    key: "{key}",
                                    class: "zoom-link-overlay external",
                                    title: t(locale(), MessageKey::ZoomLinkExternalDisabled),
                                    "aria-label": t(locale(), MessageKey::ZoomLinkExternalDisabled),
                                    style: "left:{lx}px; top:{ly}px; width:{lw}px; height:{lh}px;",
                                    onkeydown: move |evt: Event<KeyboardData>| {
                                        match evt.key() {
                                            Key::Enter => {
                                                evt.stop_propagation();
                                                let dialog_id = *external_uri_dialog_seq.peek() + 1;
                                                external_uri_dialog_seq.set(dialog_id);
                                                external_uri_dialog.set(Some(ExternalUriDialogState {
                                                    raw_uri: raw_uri.clone(),
                                                    copy_status: ExternalUriCopyStatus::Idle,
                                                    dialog_id,
                                                }));
                                            }
                                            Key::Character(ref value) if value == " " => {
                                                evt.prevent_default();
                                                evt.stop_propagation();
                                                let dialog_id = *external_uri_dialog_seq.peek() + 1;
                                                external_uri_dialog_seq.set(dialog_id);
                                                external_uri_dialog.set(Some(ExternalUriDialogState {
                                                    raw_uri: raw_uri.clone(),
                                                    copy_status: ExternalUriCopyStatus::Idle,
                                                    dialog_id,
                                                }));
                                            }
                                            _ => {}
                                        }
                                    },
                                }
                                }
                            }
                            NavigationTarget::Disabled(_) => rsx! {
                                span {
                                    key: "{key}",
                                    class: "zoom-link-overlay disabled",
                                    title: t(locale(), MessageKey::ZoomLinkDisabled),
                                    "aria-label": t(locale(), MessageKey::ZoomLinkDisabled),
                                    style: "left:{lx}px; top:{ly}px; width:{lw}px; height:{lh}px;",
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Mouse-up handler of the image wrapper: activates the link under the pointer
/// unless the pointer moved farther than the click threshold (text selection).
pub(super) fn image_mouseup_handler(
    page_link_rects: Vec<util::ZoomPageLinkRect>,
    page_count: usize,
    on_internal_link: Callback<PageIndex>,
    mut link_pointer_down: Signal<Option<(f64, f64)>>,
    mut external_uri_dialog_seq: Signal<u64>,
    mut external_uri_dialog: Signal<Option<ExternalUriDialogState>>,
) -> impl FnMut(Event<MouseData>) {
    move |evt: Event<MouseData>| {
        let point = evt.client_coordinates();
        let end = (point.x, point.y);
        let Some(start) = *link_pointer_down.peek() else {
            return;
        };
        link_pointer_down.set(None);
        if util::movement_exceeds_click_threshold(start, end, LINK_CLICK_DRAG_THRESHOLD_PX) {
            return;
        }
        evt.stop_propagation();
        let on_internal_link = on_internal_link;
        let link_rects = page_link_rects.clone();
        spawn(async move {
            let script = "const el = document.getElementById('zoom-image-wrap');\
                 if (!el) return null;\
                 const r = el.getBoundingClientRect();\
                 return [r.left, r.top];";
            let Ok(Some(rect_origin)) = eval(script).join::<Option<Vec<f64>>>().await else {
                return;
            };
            if rect_origin.len() != 2 {
                return;
            }
            let relative = util::client_point_relative_to_rect(
                (end.0, end.1),
                (rect_origin[0], rect_origin[1]),
            );
            if let Some(activation) =
                util::link_activation_at(&link_rects, relative.0, relative.1, page_count)
            {
                match activation {
                    util::ZoomLinkActivation::Internal(target) => {
                        on_internal_link.call(target);
                    }
                    util::ZoomLinkActivation::ExternalUri(raw_uri) => {
                        let dialog_id = *external_uri_dialog_seq.peek() + 1;
                        external_uri_dialog_seq.set(dialog_id);
                        external_uri_dialog.set(Some(ExternalUriDialogState {
                            raw_uri,
                            copy_status: ExternalUriCopyStatus::Idle,
                            dialog_id,
                        }));
                    }
                }
            }
        });
    }
}
