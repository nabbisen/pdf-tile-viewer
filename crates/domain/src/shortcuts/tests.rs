use super::*;
use Dismissal as D;
use KeyDescriptor::*;
use ShortcutAction as A;

const ALL_LAYERS: [Layer; 5] = [
    Layer::Base,
    Layer::Outline,
    Layer::Search,
    Layer::Zoom,
    Layer::Dialog,
];

fn ctx(top_layer: Layer, zen: bool) -> ShortcutContext {
    ShortcutContext { top_layer, zen }
}

fn r(top_layer: Layer, zen: bool, key: KeyDescriptor) -> Option<ShortcutAction> {
    resolve_with(ctx(top_layer, zen), &key, true)
}

/// Every key the matrix tests cover.
fn keys() -> Vec<KeyDescriptor> {
    let mut keys = vec![
        Escape, ArrowLeft, ArrowRight, PageUp, PageDown, Home, End, Other,
    ];
    keys.extend("zZ+=-0aq1 ".chars().map(Character));
    keys
}

#[test]
fn product_constant_keeps_the_documented_idle_escape_for_2_0_x() {
    // Compile-time check: flipping the constant (2.1.0) must be a deliberate edit of this line too.
    const { assert!(ESCAPE_LEAVES_DOCUMENT_WHEN_IDLE) };
    assert_eq!(
        resolve(ctx(Layer::Base, false), &Escape),
        Some(A::Dismiss(D::LeaveDocument))
    );
}

#[test]
fn base_shortcuts_resolve_in_base_search_and_outline() {
    for layer in [Layer::Base, Layer::Search, Layer::Outline] {
        for zen in [false, true] {
            assert_eq!(r(layer, zen, Character('z')), Some(A::ToggleZen));
            assert_eq!(r(layer, zen, Character('Z')), Some(A::ToggleZen));
            assert_eq!(r(layer, zen, Character('+')), Some(A::ScaleUp));
            assert_eq!(r(layer, zen, Character('=')), Some(A::ScaleUp));
            assert_eq!(r(layer, zen, Character('-')), Some(A::ScaleDown));
            assert_eq!(r(layer, zen, Character('0')), Some(A::ResetScale));
        }
    }
}

