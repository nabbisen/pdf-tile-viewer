use super::*;

const LETTER: PageBox = PageBox {
    x0: 0.0,
    y0: 0.0,
    x1: 612.0,
    y1: 792.0,
};
/// CropBox [50 100 562 692] (offset origin) → 512 × 592.
const CROP: PageBox = PageBox {
    x0: 50.0,
    y0: 100.0,
    x1: 562.0,
    y1: 692.0,
};

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

fn assert_rect(actual: PageRect, x: f32, y: f32, w: f32, h: f32) {
    assert!(
        close(actual.x, x)
            && close(actual.y, y)
            && close(actual.width, w)
            && close(actual.height, h),
        "got ({}, {}, {}, {}), expected ({x}, {y}, {w}, {h})",
        actual.x,
        actual.y,
        actual.width,
        actual.height
    );
}

/// The text rectangle observed in the RFC 034 experiment (user space, identical for
/// every /Rotate): left 73.8, bottom 699.7, width 166.2, height 17.6.
const TEXT: UserRect = UserRect {
    left: 73.8,
    bottom: 699.7,
    right: 240.0,
    top: 717.3,
};

#[test]
fn rotation_from_degrees_normalizes_and_rejects_non_multiples() {
    assert_eq!(Rotation::from_degrees(0), Rotation::R0);
    assert_eq!(Rotation::from_degrees(90), Rotation::R90);
    assert_eq!(Rotation::from_degrees(180), Rotation::R180);
    assert_eq!(Rotation::from_degrees(270), Rotation::R270);
    assert_eq!(Rotation::from_degrees(360), Rotation::R0);
    assert_eq!(Rotation::from_degrees(450), Rotation::R90);
    assert_eq!(Rotation::from_degrees(-90), Rotation::R270);
    assert_eq!(Rotation::from_degrees(45), Rotation::R0);
}

#[test]
fn display_size_swaps_only_for_quarter_turns() {
    assert_eq!(display_size(LETTER, Rotation::R0), (612.0, 792.0));
    assert_eq!(display_size(LETTER, Rotation::R90), (792.0, 612.0));
    assert_eq!(display_size(LETTER, Rotation::R180), (612.0, 792.0));
    assert_eq!(display_size(LETTER, Rotation::R270), (792.0, 612.0));
    assert_eq!(display_size(CROP, Rotation::R0), (512.0, 592.0));
    assert_eq!(display_size(CROP, Rotation::R90), (592.0, 512.0));
}

// Hand-computed corner mapping, page box (0,0,612,792): the four user-space corners of the
// unrotated page land on the four display corners of the rotated page.
#[test]
fn corner_mapping_for_every_rotation() {
    // (bottom-left, top-left, top-right, bottom-right) of the user-space page box
    let user = [(0.0, 0.0), (0.0, 792.0), (612.0, 792.0), (612.0, 0.0)];
    let expected: [(Rotation, [(f32, f32); 4]); 4] = [
        (
            Rotation::R0,
            [(0.0, 792.0), (0.0, 0.0), (612.0, 0.0), (612.0, 792.0)],
        ),
        // 90° clockwise: user bottom-left → display top-left.
        (
            Rotation::R90,
            [(0.0, 0.0), (792.0, 0.0), (792.0, 612.0), (0.0, 612.0)],
        ),
        // 180°: user bottom-left → display top-right.
        (
            Rotation::R180,
            [(612.0, 0.0), (612.0, 792.0), (0.0, 792.0), (0.0, 0.0)],
        ),
        // 270° clockwise: user bottom-left → display bottom-right.
        (
            Rotation::R270,
            [(792.0, 612.0), (0.0, 612.0), (0.0, 0.0), (792.0, 0.0)],
        ),
    ];
    for (rotation, want) in expected {
        for (i, &(ux, uy)) in user.iter().enumerate() {
            let (dx, dy) = point_to_display(LETTER, rotation, ux, uy);
            assert!(
                close(dx, want[i].0) && close(dy, want[i].1),
                "{rotation:?} corner {i}: got ({dx}, {dy}), want {:?}",
                want[i]
            );
        }
    }
}

#[test]
fn text_rect_on_each_rotation_matches_hand_computed_display_rect() {
    // r0: y = 792 - 717.3 = 74.7
    assert_rect(
        rect_to_display(LETTER, Rotation::R0, TEXT).unwrap(),
        73.8,
        74.7,
        166.2,
        17.6,
    );
    // r90: dx = uy - y0 (699.7..717.3), dy = ux - x0 (73.8..240)
    assert_rect(
        rect_to_display(LETTER, Rotation::R90, TEXT).unwrap(),
        699.7,
        73.8,
        17.6,
        166.2,
    );
    // r180: dx = x1 - ux (372..538.2), dy = uy - y0 (699.7..717.3)
    assert_rect(
        rect_to_display(LETTER, Rotation::R180, TEXT).unwrap(),
        372.0,
        699.7,
        166.2,
        17.6,
    );
    // r270: dx = y1 - uy (74.7..92.3), dy = x1 - ux (372..538.2)
    assert_rect(
        rect_to_display(LETTER, Rotation::R270, TEXT).unwrap(),
        74.7,
        372.0,
        17.6,
        166.2,
    );
}

