"""
Convert a run of text into an outlined SVG path.

Any brand asset that ships as an image must not depend on a font being
installed on the machine that opens it. The previous fragcap social preview
used live <text> with font-family="Geist,Arial,sans-serif", so it rendered in
Arial anywhere Geist was absent - which is what happened to the shipped PNG.
Outlining the text at build time removes the dependency entirely.

Shaping goes through HarfBuzz so kerning and tracking match what a browser
would produce, then glyph outlines are pulled straight from the font.
"""

import uharfbuzz as hb
from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.misc.transform import Transform


class Typesetter:
    def __init__(self, ttf_path):
        self.path = ttf_path
        with open(ttf_path, "rb") as fh:
            data = fh.read()
        self.hb_face = hb.Face(data)
        self.hb_font = hb.Font(self.hb_face)
        self.tt = TTFont(ttf_path)
        self.upem = self.tt["head"].unitsPerEm
        self.glyphset = self.tt.getGlyphSet()
        self.order = self.tt.getGlyphOrder()

    def shape(self, text, size, tracking=0.0):
        """Return (positions, advance) in user units for the given text."""
        buf = hb.Buffer()
        buf.add_str(text)
        buf.guess_segment_properties()
        hb.shape(self.hb_font, buf, {"kern": True, "liga": True})
        scale = size / self.upem
        out, x = [], 0.0
        for info, pos in zip(buf.glyph_infos, buf.glyph_positions):
            out.append(
                (self.order[info.codepoint], x + pos.x_offset * scale, pos.y_offset * scale)
            )
            x += pos.x_advance * scale + tracking
        if out:
            x -= tracking
        return out, x

    def path_data(self, text, size, x=0.0, y=0.0, tracking=0.0):
        """Outlined SVG path data for `text`, baseline-origin at (x, y)."""
        scale = size / self.upem
        pen = SVGPathPen(self.glyphset, ntos=lambda v: ("%.2f" % v).rstrip("0").rstrip("."))
        glyphs, _ = self.shape(text, size, tracking)
        for name, gx, gy in glyphs:
            t = Transform(scale, 0, 0, -scale, x + gx, y + gy)
            self.glyphset[name].draw(TransformPen(pen, t))
        return pen.getCommands()

    def width(self, text, size, tracking=0.0):
        return self.shape(text, size, tracking)[1]
