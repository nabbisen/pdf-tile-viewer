use crate::document::{DocumentGeneration, DocumentId, PageIndex};
use crate::navigation::*;

#[test]
fn default_navigation_limits_match_rfc_026() {
    let limits = NavigationResourceLimits::default();

    assert_eq!(limits.max_outline_nodes, 4096);
    assert_eq!(limits.max_outline_depth, 16);
    assert_eq!(limits.max_bookmark_title_chars, 512);
    assert_eq!(limits.max_bookmark_title_bytes, 2048);
    assert_eq!(limits.max_links_per_page, 512);
    assert_eq!(limits.max_uri_bytes, 2048);
    assert_eq!(limits.max_cached_link_pages, 8);
    assert_eq!(limits.max_cached_link_bytes, 2 * 1024 * 1024);
    assert_eq!(limits.max_outline_string_bytes, 1024 * 1024);
}

#[test]
fn page_links_request_produces_cache_key() {
    let request = PageLinksRequest {
        document_id: DocumentId(7),
        generation: DocumentGeneration(11),
        page_index: PageIndex(2),
        limits: NavigationResourceLimits::default(),
    };

    assert_eq!(
        request.cache_key(),
        PageLinksCacheKey {
            document_id: DocumentId(7),
            generation: DocumentGeneration(11),
            page_index: PageIndex(2),
        }
    );
}

#[test]
fn outline_title_preserves_missing_and_empty_states() {
    let limits = NavigationResourceLimits::default();

    assert_eq!(
        OutlineTitle::from_pdf_title(None, &limits),
        OutlineTitle::Missing
    );
    assert_eq!(
        OutlineTitle::from_pdf_title(Some(String::new()), &limits),
        OutlineTitle::Empty
    );
}

#[test]
fn outline_title_truncates_by_character_limit() {
    let limits = NavigationResourceLimits {
        max_bookmark_title_chars: 3,
        max_bookmark_title_bytes: 64,
        ..NavigationResourceLimits::default()
    };

    assert_eq!(
        OutlineTitle::from_pdf_title(Some("abcdef".to_string()), &limits),
        OutlineTitle::Truncated("abc".to_string())
    );
}

#[test]
fn outline_title_truncates_on_utf8_boundary_by_byte_limit() {
    let limits = NavigationResourceLimits {
        max_bookmark_title_chars: 10,
        max_bookmark_title_bytes: 5,
        ..NavigationResourceLimits::default()
    };

    assert_eq!(
        OutlineTitle::from_pdf_title(Some("ééé".to_string()), &limits),
        OutlineTitle::Truncated("éé".to_string())
    );
}

#[test]
fn external_uri_target_keeps_raw_uri_without_authorizing_open() {
    let limits = NavigationResourceLimits::default();

    assert_eq!(
        external_uri_target("https://example.com/path".to_string(), &limits),
        NavigationTarget::ExternalUri(ExternalUriTarget {
            raw_uri: "https://example.com/path".to_string(),
        })
    );
}

#[test]
fn external_uri_target_rejects_empty_and_over_limit_uri() {
    let limits = NavigationResourceLimits {
        max_uri_bytes: 4,
        ..NavigationResourceLimits::default()
    };

    assert_eq!(
        external_uri_target(String::new(), &limits),
        NavigationTarget::Disabled(DisabledNavigationReason::InvalidExternalUri)
    );
    assert_eq!(
        external_uri_target("https".to_string(), &limits),
        NavigationTarget::Disabled(DisabledNavigationReason::ExternalUriTooLong)
    );
}

#[test]
fn navigation_limit_status_reports_any_truncation() {
    assert!(!NavigationLimitStatus::default().any_truncated());
    assert!(
        NavigationLimitStatus {
            strings_truncated: true,
            ..NavigationLimitStatus::default()
        }
        .any_truncated()
    );
}
