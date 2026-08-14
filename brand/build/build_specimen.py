"""
Build the type specimen with every glyph outlined.

The v1.0.0 specimen SVG used live <text> with a bare font-family and no
fallback and no embedded font, so opening the file on any machine without
Geist and Space Grotesk installed rendered the whole specimen in the default
serif. A type specimen that cannot show its own type is not a specimen.
"""

import os, sys
sys.path.insert(0, os.path.dirname(__file__))
from typeset import Typesetter
from geometry import (CYAN, ORANGE, VOID, TEXT, TEXT_MUTED, LINE, svg)

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "specimens")
FONTS = os.path.join(ROOT, "fonts", "ttf")

SG_BOLD = Typesetter(os.path.join(FONTS, "SpaceGrotesk-Bold.ttf"))
SG_MED = Typesetter(os.path.join(FONTS, "SpaceGrotesk-Medium.ttf"))
GEIST = Typesetter(os.path.join(FONTS, "Geist-Regular.ttf"))
MONO = Typesetter(os.path.join(FONTS, "GeistMono-Regular.ttf"))

W, H = 1600, 1000
X = 100


def text(ts, s, size, y, fill, track=0.0, x=X):
    return '  <path fill="%s" d="%s"/>\n' % (fill, ts.path_data(s, size, x, y, track))


def main():
    os.makedirs(OUT, exist_ok=True)
    b = '  <rect width="%d" height="%d" fill="%s"/>\n' % (W, H, VOID)

    b += text(MONO,  "FRAGCAP TYPOGRAPHY", 24, 112, CYAN, 4)
    b += text(SG_BOLD, "Observe the traffic.", 104, 232, TEXT, -2.6)
    b += text(GEIST, "Passive process-attributed network capture for games.", 30, 298, TEXT_MUTED)

    b += text(MONO, "HEX READABILITY TEST", 22, 404, ORANGE, 2)
    b += text(MONO, "00000000  45 00 00 3C 1C 46 40 00 40 06 B1 E6", 33, 470, TEXT)
    b += text(MONO, "00000010  AC 10 0A 63 AC 10 0A 0C C2 03 00 50", 33, 528, TEXT)

    b += text(MONO, "GLYPH DISAMBIGUATION", 22, 618, CYAN, 2)
    b += text(MONO, "0 O   1 l I   8 B   5 S   2 Z   { } [ ] ( )", 40, 686, TEXT)

    b += ('  <line x1="%d" y1="752" x2="%d" y2="752" stroke="%s" stroke-width="1"/>\n'
          % (X, W - X, LINE))

    b += text(GEIST, "Display: Space Grotesk 500 / 700, tracking -0.025em", 26, 832, TEXT)
    b += text(GEIST, "Body: Geist 400 / 500, line-height 1.65", 26, 878, TEXT)
    b += text(MONO,  "Data: Geist Mono 400 - payloads, code, labels, metadata", 26, 924, TEXT)

    out = svg(W, H, "0 0 %d %d" % (W, H), b.rstrip("\n"),
              title="fragcap type specimen")
    path = os.path.join(OUT, "fragcap-type-specimen.svg")
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(out)

    import cairosvg
    png = os.path.join(OUT, "fragcap-type-specimen.png")
    cairosvg.svg2png(url=path, write_to=png, output_width=W, output_height=H)

    for p in (path, png):
        print("  specimens/%s" % os.path.basename(p), os.path.getsize(p), "bytes")


if __name__ == "__main__":
    main()
