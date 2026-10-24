use super::*;

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
        Some(&TileImageState::Failed("RenderFailed".to_string())),
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
