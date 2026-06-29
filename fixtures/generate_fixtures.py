#!/usr/bin/env python3
"""Generate deterministic PDF test fixtures (RFC 015 §6).

Hand-built PDF 1.4 files with correct xref tables — no third-party PDF
library, so fixtures are reproducible and reviewable byte-for-byte.

Fixtures:
  single-page-basic.pdf   1 page, US Letter (612x792 pt), text "Hello tile viewer".
  multi-page-search.pdf   3 pages; the word "tile" appears on pages 1 and 3
                          (display numbering), page 2 contains unrelated text.
  not-a-pdf.pdf           wrong magic bytes, for intake-rejection tests.

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
    out += b"0000000000 65535 f \n"
    for off in offsets[1:]:
        out += f"{off:010d} 00000 n \n".encode()
    out += (
        f"trailer\n<< /Size {count} /Root 1 0 R >>\n"
        f"startxref\n{xref_pos}\n%%EOF\n"
    ).encode()
    return bytes(out)


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

    (OUT_DIR / "not-a-pdf.pdf").write_bytes(b"GIF89a this is not a pdf\n")

    for name in ("single-page-basic.pdf", "multi-page-search.pdf", "not-a-pdf.pdf"):
        print(f"wrote {OUT_DIR / name}")


if __name__ == "__main__":
    main()
