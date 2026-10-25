use super::*;
use crate::i18n::MessageKey;

fn ready(bucket: u16) -> TileImageState {
    TileImageState::Ready {
        uri: "data:image/png;base64,".to_string(),
        bucket: ScaleBucket(bucket),
    }
}

const WANTED: ScaleBucket = ScaleBucket(200);

#[test]
fn a_missing_tile_needs_a_render() {
    assert!(tile_needs_render(None, WANTED));
}

#[test]
fn pending_rendering_and_failed_tiles_need_a_render() {
    assert!(tile_needs_render(Some(&TileImageState::Pending), WANTED));
    assert!(tile_needs_render(
        Some(&TileImageState::Rendering(3)),
        WANTED
    ));
    assert!(tile_needs_render(
        Some(&TileImageState::Failed(MessageKey::ErrRenderFailed)),
        WANTED
    ));
}

#[test]
fn a_ready_tile_of_the_wanted_bucket_is_satisfied() {
    assert!(!tile_needs_render(Some(&ready(200)), WANTED));
}

#[test]
fn a_ready_tile_of_another_bucket_is_stale_and_needs_a_render() {
    assert!(tile_needs_render(Some(&ready(100)), WANTED), "lower bucket");
    assert!(
        tile_needs_render(Some(&ready(300)), WANTED),
        "higher bucket"
    );
}

#[test]
fn the_scheduler_decision_does_not_touch_the_state() {
    // The old image must keep showing while the new one renders: the decision is a pure
    // read, and a stale `Ready` stays `Ready` (checked through the same value).
    let state = ready(100);
    assert!(tile_needs_render(Some(&state), WANTED));
    assert_eq!(state, ready(100));
}

// ── apply_retention (RFC 040) ─────────────────────────────────────────────────

fn states(entries: &[(usize, TileImageState)]) -> TileImages {
    entries
        .iter()
        .map(|(page, state)| (PageIndex(*page), state.clone()))
        .collect()
}

#[test]
fn victims_in_the_ready_state_return_to_pending() {
    let mut images = states(&[(0, ready(100)), (1, ready(100)), (2, ready(100))]);
    assert!(apply_retention(&mut images, &[PageIndex(0), PageIndex(2)]));
    assert_eq!(images[&PageIndex(0)], TileImageState::Pending);
    assert_eq!(images[&PageIndex(1)], ready(100), "not a victim");
    assert_eq!(images[&PageIndex(2)], TileImageState::Pending);
}

#[test]
fn rendering_pending_and_failed_tiles_are_untouched() {
    let failed = TileImageState::Failed(MessageKey::ErrRenderFailed);
    let mut images = states(&[
        (0, TileImageState::Rendering(4)),
        (1, TileImageState::Pending),
        (2, failed.clone()),
    ]);
    assert!(!apply_retention(
        &mut images,
        &[PageIndex(0), PageIndex(1), PageIndex(2)]
    ));
    assert_eq!(images[&PageIndex(0)], TileImageState::Rendering(4));
    assert_eq!(images[&PageIndex(1)], TileImageState::Pending);
    assert_eq!(images[&PageIndex(2)], failed);
}

#[test]
fn stale_scale_ready_tiles_are_released_like_any_other() {
    // Retention looks at distance only, not at the bucket (RFC 040 § 4.3).
    let mut images = states(&[(0, ready(100))]);
    assert!(apply_retention(&mut images, &[PageIndex(0)]));
    assert_eq!(images[&PageIndex(0)], TileImageState::Pending);
}

#[test]
fn unknown_victims_and_an_empty_plan_change_nothing() {
    let mut images = states(&[(0, ready(100))]);
    assert!(!apply_retention(&mut images, &[PageIndex(9)]));
    assert!(!apply_retention(&mut images, &[]));
    assert_eq!(images[&PageIndex(0)], ready(100));
}

#[test]
fn a_released_tile_needs_a_render_when_it_comes_back() {
    let mut images = states(&[(0, ready(200))]);
    apply_retention(&mut images, &[PageIndex(0)]);
    assert!(tile_needs_render(images.get(&PageIndex(0)), WANTED));
}
