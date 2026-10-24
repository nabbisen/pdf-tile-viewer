#!/usr/bin/env python3
"""Generate deterministic PDF test fixtures (RFC 015 §6).

Hand-built PDF 1.4 files with correct xref tables — no third-party PDF
library, so fixtures are reproducible and reviewable byte-for-byte.

Fixtures:
  single-page-basic.pdf   1 page, US Letter (612x792 pt), text "Hello tile viewer".
  multi-page-search.pdf   3 pages; the word "tile" appears on pages 1 and 3
                          (display numbering), page 2 contains unrelated text.
  navigation-links-outline.pdf
                          3 pages with outline entries, internal links, URI
                          links, and blocked action examples for RFC 026.
  not-a-pdf.pdf           wrong magic bytes, for intake-rejection tests.
  rotate-0.pdf, rotate-90.pdf, rotate-180.pdf, rotate-270.pdf
                          RFC 034 geometry fixtures: one 612x792 page each,
                          identical content, /Rotate 0/90/180/270. Text
                          "Hello tile viewer" (Helvetica 24) with its baseline
                          origin at user space (72, 700); one internal link
                          annotation /Rect [72 650 240 680] whose destination
                          is the same page, /XYZ 100 500 0.
  crop-offset.pdf         RFC 034: MediaBox [0 0 612 792], CropBox
                          [50 100 562 692] (non-zero crop origin, displayed
                          512x592). Text origin at user space (72, 600); link
                          /Rect [72 400 240 440], destination /XYZ 100 500 0.
  mixed-rotation.pdf      RFC 034: four 612x792 pages with /Rotate 0, 90,
                          180, 270 (display 612x792, 792x612, 612x792,
                          792x612). Same text and link as rotate-*.pdf; the
                          link on page N points to page (N+1) mod 4,
                          /XYZ 100 500 0.

Expected display-space geometry (RFC 034 section 5.3: origin top-left, y down,
points; user-space point (ux, uy) in page box (x0 y0 x1 y1)):
  r0   (ux-x0, y1-uy)    r90  (uy-y0, ux-x0)
  r180 (x1-ux, uy-y0)    r270 (y1-uy, x1-ux)
Link rect [72 650 240 680] on the 612x792 page -> display (x, y, w, h):
  rotate-0   (72, 112, 168, 30)     rotate-90  (650, 72, 30, 168)
  rotate-180 (372, 650, 168, 30)    rotate-270 (112, 372, 30, 168)
Destination (100, 500) -> display point:
  rotate-0 (100, 292)  rotate-90 (500, 100)  rotate-180 (512, 500)
  rotate-270 (292, 512)
crop-offset (r0, box 50 100 562 692): link [72 400 240 440] -> (22, 252, 168, 40);
  destination (100, 500) -> (50, 192); displayed page 512x592.
The text rectangle is whatever PDFium reports for the glyphs in user space (the
same for every rotation); the smoke tests convert it by hand with the formulas
above and compare within 0.5 pt.

Carve-out:
  password-protected.pdf  generated from single-page-basic.pdf with qpdf 12.3.2
                          for RFC 024 encrypted-PDF smoke coverage. Public
                          user password: pdf-tile-viewer-test. Public owner
                          password: pdf-tile-viewer-owner-test.
                          Uses 128-bit AES because qpdf 12.3.2 256-bit
                          encrypted output is not byte-stable across runs.
                          Regenerate after running this script with:
                          qpdf --static-id --static-aes-iv \
                            --encrypt pdf-tile-viewer-test \
                            pdf-tile-viewer-owner-test 128 --use-aes=y -- \
                            crates/pdf_engine/tests/fixtures/single-page-basic.pdf \
                            crates/pdf_engine/tests/fixtures/password-protected.pdf

Run from the repository root:
  python3 crates/pdf_engine/tests/fixtures/generate_fixtures.py
"""

from pathlib import Path

OUT_DIR = Path(__file__).resolve().parent


