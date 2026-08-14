"""Shared fragcap logo geometry, in a single place so every asset agrees."""

import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from outline import outline_paths

# ── Palette ────────────────────────────────────────────────────────────────
CYAN        = "#27C7E7"
ORANGE      = "#FF5300"
VOID        = "#050708"
SURFACE     = "#0B1115"
LINE        = "#21323A"
TEXT        = "#F2F7F8"
TEXT_MUTED  = "#94A8B0"
LIGHT_SURF  = "#F5F8F9"
LIGHT_TEXT  = "#102027"
LIGHT_CYAN  = "#006F82"
LIGHT_ORANGE= "#C24100"
WHITE       = "#FFFFFF"
BLACK       = "#050708"

# ── Mark ───────────────────────────────────────────────────────────────────
# Filled artwork, unchanged from the approved identity.
# Drawn on a 512 canvas; the artwork itself occupies 72..440 on both axes.
RETICLE = [
    "M72 154V72h82v22H94v60z",
    "M358 72h82v82h-22V94h-60z",
    "M72 358h22v60h60v22H72z",
    "M418 358h22v82h-82v-22h60z",
]
PACKET_F = [
    "M156 134l16-16h34v272h-50z",
    "M188 118h150l14 12-14 12H188z",
    "M188 220h150l14 12-14 12H188z",
    "M188 322h92l14 12-14 12h-92z",
]
TERMINALS = [
    "M370 118h34v34h-34z",
    "M370 220h34v34h-34z",
    "M312 322h34v34h-34z",
]
MARK_BOX = (72, 72, 440, 440)          # x0, y0, x1, y1 of the artwork
MARK_CANVAS = 512
TERMINAL = 34                           # clear-space unit: one orange terminal

# Reduced mark for <= 32 px. The four reticle corners and the three terminals
# collapse into noise at browser-tab size, so small icons drop the corners and
# keep the F with its terminals - the part that still reads as fragcap.
# Documented as an approved exception, not an ad-hoc redraw.
PACKET_F_SMALL = [
    "M120 108l24-24h50v344h-74z",
    "M168 84h198l20 18-20 18H168z",
    "M168 220h198l20 18-20 18H168z",
    "M168 356h122l20 18-20 18H168z",
]
TERMINALS_SMALL = [
    "M398 84h46v46h-46z",
    "M398 220h46v46h-46z",
    "M320 356h46v46h-46z",
]

# ── Wordmark ───────────────────────────────────────────────────────────────
# Source construction: open polylines, stroke-width 22, square caps, bevel
# joins. Expanded to filled outlines at build time (see outline.py). The
# silhouette is unchanged; it simply no longer depends on stroke rendering.
WORDMARK_STROKES = [
    "M30 150V42L42 30h42M10 82h68",                        # f
    "M128 150V72m0 0h34l18 18",                            # r
    "M224 82l12-12h45v80h-48l-13-13v-24l13-13h48",         # a
    "M378 70h-45l-13 13v42l13 13h45m0-68v88l-13 13h-32",   # g
    "M475 79l-9-9h-38l-13 13v54l13 13h43",                 # c
    "M516 82l12-12h45v80h-48l-13-13v-24l13-13h48",         # a
    "M621 174V70h45l13 13v44l-13 13h-45",                  # p
]
STROKE_W = 22

_wm_raw, _wm_geom = outline_paths(WORDMARK_STROKES, STROKE_W)
_minx, _miny, _maxx, _maxy = _wm_geom.bounds        # (-1, 19, 690, 185)

# Re-canvas to the tight bounding box. The original wordmark SVG left the
# f crossbar at x = -1, i.e. clipped by its own viewBox, with 0 px of padding
# on the left and 60 px on the right. Consumers apply clear space themselves,
# per the one-terminal-width rule.
WORDMARK_PATH, _ = outline_paths(
    WORDMARK_STROKES, STROKE_W, translate=(-_minx, -_miny)
)
WM_W = _maxx - _minx        # 691
WM_H = _maxy - _miny        # 166
WM_ASCENDER = 19            # top of f, in pre-translate coords
WM_BASELINE = 161 - _miny   # baseline y within the re-canvassed box
WM_CAP = WM_BASELINE        # ascender height above baseline


def paths(ds, fill):
    return "".join('<path fill="%s" d="%s"/>' % (fill, d) for d in ds)


def mark_group(cyan=CYAN, orange=ORANGE, small=False):
    """The mark's inner markup, on its native 512 grid."""
    if small:
        return (
            '<g class="packet-f">%s</g><g class="captured-terminals">%s</g>'
            % (paths(PACKET_F_SMALL, cyan), paths(TERMINALS_SMALL, orange))
        )
    return (
        '<g class="capture-reticle">%s</g>'
        '<g class="packet-f">%s</g>'
        '<g class="captured-terminals">%s</g>'
        % (paths(RETICLE, cyan), paths(PACKET_F, cyan), paths(TERMINALS, orange))
    )


def wordmark_group(fill=CYAN):
    return '<path class="fragcap-wordmark" fill="%s" d="%s"/>' % (fill, WORDMARK_PATH)


def svg(width, height, viewbox, body, title="fragcap"):
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="%s" height="%s" '
        'viewBox="%s" role="img" aria-label="%s">\n  <title>%s</title>\n%s\n</svg>\n'
        % (width, height, viewbox, title, title, body)
    )
