use crate::geometry::PageRect;

#[test]
fn edges_are_top_left_origin_with_y_down() {
    let r = PageRect {
        x: 10.0,
        y: 20.0,
        width: 30.0,
        height: 5.0,
    };
    assert_eq!(r.right(), 40.0);
    assert_eq!(r.bottom(), 25.0);
}
