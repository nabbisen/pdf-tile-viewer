//! Independent oracle for the user-space → display-space conversion (RFC 034 §5.3).
//!
//! PDFium's own `FPDF_PageToDevice` (via `PdfPage::points_to_pixels`) maps a user-space
//! point to device pixels with rotation and crop applied. At a large target width one pixel
//! is below 0.05 pt, so dividing the device result by the scale yields the display coordinate
//! PDFium itself would draw at. `user_space::point_to_display` must agree within 0.05 pt for
//! every fixture page (all four rotations, crop offset, mixed rotation).
//!
//! Lives in its own test binary: PDFium is initialised once per process and the smoke binary
//! already owns that for the serialized worker.
//!
//! Requires PDFium like the smoke tests (`PDF_TILE_VIEWER_PDFIUM_DIR` or `ci/.pdfium`);
//! skips with a notice otherwise.

use std::path::PathBuf;

use pdf_engine::loader::bind_from_dir;
use pdf_engine::user_space::{PageBox, PageGeometry, Rotation, UserRect, point_to_display};
use pdfium_render::prelude::*;

const TOLERANCE_POINTS: f32 = 0.05;
/// Large enough that one device pixel is below `TOLERANCE_POINTS` on the longest fixture edge
/// (792 pt → 25 px/pt → 0.04 pt per pixel).
const TARGET_WIDTH_PX: i32 = 20_000;

fn pdfium_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("PDF_TILE_VIEWER_PDFIUM_DIR") {
        return Some(PathBuf::from(dir));
    }
    let default = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../ci/.pdfium")
        .canonicalize()
        .ok()?;
    default.is_dir().then_some(default)
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Sample points in user space: the page-box corners, edge midpoints, centre, the fixtures'
/// known text origin / link corners / destination, and an arbitrary interior point.
fn sample_points(b: PageBox) -> Vec<(f32, f32)> {
    let (mx, my) = ((b.x0 + b.x1) / 2.0, (b.y0 + b.y1) / 2.0);
    let mut points = vec![
        (b.x0, b.y0),
        (b.x0, b.y1),
        (b.x1, b.y1),
        (b.x1, b.y0),
        (mx, b.y0),
        (mx, b.y1),
        (b.x0, my),
        (b.x1, my),
        (mx, my),
        (b.x0 + 123.4, b.y0 + 56.7),
    ];
    for known in [
        (72.0, 700.0),
        (72.0, 650.0),
        (240.0, 680.0),
        (100.0, 500.0),
        (72.0, 600.0),
    ] {
        if known.0 >= b.x0 && known.0 <= b.x1 && known.1 >= b.y0 && known.1 <= b.y1 {
            points.push(known);
        }
    }
    points
}

#[test]
fn point_conversion_agrees_with_pdfium_page_to_device() {
    let Some(dir) = pdfium_dir() else {
        eprintln!(
            "SKIP: set PDF_TILE_VIEWER_PDFIUM_DIR (or run ci/fetch-pdfium.sh) to run the oracle test"
        );
        return;
    };
    let (pdfium, _report) = bind_from_dir(&dir).expect("PDFium bind failed");

    let mut checked = 0usize;
    let mut worst = 0.0_f32;
    for name in [
        "rotate-0.pdf",
        "rotate-90.pdf",
        "rotate-180.pdf",
        "rotate-270.pdf",
        "crop-offset.pdf",
        "mixed-rotation.pdf",
    ] {
        let document = pdfium
            .load_pdf_from_file(&fixture(name), None)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        for (index, page) in document.pages().iter().enumerate() {
            let bounds = page.boundaries().bounding().expect("bounding box").bounds;
            let page_box = PageBox {
                x0: bounds.left().value,
                y0: bounds.bottom().value,
                x1: bounds.right().value,
                y1: bounds.top().value,
            };
            let rotation =
                Rotation::from_degrees(page.rotation().expect("rotation").as_degrees() as i32);

            // The cross-check the engine runs at open must hold for every fixture page.
            let geometry = PageGeometry::new(
                page_box,
                rotation,
                (page.width().value, page.height().value),
            );
            assert!(
                geometry.is_consistent(),
                "{name} page {index}: inconsistent geometry"
            );

            let config = PdfRenderConfig::new().set_target_width(TARGET_WIDTH_PX);
            let scale = TARGET_WIDTH_PX as f32 / page.width().value;
            assert!(
                1.0 / scale <= TOLERANCE_POINTS,
                "{name}: raster too coarse for the tolerance"
            );

            for (ux, uy) in sample_points(page_box) {
                let (px, py) = page
                    .points_to_pixels(PdfPoints::new(ux), PdfPoints::new(uy), &config)
                    .unwrap_or_else(|e| panic!("{name} page {index}: points_to_pixels: {e}"));
                let oracle = (px as f32 / scale, py as f32 / scale);
                let ours = point_to_display(page_box, rotation, ux, uy);
                assert!(
                    (ours.0 - oracle.0).abs() <= TOLERANCE_POINTS
                        && (ours.1 - oracle.1).abs() <= TOLERANCE_POINTS,
                    "{name} page {index} ({rotation:?}) user ({ux}, {uy}): ours {ours:?}, PDFium {oracle:?}"
                );
                worst = worst
                    .max((ours.0 - oracle.0).abs())
                    .max((ours.1 - oracle.1).abs());
                checked += 1;
            }

            // Rectangles: PDFium has no rect oracle, so convert both corners with it and
            // normalize — the same definition §5.3 gives for `rect_to_display`.
            let rect = UserRect {
                left: page_box.x0 + 20.0,
                bottom: page_box.y0 + 30.0,
                right: page_box.x0 + 150.0,
                top: page_box.y0 + 90.0,
            };
            let corner = |x: f32, y: f32| {
                let (px, py) = page
                    .points_to_pixels(PdfPoints::new(x), PdfPoints::new(y), &config)
                    .expect("points_to_pixels");
                (px as f32 / scale, py as f32 / scale)
            };
            let (ax, ay) = corner(rect.left, rect.bottom);
            let (bx, by) = corner(rect.right, rect.top);
            let ours = geometry.rect_to_display(rect).expect("rect converts");
            for (got, want, what) in [
                (ours.x, ax.min(bx), "x"),
                (ours.y, ay.min(by), "y"),
                (ours.width, (bx - ax).abs(), "width"),
                (ours.height, (by - ay).abs(), "height"),
            ] {
                assert!(
                    (got - want).abs() <= 2.0 * TOLERANCE_POINTS,
                    "{name} page {index} rect {what}: ours {got}, PDFium {want}"
                );
            }
        }
    }
    eprintln!(
        "oracle: {checked} points compared, worst deviation {worst:.4} pt (tolerance {TOLERANCE_POINTS} pt)"
    );
    assert!(checked >= 100, "oracle compared only {checked} points");
}