#[test]
fn non_zero_crop_origin_is_subtracted() {
    // user point (100, 200) inside CropBox [50 100 562 692]
    let cases = [
        (Rotation::R0, (50.0, 492.0)),    // (ux-x0, y1-uy)
        (Rotation::R90, (100.0, 50.0)),   // (uy-y0, ux-x0)
        (Rotation::R180, (462.0, 100.0)), // (x1-ux, uy-y0)
        (Rotation::R270, (492.0, 462.0)), // (y1-uy, x1-ux)
    ];
    for (rotation, (ex, ey)) in cases {
        let (dx, dy) = point_to_display(CROP, rotation, 100.0, 200.0);
        assert!(
            close(dx, ex) && close(dy, ey),
            "{rotation:?}: ({dx}, {dy}) vs ({ex}, {ey})"
        );
    }
}

#[test]
fn rectangle_is_normalized_to_min_corner_and_absolute_size() {
    // Inverted input (right < left, top < bottom) gives the same rectangle.
    let inverted = UserRect {
        left: TEXT.right,
        bottom: TEXT.top,
        right: TEXT.left,
        top: TEXT.bottom,
    };
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        assert_eq!(
            rect_to_display(LETTER, rotation, inverted),
            rect_to_display(LETTER, rotation, TEXT)
        );
    }
}

#[test]
fn conversion_preserves_area_and_stays_inside_the_displayed_page() {
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        let (dw, dh) = display_size(CROP, rotation);
        let r = rect_to_display(
            CROP,
            rotation,
            UserRect {
                left: 60.0,
                bottom: 110.0,
                right: 550.0,
                top: 680.0,
            },
        )
        .unwrap();
        assert!(close(r.width * r.height, (550.0 - 60.0) * (680.0 - 110.0)));
        assert!(r.x >= -1e-3 && r.y >= -1e-3 && r.right() <= dw + 1e-3 && r.bottom() <= dh + 1e-3);
    }
}

#[test]
fn degenerate_and_non_finite_rectangles_are_rejected() {
    let zero_w = UserRect {
        right: TEXT.left,
        ..TEXT
    };
    let zero_h = UserRect {
        top: TEXT.bottom,
        ..TEXT
    };
    let nan = UserRect {
        left: f32::NAN,
        ..TEXT
    };
    let inf = UserRect {
        top: f32::INFINITY,
        ..TEXT
    };
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        for bad in [zero_w, zero_h, nan, inf] {
            assert_eq!(
                rect_to_display(LETTER, rotation, bad),
                None,
                "{rotation:?} {bad:?}"
            );
        }
    }
    let bad_box = PageBox {
        x1: f32::NAN,
        ..LETTER
    };
    assert_eq!(rect_to_display(bad_box, Rotation::R0, TEXT), None);
}

#[test]
fn geometry_is_consistent_when_reported_size_matches() {
    let g = PageGeometry::new(LETTER, Rotation::R90, (792.0, 612.0));
    assert!(g.is_consistent());
    assert!(g.rect_to_display(TEXT).is_some());
    // within the 0.5 pt tolerance
    assert!(PageGeometry::new(LETTER, Rotation::R0, (612.4, 791.6)).is_consistent());
}

#[test]
fn inconsistent_geometry_produces_no_overlay_geometry() {
    // PDFium says 612×792 but the box + /Rotate 90 compute 792×612 (e.g. a box PDFium ignored).
    let g = PageGeometry::new(LETTER, Rotation::R90, (612.0, 792.0));
    assert!(!g.is_consistent());
    assert_eq!(g.rect_to_display(TEXT), None);
    assert_eq!(g.point_to_display(10.0, 10.0), None);
    // just beyond the tolerance
    assert!(!PageGeometry::new(LETTER, Rotation::R0, (612.6, 792.0)).is_consistent());
    // degenerate box
    let flat = PageBox { y1: 0.0, ..LETTER };
    assert!(!PageGeometry::new(flat, Rotation::R0, (612.0, 0.0)).is_consistent());
}

#[test]
fn geometry_point_to_display_rejects_non_finite_input() {
    let g = PageGeometry::new(LETTER, Rotation::R0, (612.0, 792.0));
    assert_eq!(g.point_to_display(100.0, 100.0), Some((100.0, 692.0)));
    assert_eq!(g.point_to_display(f32::NAN, 1.0), None);
}
