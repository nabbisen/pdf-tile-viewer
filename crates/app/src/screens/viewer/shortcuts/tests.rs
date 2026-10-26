use super::*;

#[test]
fn the_top_layer_is_zoom_then_search_then_outline_then_base() {
    let flags = |zoom_open, search_open, outline_open| LayerFlags {
        zoom_open,
        search_open,
        outline_open,
    };
    assert_eq!(top_layer(flags(true, true, true)), Layer::Zoom);
    assert_eq!(top_layer(flags(false, true, true)), Layer::Search);
    assert_eq!(top_layer(flags(false, false, true)), Layer::Outline);
    assert_eq!(top_layer(flags(false, false, false)), Layer::Base);
    assert_eq!(top_layer(LayerFlags::default()), Layer::Base);
}