#[test]
fn no_base_shortcut_resolves_while_a_dialog_or_zoom_is_on_top() {
    let base = [A::ToggleZen, A::ScaleUp, A::ScaleDown, A::ResetScale];
    for layer in [Layer::Zoom, Layer::Dialog] {
        for zen in [false, true] {
            for key in keys() {
                if let Some(action) = r(layer, zen, key) {
                    assert!(
                        !base.contains(&action),
                        "{layer:?} zen={zen} {key:?} resolved to base action {action:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn only_dismiss_resolves_while_a_dialog_is_on_top() {
    for zen in [false, true] {
        for key in keys() {
            let got = r(Layer::Dialog, zen, key);
            if key == Escape {
                assert_eq!(got, Some(A::Dismiss(D::Dialog)));
            } else {
                assert_eq!(got, None, "{key:?} must not resolve inside a dialog");
            }
        }
    }
}

#[test]
fn zoom_layer_keys() {
    let z = |key| r(Layer::Zoom, false, key);
    assert_eq!(z(ArrowLeft), Some(A::ZoomPrev));
    assert_eq!(z(PageUp), Some(A::ZoomPrev));
    assert_eq!(z(ArrowRight), Some(A::ZoomNext));
    assert_eq!(z(PageDown), Some(A::ZoomNext));
    assert_eq!(z(Home), Some(A::ZoomFirst));
    assert_eq!(z(End), Some(A::ZoomLast));
    assert_eq!(z(Character('+')), Some(A::ZoomScaleUp));
    assert_eq!(z(Character('=')), Some(A::ZoomScaleUp));
    assert_eq!(z(Character('-')), Some(A::ZoomScaleDown));
    assert_eq!(z(Escape), Some(A::Dismiss(D::Zoom)));
    // keys that mean something in the grid mean nothing here
    assert_eq!(z(Character('z')), None);
    assert_eq!(z(Character('0')), None);
}

#[test]
fn zoom_layer_actions_never_resolve_outside_the_zoom_layer() {
    let zoom_only = [
        A::ZoomScaleUp,
        A::ZoomScaleDown,
        A::ZoomPrev,
        A::ZoomNext,
        A::ZoomFirst,
        A::ZoomLast,
    ];
    for layer in [Layer::Base, Layer::Outline, Layer::Search, Layer::Dialog] {
        for zen in [false, true] {
            for key in keys() {
                if let Some(action) = r(layer, zen, key) {
                    assert!(
                        !zoom_only.contains(&action),
                        "{layer:?} {key:?} resolved to zoom action {action:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn escape_closes_the_top_most_layer_in_the_documented_order() {
    // Dialog → Zoom → Search → Outline → Zen → (idle).
    assert_eq!(r(Layer::Dialog, false, Escape), Some(A::Dismiss(D::Dialog)));
    assert_eq!(r(Layer::Zoom, false, Escape), Some(A::Dismiss(D::Zoom)));
    assert_eq!(r(Layer::Search, false, Escape), Some(A::Dismiss(D::Search)));
    assert_eq!(
        r(Layer::Outline, false, Escape),
        Some(A::Dismiss(D::Outline))
    );
    assert_eq!(r(Layer::Base, true, Escape), Some(A::Dismiss(D::Zen)));
    assert_eq!(
        r(Layer::Base, false, Escape),
        Some(A::Dismiss(D::LeaveDocument))
    );
}

#[test]
fn layers_are_ordered_from_base_to_dialog() {
    let mut sorted = ALL_LAYERS;
    sorted.sort();
    assert_eq!(sorted, ALL_LAYERS);
}

#[test]
fn panels_are_not_dismissal_candidates_in_zen_mode() {
    // "Zen with the search flag set": the panels are not rendered, so Escape leaves Zen first.
    assert_eq!(r(Layer::Search, true, Escape), Some(A::Dismiss(D::Zen)));
    assert_eq!(r(Layer::Outline, true, Escape), Some(A::Dismiss(D::Zen)));
    assert_eq!(ctx(Layer::Search, true).effective_layer(), Layer::Base);
    // but zoom and dialog stay on top of zen
    assert_eq!(r(Layer::Zoom, true, Escape), Some(A::Dismiss(D::Zoom)));
    assert_eq!(r(Layer::Dialog, true, Escape), Some(A::Dismiss(D::Dialog)));
}

#[test]
fn one_press_resolves_to_exactly_one_dismissal_per_context() {
    for layer in ALL_LAYERS {
        for zen in [false, true] {
            assert!(
                matches!(r(layer, zen, Escape), Some(A::Dismiss(_))),
                "{layer:?} zen={zen}"
            );
        }
    }
}

#[test]
fn idle_escape_follows_the_constant_in_both_values() {
    let idle = ctx(Layer::Base, false);
    // 2.0.x: back to the dashboard
    assert_eq!(
        resolve_with(idle, &Escape, true),
        Some(A::Dismiss(D::LeaveDocument))
    );
    // 2.1.0: nothing (exercised here only; the product constant is `true`)
    assert_eq!(
        resolve_with(idle, &Escape, false),
        Some(A::Dismiss(D::Nothing))
    );
}

#[test]
fn the_idle_flag_only_matters_when_nothing_is_open() {
    for flag in [true, false] {
        for (layer, zen, expected) in [
            (Layer::Dialog, false, D::Dialog),
            (Layer::Zoom, false, D::Zoom),
            (Layer::Search, false, D::Search),
            (Layer::Outline, false, D::Outline),
            (Layer::Base, true, D::Zen),
        ] {
            assert_eq!(
                resolve_with(ctx(layer, zen), &Escape, flag),
                Some(A::Dismiss(expected)),
                "{layer:?} zen={zen} flag={flag}"
            );
        }
    }
}

#[test]
fn unrelated_keys_resolve_to_nothing_everywhere() {
    for layer in ALL_LAYERS {
        for zen in [false, true] {
            for key in [
                Character('a'),
                Character('q'),
                Character('1'),
                Character(' '),
                Other,
            ] {
                assert_eq!(r(layer, zen, key), None, "{layer:?} zen={zen} {key:?}");
            }
        }
    }
}

// ── Resolver ↔ documentation (docs/src/intermediate/shortcuts.md) ────────────

/// The documented shortcut table, mirrored: `label` is the exact first cell of the row in
/// `shortcuts.md`; `probes` are the (context, key, expected action) triples the row promises.
struct Row {
    section: &'static str,
    label: &'static str,
    probes: &'static [(Layer, bool, KeyDescriptor, ShortcutAction)],
}

const DOCUMENTED: &[Row] = &[
    Row {
        section: "Tile grid",
        label: "`Z`",
        probes: &[(Layer::Base, false, Character('z'), A::ToggleZen)],
    },
    Row {
        section: "Tile grid",
        label: "`+` or `=`",
        probes: &[
            (Layer::Base, false, Character('+'), A::ScaleUp),
            (Layer::Base, false, Character('='), A::ScaleUp),
        ],
    },
    Row {
        section: "Tile grid",
        label: "`-`",
        probes: &[(Layer::Base, false, Character('-'), A::ScaleDown)],
    },
    Row {
        section: "Tile grid",
        label: "`0`",
        probes: &[(Layer::Base, false, Character('0'), A::ResetScale)],
    },
    Row {
        section: "Tile grid",
        label: "`Ctrl + Scroll`",
        probes: &[], // a pointer gesture, handled by the wheel handler, not the resolver
    },
    Row {
        section: "Zoom overlay",
        label: "`←` / `PageUp`",
        probes: &[
            (Layer::Zoom, false, ArrowLeft, A::ZoomPrev),
            (Layer::Zoom, false, PageUp, A::ZoomPrev),
        ],
    },
    Row {
        section: "Zoom overlay",
        label: "`→` / `PageDown`",
        probes: &[
            (Layer::Zoom, false, ArrowRight, A::ZoomNext),
            (Layer::Zoom, false, PageDown, A::ZoomNext),
        ],
    },
    Row {
        section: "Zoom overlay",
        label: "`Home`",
        probes: &[(Layer::Zoom, false, Home, A::ZoomFirst)],
    },
    Row {
        section: "Zoom overlay",
        label: "`End`",
        probes: &[(Layer::Zoom, false, End, A::ZoomLast)],
    },
    Row {
        section: "Zoom overlay",
        label: "`+` / `=`",
        probes: &[
            (Layer::Zoom, false, Character('+'), A::ZoomScaleUp),
            (Layer::Zoom, false, Character('='), A::ZoomScaleUp),
        ],
    },
    Row {
        section: "Zoom overlay",
        label: "`-`",
        probes: &[(Layer::Zoom, false, Character('-'), A::ZoomScaleDown)],
    },
    Row {
        section: "Escape closes one layer",
        label: "External-link dialog or password prompt",
        probes: &[(Layer::Dialog, false, Escape, A::Dismiss(D::Dialog))],
    },
    Row {
        section: "Escape closes one layer",
        label: "Zoom overlay",
        probes: &[(Layer::Zoom, false, Escape, A::Dismiss(D::Zoom))],
    },
    Row {
        section: "Escape closes one layer",
        label: "Search panel",
        probes: &[(Layer::Search, false, Escape, A::Dismiss(D::Search))],
    },
    Row {
        section: "Escape closes one layer",
        label: "Outline panel",
        probes: &[(Layer::Outline, false, Escape, A::Dismiss(D::Outline))],
    },
    Row {
        section: "Escape closes one layer",
        label: "Zen mode",
        probes: &[
            (Layer::Base, true, Escape, A::Dismiss(D::Zen)),
            (Layer::Search, true, Escape, A::Dismiss(D::Zen)),
        ],
    },
    Row {
        section: "Escape closes one layer",
        label: "Nothing open",
        probes: &[(Layer::Base, false, Escape, A::Dismiss(D::LeaveDocument))],
    },
];

const SHORTCUTS_MD: &str = include_str!("../../../../docs/src/intermediate/shortcuts.md");

/// `(section, first cell)` of every table row of the document (header and separator rows skipped).
fn documented_rows() -> Vec<(String, String)> {
    let mut section = String::new();
    let mut rows = Vec::new();
    let mut table_line = 0;
    for line in SHORTCUTS_MD.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            section = title.trim().to_string();
            table_line = 0;
        } else if line.starts_with('|') {
            table_line += 1;
            if table_line > 2 {
                let first = line.trim_start_matches('|').split('|').next().unwrap_or("");
                rows.push((section.clone(), first.trim().to_string()));
            }
        } else {
            table_line = 0;
        }
    }
    rows
}

#[test]
fn every_documented_shortcut_row_is_mirrored_and_every_mirrored_row_is_documented() {
    let documented = documented_rows();
    for (section, label) in &documented {
        if section == "Text fields" {
            continue; // fields are not resolver contexts; covered by the editable-field helper tests
        }
        assert!(
            DOCUMENTED
                .iter()
                .any(|row| row.section == section && row.label == label),
            "documented row [{section}] {label} has no probe in the resolver test table"
        );
    }
    for row in DOCUMENTED {
        assert!(
            documented
                .iter()
                .any(|(s, l)| s == row.section && l == row.label),
            "probe row [{}] {} is not in shortcuts.md",
            row.section,
            row.label
        );
    }
}

#[test]
fn every_documented_shortcut_resolves_to_its_documented_action() {
    for row in DOCUMENTED {
        for (layer, zen, key, expected) in row.probes {
            assert_eq!(
                r(*layer, *zen, *key),
                Some(*expected),
                "[{}] {}: {layer:?} zen={zen} {key:?}",
                row.section,
                row.label
            );
        }
    }
}
