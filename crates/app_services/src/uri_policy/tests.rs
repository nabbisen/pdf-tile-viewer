use domain::navigation::NavigationResourceLimits;

use crate::uri_policy::{
    ExternalUriOpenDecision, ExternalUriRejectReason, validate_external_uri_for_open,
};

fn limits() -> NavigationResourceLimits {
    NavigationResourceLimits::default()
}

#[test]
fn allows_https_http_and_mailto_with_lowercase_scheme_normalization() {
    assert_eq!(
        validate_external_uri_for_open(" HTTPS://example.com/path ", &limits()),
        ExternalUriOpenDecision::Allowed {
            normalized_uri: "https://example.com/path".to_string(),
        }
    );
    assert_eq!(
        validate_external_uri_for_open("http://example.com", &limits()),
        ExternalUriOpenDecision::Allowed {
            normalized_uri: "http://example.com".to_string(),
        }
    );
    assert_eq!(
        validate_external_uri_for_open("MAILTO:user@example.com", &limits()),
        ExternalUriOpenDecision::Allowed {
            normalized_uri: "mailto:user@example.com".to_string(),
        }
    );
}

#[test]
fn rejects_file_and_custom_schemes() {
    assert_eq!(
        validate_external_uri_for_open("file:///tmp/blocked.pdf", &limits()),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::DisallowedScheme)
    );
    assert_eq!(
        validate_external_uri_for_open("vscode://file/tmp/blocked", &limits()),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::DisallowedScheme)
    );
}

#[test]
fn rejects_relative_empty_and_scheme_less_strings() {
    assert_eq!(
        validate_external_uri_for_open("", &limits()),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::Empty)
    );
    assert_eq!(
        validate_external_uri_for_open("docs/page.html", &limits()),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::RelativeOrSchemeLess)
    );
    assert_eq!(
        validate_external_uri_for_open("//example.com/no-scheme", &limits()),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::RelativeOrSchemeLess)
    );
}

#[test]
fn rejects_control_characters_and_interior_whitespace() {
    assert_eq!(
        validate_external_uri_for_open("https://example.com/\nblocked", &limits()),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::ContainsControlCharacter)
    );
    assert_eq!(
        validate_external_uri_for_open("https://example.com/a b", &limits()),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::ContainsWhitespace)
    );
}

#[test]
fn rejects_invalid_scheme_and_missing_authority() {
    assert_eq!(
        validate_external_uri_for_open("1https://example.com", &limits()),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::InvalidScheme)
    );
    assert_eq!(
        validate_external_uri_for_open("https:/example.com", &limits()),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::MissingAuthority)
    );
    assert_eq!(
        validate_external_uri_for_open("https:///path", &limits()),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::MissingAuthority)
    );
}

#[test]
fn rejects_empty_mailto_and_over_limit_uri() {
    assert_eq!(
        validate_external_uri_for_open("mailto:", &limits()),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::MissingMailtoAddress)
    );

    let short_limits = NavigationResourceLimits {
        max_uri_bytes: 8,
        ..NavigationResourceLimits::default()
    };
    assert_eq!(
        validate_external_uri_for_open("https://example.com", &short_limits),
        ExternalUriOpenDecision::Rejected(ExternalUriRejectReason::TooLong)
    );
}
