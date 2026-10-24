"""Convert a folder of PDFs into one .txt per PDF for convaudit.

usage: python tools/pdf2txt.py <pdf_dir> <out_dir>
Requires: pip install pypdf   (pages that fail to parse are skipped)
"""
import sys
from pathlib import Path
from pypdf import PdfReader

src, dst = Path(sys.argv[1]), Path(sys.argv[2])
dst.mkdir(parents=True, exist_ok=True)
for pdf in sorted(src.glob("*.pdf")):
    parts = []
    try:
        for page in PdfReader(pdf).pages:
            try:
                parts.append(page.extract_text() or "")
            except Exception:
                pass
    except Exception as e:
        print(f"skip {pdf.name}: {e}")
        continue
    text = "\n".join(parts)
    if len(text.strip()) < 500:
        print(f"warn {pdf.name}: almost no text (scanned PDF? needs OCR)")
    (dst / (pdf.stem + ".txt")).write_text(text, encoding="utf-8")
    print(f"ok   {pdf.name}: {len(text):,} chars")
