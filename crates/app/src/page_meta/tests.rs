use super::*;

#[test]
fn language_follows_the_locale() {
    assert_eq!(lang_tag(Locale::En), "en");
    assert_eq!(lang_tag(Locale::Ja), "ja");
}

#[test]
fn title_is_the_app_title_or_file_and_app_title() {
    assert_eq!(title_text("PDF Tile Viewer", None), "PDF Tile Viewer");
    assert_eq!(
        title_text("PDF Tile Viewer", Some("report.pdf")),
        "report.pdf — PDF Tile Viewer"
    );
    assert_eq!(
        title_text("PDF タイルビューア", Some("報告書.pdf")),
        "報告書.pdf — PDF タイルビューア"
    );
}

#[test]
fn the_script_quotes_both_values() {
    let script = meta_script("ja", "a\"b\\c</script>.pdf — App");
    assert!(script.starts_with("document.documentElement.lang = \"ja\";"));
    assert!(script.contains("document.title = \"a\\\"b\\\\c</script>.pdf — App\";"));
}

#[test]
fn the_script_has_no_logic_only_two_assignments() {
    let script = meta_script("en", "T");
    assert_eq!(script.matches('=').count(), 2);
    for word in ["if", "function", "=>", "for ", "while"] {
        assert!(!script.contains(word), "{word}");
    }
}
