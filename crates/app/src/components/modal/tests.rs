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
