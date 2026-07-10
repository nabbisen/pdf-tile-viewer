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
                            fixtures/single-page-basic.pdf \
                            fixtures/password-protected.pdf

Run from the repository root:  python3 fixtures/generate_fixtures.py
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

    for name in (
        "single-page-basic.pdf",
        "multi-page-search.pdf",
        "fifty-pages-benchmark.pdf",
        "navigation-links-outline.pdf",
        "not-a-pdf.pdf",
    ):
        print(f"wrote {OUT_DIR / name}")


if __name__ == "__main__":
    main()
