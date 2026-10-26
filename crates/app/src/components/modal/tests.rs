use super::*;

#[test]
fn ids_are_quoted_as_json_strings() {
    assert_eq!(js_str("zoom-overlay-root"), "\"zoom-overlay-root\"");
    assert_eq!(js_str("a\"b"), "\"a\\\"b\"");
    assert_eq!(js_str("</script>"), "\"</script>\"");
}

#[test]
fn helper_defines_the_contract_functions() {
    for needle in [
        "window.__ptvModal",
        "isolate",
        "release",
        "focusable",
        "'inert' in",
        "aria-hidden",
        "focusin",
    ] {
        assert!(HELPER.contains(needle), "helper script lacks {needle}");
    }
}

/// Every script the module sends to the page, with typical ids.
fn all_scripts() -> Vec<(&'static str, String)> {
    vec![
        ("helper", HELPER.to_string()),
        ("focus", focus_script("some-id")),
        ("trap forward", trap_script("root", false)),
        ("trap backward", trap_script("root", true)),
        ("open", open_script("root", "first")),
        ("close", close_script("root", &["first", "second"])),
    ]
}

#[test]
fn placeholders_are_all_replaced() {
    for (name, script) in all_scripts() {
        for placeholder in [
            "__ID__",
            "__ROOT__",
            "__FIRST__",
            "__FALLBACK__",
            "__BACKWARD__",
        ] {
            assert!(
                !script.contains(placeholder),
                "{name} still has {placeholder}"
            );
        }
    }
}

#[test]
fn brackets_are_balanced_in_every_script() {
    for (name, script) in all_scripts() {
        for (open, close) in [('{', '}'), ('(', ')'), ('[', ']')] {
            let o = script.matches(open).count();
            let c = script.matches(close).count();
            assert_eq!(o, c, "{name}: {o} '{open}' vs {c} '{close}'");
        }
    }
}

#[test]
fn scripts_keep_their_line_structure() {
    // A `\` line continuation in a Rust string would join the lines, and a `//` comment would then
    // comment out the rest of the script (this broke the first version of the focus rescue).
    for (name, script) in all_scripts() {
        let lines = script.lines().filter(|l| !l.trim().is_empty()).count();
        assert!(lines >= 3, "{name} was collapsed into {lines} line(s)");
        for line in script.lines() {
            if let Some(at) = line.find("//") {
                assert!(
                    line[..at].trim().is_empty(),
                    "{name}: a `//` comment follows code on one line: {line}"
                );
            }
        }
    }
}

#[test]
fn the_open_script_installs_the_focus_rescue_and_the_isolation() {
    let open = open_script("zoom-overlay-root", "zoom-close-button");
    for needle in [
        "MutationObserver",
        "attributeFilter",
        "isolate(root)",
        "\"zoom-close-button\"",
        "stacks[\"zoom-overlay-root\"]",
    ] {
        assert!(open.contains(needle), "open script lacks {needle}");
    }
    assert!(close_script("a", &["b"]).contains("watch.disconnect()"));
}

/// Not a check by itself: `PTV_DUMP_JS_DIR=<dir> cargo test -p app dump_scripts` writes every script
/// to a file so that `node --check` can parse it (syntax errors in a script fail silently in the page).
#[test]
fn dump_scripts() {
    let Some(dir) = std::env::var_os("PTV_DUMP_JS_DIR") else {
        return;
    };
    for (name, script) in all_scripts() {
        let file = std::path::Path::new(&dir).join(format!("{}.js", name.replace(' ', "-")));
        // wrapped in a function: the scripts use top-level `const` and `return`
        std::fs::write(file, format!("(function () {{\n{script}\n}})();\n")).unwrap();
    }
}

#[test]
fn the_close_script_lists_the_fallback_ids_in_order() {
    let script = close_script(
        "password-dialog-root",
        &["viewer-root", "dashboard-open-button"],
    );
    let first = script.find("\"viewer-root\"").expect("first id");
    let second = script.find("\"dashboard-open-button\"").expect("second id");
    assert!(
        first < second,
        "order matters: the first id that exists wins"
    );
    assert!(script.contains("[\"viewer-root\", \"dashboard-open-button\"]"));
}

#[test]
fn an_empty_fallback_list_is_a_valid_array() {
    assert!(close_script("r", &[]).contains("of []"));
}
