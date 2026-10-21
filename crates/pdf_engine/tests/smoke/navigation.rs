//! Engine smoke tests: navigation.

use super::*;
use domain::document::PageIndex;
use domain::navigation::{
    DestinationView, DisabledNavigationReason, DocumentOutlineRequest, NavigationResourceLimits,
    NavigationTarget, OutlineTitle, PageLinksRequest,
};

#[test]
fn navigation_outline_extracts_roots_and_nested_entries() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("navigation-links-outline.pdf")))
        .unwrap()
        .unwrap();

    let outline = block_on(engine.extract_document_outline(DocumentOutlineRequest {
        document_id: session.id,
        generation: session.generation,
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap()
    .unwrap();

    assert_eq!(outline.document_id, session.id);
    assert_eq!(outline.generation, session.generation);
    assert_eq!(outline.roots.len(), 3);
    assert_eq!(
        outline.roots[0].title,
        OutlineTitle::Present("Chapter 1".to_string())
    );
    assert_eq!(outline.roots[0].children.len(), 1);
    assert_eq!(outline.roots[0].children[0].title, OutlineTitle::Missing);
    assert_eq!(
        outline.roots[1].title,
        OutlineTitle::Present("Chapter 2".to_string())
    );
    assert_eq!(
        outline.roots[2].title,
        OutlineTitle::Present("Empty URI".to_string())
    );
    assert!(matches!(
        &outline.roots[2].target,
        NavigationTarget::Disabled(DisabledNavigationReason::InvalidExternalUri)
    ));

    match &outline.roots[1].target {
        NavigationTarget::InternalDestination(destination) => {
            assert_eq!(destination.page_index, PageIndex(2));
            assert!(matches!(
                destination.view,
                DestinationView::CoordinatesAndZoom { .. }
            ));
        }
        other => panic!("expected internal destination, got {other:?}"),
    }
}

#[test]
fn navigation_page_links_extract_internal_uri_and_disabled_actions() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("navigation-links-outline.pdf")))
        .unwrap()
        .unwrap();

    let page_one_links = block_on(engine.extract_page_links(PageLinksRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap()
    .unwrap();

    assert_eq!(page_one_links.links.len(), 2);
    assert!(
        page_one_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::InternalDestination(destination)
                if destination.page_index == PageIndex(2)
        )),
        "page 1 should include an internal link to page 3"
    );
    assert!(
        page_one_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::ExternalUri(uri)
                if uri.raw_uri == "https://example.com/pdf-tile-viewer"
        )),
        "page 1 should include the https URI"
    );

    let page_two_links = block_on(engine.extract_page_links(PageLinksRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(1),
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap()
    .unwrap();

    assert_eq!(page_two_links.links.len(), 3);
    assert!(
        page_two_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::ExternalUri(uri)
                if uri.raw_uri == "file:///tmp/pdf-tile-viewer-blocked"
        )),
        "PDFium extraction stores raw file URI but does not authorize opening"
    );
    assert!(
        page_two_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::ExternalUri(uri) if uri.raw_uri == "relative/path"
        )),
        "relative URI remains raw metadata for app_services policy rejection"
    );
    assert!(
        page_two_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::Disabled(DisabledNavigationReason::LaunchAction)
        )),
        "launch action must not be executable"
    );

    let page_three_links = block_on(engine.extract_page_links(PageLinksRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(2),
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap()
    .unwrap();

    assert_eq!(page_three_links.links.len(), 4);
    assert!(
        page_three_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::InternalDestination(destination)
                if destination.page_index == PageIndex(0)
        )),
        "page 3 should include an internal link back to page 1"
    );
    assert!(
        page_three_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::Disabled(DisabledNavigationReason::RemoteDestination)
        )),
        "remote document action must not become internal navigation"
    );
    assert!(
        page_three_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::Disabled(DisabledNavigationReason::EmbeddedDestination)
        )),
        "embedded document action must not become internal navigation"
    );
    assert!(
        page_three_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::Disabled(DisabledNavigationReason::UnsupportedAction)
        )),
        "JavaScript action must not be executable"
    );
}

#[test]
fn navigation_rejects_stale_generation_and_enforces_page_link_cap() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("navigation-links-outline.pdf")))
        .unwrap()
        .unwrap();

    let stale = block_on(engine.extract_document_outline(DocumentOutlineRequest {
        document_id: session.id,
        generation: domain::document::DocumentGeneration(session.generation.0 + 999),
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap();
    assert_eq!(
        stale,
        Err(domain::navigation::NavigationError::DocumentNotOpen)
    );

    let links = block_on(engine.extract_page_links(PageLinksRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(1),
        limits: NavigationResourceLimits {
            max_links_per_page: 1,
            ..NavigationResourceLimits::default()
        },
    }))
    .unwrap()
    .unwrap();

    assert_eq!(links.links.len(), 1);
    assert!(links.limit_status.links_truncated);
}

#[test]
fn navigation_outline_enforces_node_depth_and_string_payload_caps() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("navigation-links-outline.pdf")))
        .unwrap()
        .unwrap();

    let node_limited = block_on(engine.extract_document_outline(DocumentOutlineRequest {
        document_id: session.id,
        generation: session.generation,
        limits: NavigationResourceLimits {
            max_outline_nodes: 1,
            ..NavigationResourceLimits::default()
        },
    }))
    .unwrap()
    .unwrap();
    assert_eq!(node_limited.roots.len(), 1);
    assert!(node_limited.limit_status.outline_truncated);

    let depth_limited = block_on(engine.extract_document_outline(DocumentOutlineRequest {
        document_id: session.id,
        generation: session.generation,
        limits: NavigationResourceLimits {
            max_outline_depth: 1,
            ..NavigationResourceLimits::default()
        },
    }))
    .unwrap()
    .unwrap();
    assert!(
        depth_limited.roots[0].children.is_empty(),
        "nested outline entries should be omitted after depth cap"
    );
    assert!(depth_limited.limit_status.outline_truncated);

    let string_limited = block_on(engine.extract_document_outline(DocumentOutlineRequest {
        document_id: session.id,
        generation: session.generation,
        limits: NavigationResourceLimits {
            max_outline_string_bytes: 3,
            ..NavigationResourceLimits::default()
        },
    }))
    .unwrap()
    .unwrap();
    assert!(matches!(
        &string_limited.roots[0].title,
        OutlineTitle::Truncated(value) if value == "Cha"
    ));
    assert!(string_limited.limit_status.outline_truncated);
    assert!(string_limited.limit_status.strings_truncated);
}