def build_pdf(pages: list[str]) -> bytes:
    """Assemble a minimal PDF: catalog, page tree, one content stream and
    one shared Helvetica font per document. Returns the full file bytes."""
    objects: list[bytes] = []  # 1-indexed object bodies, without "N 0 obj"

    page_count = len(pages)
    # Object layout:
    #   1 Catalog, 2 Pages, 3 Font,
    #   4..3+n Page objects, 4+n..3+2n content streams.
    first_page_obj = 4
    first_content_obj = first_page_obj + page_count

    kids = " ".join(f"{first_page_obj + i} 0 R" for i in range(page_count))
    objects.append(b"<< /Type /Catalog /Pages 2 0 R >>")
    objects.append(
        f"<< /Type /Pages /Kids [{kids}] /Count {page_count} >>".encode()
    )
    objects.append(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"
    )

    for i in range(page_count):
        objects.append(
            (
                f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
                f"/Resources << /Font << /F1 3 0 R >> >> "
                f"/Contents {first_content_obj + i} 0 R >>"
            ).encode()
        )

    for text in pages:
        stream = f"BT /F1 24 Tf 72 700 Td ({text}) Tj ET".encode()
        objects.append(
            b"<< /Length %d >>\nstream\n%s\nendstream" % (len(stream), stream)
        )

    out = bytearray()
    out += b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n"
    offsets = [0]  # object 0 is the free head
    for number, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += f"{number} 0 obj\n".encode() + body + b"\nendobj\n"

    xref_pos = len(out)
    count = len(objects) + 1
    out += f"xref\n0 {count}\n".encode()
    out += b"0000000000 65535 f\n"
    for off in offsets[1:]:
        out += f"{off:010d} 00000 n\n".encode()
    out += (
        f"trailer\n<< /Size {count} /Root 1 0 R >>\n"
        f"startxref\n{xref_pos}\n%%EOF\n"
    ).encode()
    return bytes(out)


def pdf_string(value: str) -> str:
    return (
        value
        .replace("\\", "\\\\")
        .replace("(", "\\(")
        .replace(")", "\\)")
    )


def assemble_objects(objects: list[bytes]) -> bytes:
    out = bytearray()
    out += b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n"
    offsets = [0]
    for number, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += f"{number} 0 obj\n".encode() + body + b"\nendobj\n"

    xref_pos = len(out)
    count = len(objects) + 1
    out += f"xref\n0 {count}\n".encode()
    out += b"0000000000 65535 f\n"
    for off in offsets[1:]:
        out += f"{off:010d} 00000 n\n".encode()
    out += (
        f"trailer\n<< /Size {count} /Root 1 0 R >>\n"
        f"startxref\n{xref_pos}\n%%EOF\n"
    ).encode()
    return bytes(out)


def content_stream(text: str) -> bytes:
    stream = f"BT /F1 24 Tf 72 700 Td ({pdf_string(text)}) Tj ET".encode()
    return b"<< /Length %d >>\nstream\n%s\nendstream" % (len(stream), stream)


def content_stream_lines(lines: list[tuple[int, int, int, str]]) -> bytes:
    stream = "\n".join(
        f"BT /F1 {size} Tf {x} {y} Td ({pdf_string(text)}) Tj ET"
        for size, x, y, text in lines
    ).encode()
    return b"<< /Length %d >>\nstream\n%s\nendstream" % (len(stream), stream)


