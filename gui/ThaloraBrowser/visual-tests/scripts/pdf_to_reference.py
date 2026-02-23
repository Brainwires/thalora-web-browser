#!/usr/bin/env python3
"""
Convert PDF pages to reference PNG images for visual regression testing.

Usage:
    python3 pdf_to_reference.py <input.pdf> <output_dir> [--width 1280]

Each page is rendered as a separate PNG at the target pixel width,
maintaining the PDF's aspect ratio.

Requires: pymupdf (pip3 install pymupdf)
"""

import argparse
import os
import sys

try:
    import fitz  # pymupdf
except ImportError:
    print("ERROR: pymupdf is required. Install with: pip3 install pymupdf", file=sys.stderr)
    sys.exit(1)


def convert_pdf_to_pngs(pdf_path: str, output_dir: str, target_width: int = 1280) -> list[str]:
    """
    Convert each page of a PDF to a PNG at the specified pixel width.

    Returns a list of output file paths.
    """
    if not os.path.isfile(pdf_path):
        print(f"ERROR: PDF not found: {pdf_path}", file=sys.stderr)
        sys.exit(1)

    os.makedirs(output_dir, exist_ok=True)

    doc = fitz.open(pdf_path)
    output_files = []

    print(f"Converting {doc.page_count} pages from: {pdf_path}")
    print(f"Target width: {target_width}px")
    print(f"Output directory: {output_dir}")

    for page_num in range(doc.page_count):
        page = doc.load_page(page_num)
        rect = page.rect

        # Calculate zoom factor to achieve target width
        zoom = target_width / rect.width
        matrix = fitz.Matrix(zoom, zoom)

        # Render at the computed resolution
        pixmap = page.get_pixmap(matrix=matrix, alpha=False)

        filename = f"page-{page_num + 1:02d}.png"
        filepath = os.path.join(output_dir, filename)
        pixmap.save(filepath)

        print(f"  [{page_num + 1}/{doc.page_count}] {filename} ({pixmap.width}x{pixmap.height})")
        output_files.append(filepath)

    doc.close()
    print(f"\nDone. {len(output_files)} reference images saved to {output_dir}")
    return output_files


def main():
    parser = argparse.ArgumentParser(
        description="Convert PDF pages to reference PNGs for visual regression testing."
    )
    parser.add_argument("pdf", help="Path to the input PDF file")
    parser.add_argument("output_dir", help="Directory to save output PNGs")
    parser.add_argument(
        "--width",
        type=int,
        default=1280,
        help="Target pixel width for rendered pages (default: 1280)",
    )

    args = parser.parse_args()
    convert_pdf_to_pngs(args.pdf, args.output_dir, args.width)


if __name__ == "__main__":
    main()
