"""
Post-process the printed guide: document metadata and a navigation outline.

Chromium sets a Title and nothing else, and emits no outline at all, so the
PDF opens with an empty bookmarks pane and reports no author. Both are fixed
here rather than left to whatever the print dialog happened to produce.
"""

import os
import pikepdf
from pikepdf import Dictionary, Name, String, OutlineItem

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
PDF = os.path.join(ROOT, "brand-guide.pdf")

BOOKMARKS = [
    ("Cover", 0),
    ("Brand foundation", 1),
    ("Logo system - marks and lockups", 2),
    ("Logo system - backgrounds and prohibited treatments", 3),
    ("Colour - palette", 4),
    ("Typography - display, reading, payload", 5),
    ("Typography - the type scale", 6),
    ("Visual language", 7),
    ("Voice and writing", 8),
    ("Parent relationship and implementation", 9),
    ("Asset inventory", 10),
]

VERSION = "1.1.0"


def main():
    with pikepdf.open(PDF, allow_overwriting_input=True) as pdf:
        with pdf.open_metadata(update_docinfo=False) as meta:
            meta["dc:title"] = "fragcap Brand System"
            meta["dc:creator"] = ["ShruggieTech"]
            meta["dc:description"] = (
                "Brand and design system for fragcap - passive "
                "process-attributed network capture for games."
            )
            meta["pdf:Keywords"] = (
                "fragcap, ShruggieTech, brand system, design system, "
                "packet capture, visual identity"
            )

        pdf.docinfo[Name.Title] = String("fragcap Brand System")
        pdf.docinfo[Name.Author] = String("ShruggieTech")
        pdf.docinfo[Name.Subject] = String(
            "Brand and design system, version %s" % VERSION
        )
        pdf.docinfo[Name.Keywords] = String(
            "fragcap, ShruggieTech, brand system, design system, packet capture"
        )

        with pdf.open_outline() as outline:
            outline.root.clear()
            for title, page in BOOKMARKS:
                outline.root.append(OutlineItem(title, page))

        # Open on the first page, fit to width, with the bookmarks pane showing.
        pdf.Root[Name.PageMode] = Name.UseOutlines
        pdf.Root[Name.PageLayout] = Name.SinglePage
        pdf.Root[Name.ViewerPreferences] = Dictionary(
            DisplayDocTitle=True
        )

        pdf.save(PDF, linearize=True)

    with pikepdf.open(PDF) as check:
        with check.open_outline() as o:
            n = len(o.root)
        print("  title    :", str(check.docinfo.get(Name.Title)))
        print("  author   :", str(check.docinfo.get(Name.Author)))
        print("  subject  :", str(check.docinfo.get(Name.Subject)))
        print("  pages    :", len(check.pages))
        print("  bookmarks:", n)
    print("  bytes    :", os.path.getsize(PDF))


if __name__ == "__main__":
    main()