def build_navigation_pdf() -> bytes:
    """Assemble a deterministic PDF with RFC 026 navigation metadata."""
    objects: list[bytes] = [
        # 1 Catalog, 2 Pages, 3 Font.
        b"<< /Type /Catalog /Pages 2 0 R /Outlines 10 0 R /PageMode /UseOutlines >>",
        b"<< /Type /Pages /Kids [4 0 R 5 0 R 6 0 R] /Count 3 >>",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        # 4..6 Page objects.
        (
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
            b"/Resources << /Font << /F1 3 0 R >> >> "
            b"/Annots [14 0 R 16 0 R] /Contents 7 0 R >>"
        ),
        (
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
            b"/Resources << /Font << /F1 3 0 R >> >> "
            b"/Annots [17 0 R 18 0 R 19 0 R] /Contents 8 0 R >>"
        ),
        (
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
            b"/Resources << /Font << /F1 3 0 R >> >> "
            b"/Annots [15 0 R 20 0 R 21 0 R 22 0 R] /Contents 9 0 R >>"
        ),
        # 7..9 Content streams.
        content_stream_lines(
            [
                (24, 72, 700, "Navigation page one"),
                (14, 72, 660, "Internal link to page 3"),
                (14, 72, 610, "External https link"),
            ]
        ),
        content_stream_lines(
            [
                (24, 72, 700, "Navigation page two"),
                (14, 72, 660, "Blocked file URI"),
                (14, 72, 610, "Relative URI"),
                (14, 72, 560, "Blocked launch action"),
            ]
        ),
        content_stream_lines(
            [
                (24, 72, 700, "Navigation page three"),
                (14, 72, 660, "Internal link to page 1"),
                (14, 72, 610, "Blocked remote document"),
                (14, 72, 560, "Blocked embedded document"),
                (14, 72, 510, "Blocked JavaScript action"),
            ]
        ),
        # 10..13 and 23 Outline tree.
        b"<< /Type /Outlines /First 11 0 R /Last 23 0 R /Count 4 >>",
        (
            b"<< /Title (Chapter 1) /Parent 10 0 R /Next 13 0 R "
            b"/First 12 0 R /Last 12 0 R /Count 1 "
            b"/Dest [4 0 R /XYZ 0 792 0] >>"
        ),
        b"<< /Title () /Parent 11 0 R /Dest [5 0 R /XYZ 0 792 0] >>",
        (
            b"<< /Title (Chapter 2) /Parent 10 0 R /Prev 11 0 R /Next 23 0 R "
            b"/Dest [6 0 R /XYZ 0 792 0] >>"
        ),
        # 14..22 Link annotations.
        (
            b"<< /Type /Annot /Subtype /Link /Rect [72 650 240 680] "
            b"/Border [0 0 0] /Dest [6 0 R /XYZ 0 792 0] >>"
        ),
        (
            b"<< /Type /Annot /Subtype /Link /Rect [72 650 240 680] "
            b"/Border [0 0 0] /Dest [4 0 R /XYZ 0 792 0] >>"
        ),
        (
            b"<< /Type /Annot /Subtype /Link /Rect [72 600 320 630] "
            b"/Border [0 0 0] "
            b"/A << /S /URI /URI (https://example.com/pdf-tile-viewer) >> >>"
        ),
        (
            b"<< /Type /Annot /Subtype /Link /Rect [72 650 320 680] "
            b"/Border [0 0 0] "
            b"/A << /S /URI /URI (file:///tmp/pdf-tile-viewer-blocked) >> >>"
        ),
        (
            b"<< /Type /Annot /Subtype /Link /Rect [72 600 320 630] "
            b"/Border [0 0 0] "
            b"/A << /S /URI /URI (relative/path) >> >>"
        ),
        (
            b"<< /Type /Annot /Subtype /Link /Rect [72 550 320 580] "
            b"/Border [0 0 0] "
            b"/A << /S /Launch /F (blocked.exe) >> >>"
        ),
        (
            b"<< /Type /Annot /Subtype /Link /Rect [72 600 320 630] "
            b"/Border [0 0 0] "
            b"/A << /S /GoToR /F (remote.pdf) /D [0 /Fit] >> >>"
        ),
        (
            b"<< /Type /Annot /Subtype /Link /Rect [72 550 320 580] "
            b"/Border [0 0 0] "
            b"/A << /S /GoToE /T << /R /C /N (embedded.pdf) >> "
            b"/D [0 /Fit] >> >>"
        ),
        (
            b"<< /Type /Annot /Subtype /Link /Rect [72 500 320 530] "
            b"/Border [0 0 0] "
            b"/A << /S /JavaScript /JS (app.alert\\(1\\)) >> >>"
        ),
        (
            b"<< /Title (Empty URI) /Parent 10 0 R /Prev 13 0 R "
            b"/A << /S /URI /URI () >> >>"
        ),
    ]
    return assemble_objects(objects)


def build_geometry_pdf(pages: list[dict]) -> bytes:
    """Assemble RFC 034 geometry fixtures. Each page dict has: rotate (int),
    crop (optional [x0, y0, x1, y1]), text_xy (x, y), link_rect (4 numbers),
    dest (target page index, x, y). Objects: 1 Catalog, 2 Pages, 3 Font,
    4..3+n Pages, 4+n..3+2n contents, 4+2n..3+3n link annotations."""
    n = len(pages)
    first_page, first_content, first_annot = 4, 4 + n, 4 + 2 * n
    kids = " ".join(f"{first_page + i} 0 R" for i in range(n))
    objects: list[bytes] = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        f"<< /Type /Pages /Kids [{kids}] /Count {n} >>".encode(),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    ]
    for i, page in enumerate(pages):
        crop = ""
        if page.get("crop"):
            crop = "/CropBox [%s] " % " ".join(str(v) for v in page["crop"])
        objects.append(
            (
                f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] {crop}"
                f"/Rotate {page['rotate']} "
                f"/Resources << /Font << /F1 3 0 R >> >> "
                f"/Annots [{first_annot + i} 0 R] "
                f"/Contents {first_content + i} 0 R >>"
            ).encode()
        )
    for page in pages:
        x, y = page["text_xy"]
        stream = f"BT /F1 24 Tf {x} {y} Td (Hello tile viewer) Tj ET".encode()
        objects.append(
            b"<< /Length %d >>\nstream\n%s\nendstream" % (len(stream), stream)
        )
    for page in pages:
        target, dx, dy = page["dest"]
        rect = " ".join(str(v) for v in page["link_rect"])
        objects.append(
            (
                f"<< /Type /Annot /Subtype /Link /Rect [{rect}] /Border [0 0 0] "
                f"/Dest [{first_page + target} 0 R /XYZ {dx} {dy} 0] >>"
            ).encode()
        )
    return assemble_objects(objects)


def standard_page(rotate: int, target: int = 0) -> dict:
    return {
        "rotate": rotate,
        "text_xy": (72, 700),
        "link_rect": [72, 650, 240, 680],
        "dest": (target, 100, 500),
    }


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    (OUT_DIR / "single-page-basic.pdf").write_bytes(
        build_pdf(["Hello tile viewer"])
    )

    (OUT_DIR / "multi-page-search.pdf").write_bytes(
        build_pdf(
            [
                "First page mentions a tile here",  # page 1: match
                "Second page talks about mosaics only",  # page 2: no match
                "Third page lays another tile down",  # page 3: match
            ]
        )
    )

    pages_50 = [
        f"Page {i}: benchmark text for tile render performance testing"
        for i in range(1, 51)
    ]
    (OUT_DIR / "fifty-pages-benchmark.pdf").write_bytes(build_pdf(pages_50))

    (OUT_DIR / "navigation-links-outline.pdf").write_bytes(build_navigation_pdf())

    (OUT_DIR / "not-a-pdf.pdf").write_bytes(b"GIF89a this is not a pdf\n")

    for rotate in (0, 90, 180, 270):
        (OUT_DIR / f"rotate-{rotate}.pdf").write_bytes(
            build_geometry_pdf([standard_page(rotate)])
        )
    (OUT_DIR / "crop-offset.pdf").write_bytes(
        build_geometry_pdf(
            [
                {
                    "rotate": 0,
                    "crop": [50, 100, 562, 692],
                    "text_xy": (72, 600),
                    "link_rect": [72, 400, 240, 440],
                    "dest": (0, 100, 500),
                }
            ]
        )
    )
    (OUT_DIR / "mixed-rotation.pdf").write_bytes(
        build_geometry_pdf(
            [standard_page(r, (i + 1) % 4) for i, r in enumerate((0, 90, 180, 270))]
        )
    )

    for name in (
        "single-page-basic.pdf",
        "multi-page-search.pdf",
        "fifty-pages-benchmark.pdf",
        "navigation-links-outline.pdf",
        "not-a-pdf.pdf",
        "rotate-0.pdf",
        "rotate-90.pdf",
        "rotate-180.pdf",
        "rotate-270.pdf",
        "crop-offset.pdf",
        "mixed-rotation.pdf",
    ):
        print(f"wrote {OUT_DIR / name}")


if __name__ == "__main__":
    main()
