"""Generate every fragcap logo asset from the shared geometry."""

import os, sys, math
sys.path.insert(0, os.path.dirname(__file__))
from geometry import *          # noqa: F401,F403
from typeset import Typesetter

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SVG = os.path.join(ROOT, "logos", "svg")
FAV = os.path.join(ROOT, "favicons")
FONTS = os.path.join(ROOT, "fonts", "ttf")

GEIST = Typesetter(os.path.join(FONTS, "Geist-Regular.ttf"))
GEIST_MED = Typesetter(os.path.join(FONTS, "Geist-Medium.ttf"))
MONO = Typesetter(os.path.join(FONTS, "GeistMono-Regular.ttf"))

# ── Lockup metrics ─────────────────────────────────────────────────────────
# The mark's inner F spans y 118..406 on the 512 grid. The wordmark's
# ascender-to-baseline range is bracketed to exactly that span, so the two
# elements share an optical top and an optical bottom instead of drifting
# apart the way the previous lockup did.
MARK_ART = MARK_BOX[2] - MARK_BOX[0]        # 368
F_TOP, F_BOTTOM = 118, 406
F_SPAN = F_BOTTOM - F_TOP                   # 288


def horizontal(cyan, orange, wm_fill, bg=None, w=1200, h=320):
    mark_h = 188.0
    s_m = mark_h / MARK_ART
    f_top = (F_TOP - MARK_BOX[1]) * s_m
    f_h = F_SPAN * s_m
    s_w = f_h / WM_CAP                       # wordmark ascender == F height
    wm_w, wm_h = WM_W * s_w, WM_H * s_w
    gap = mark_h * 0.28                      # >> one terminal width of clear air

    ink_w = mark_h + gap + wm_w
    x0 = (w - ink_w) / 2.0
    ink_h = max(mark_h, f_top + wm_h)
    y_m = (h - ink_h) / 2.0
    y_w = y_m + f_top

    body = (
        ('  <rect width="%d" height="%d" fill="%s"/>\n' % (w, h, bg)) if bg else ""
    )
    body += (
        '  <g transform="translate(%.3f %.3f) scale(%.6f)">\n'
        '    <g transform="translate(%d %d)">%s</g>\n  </g>\n'
        % (x0, y_m, s_m, -MARK_BOX[0], -MARK_BOX[1], mark_group(cyan, orange))
    )
    body += (
        '  <g transform="translate(%.3f %.3f) scale(%.6f)">%s</g>'
        % (x0 + mark_h + gap, y_w, s_w, wordmark_group(wm_fill))
    )
    return svg(w, h, "0 0 %d %d" % (w, h), body)


def stacked(cyan, orange, wm_fill, bg=None, size=1024):
    mark_h = 430.0
    s_m = mark_h / MARK_ART
    wm_w = 624.0
    s_w = wm_w / WM_W
    wm_h = WM_H * s_w
    gap = 70.0

    ink_h = mark_h + gap + wm_h
    y0 = (size - ink_h) / 2.0
    body = ('  <rect width="%d" height="%d" fill="%s"/>\n' % (size, size, bg)) if bg else ""
    body += (
        '  <g transform="translate(%.3f %.3f) scale(%.6f)">\n'
        '    <g transform="translate(%d %d)">%s</g>\n  </g>\n'
        % ((size - mark_h) / 2.0, y0, s_m, -MARK_BOX[0], -MARK_BOX[1],
           mark_group(cyan, orange))
    )
    body += (
        '  <g transform="translate(%.3f %.3f) scale(%.6f)">%s</g>'
        % ((size - wm_w) / 2.0, y0 + mark_h + gap, s_w, wordmark_group(wm_fill))
    )
    return svg(size, size, "0 0 %d %d" % (size, size), body)


def wordmark_only(fill):
    return svg(WM_W, WM_H, "0 0 %g %g" % (WM_W, WM_H),
               "  " + wordmark_group(fill))


def mark_only(cyan, orange, bg=None, small=False):
    body = ('  <rect width="512" height="512" fill="%s"/>\n' % bg) if bg else ""
    body += "  " + mark_group(cyan, orange, small=small)
    return svg(512, 512, "0 0 512 512", body)


def social_preview(w=1280, h=640):
    """Mark, wordmark, tagline and endorsement - every glyph outlined."""
    mark_h = 300.0
    s_m = mark_h / MARK_ART
    x_mark = 96.0
    y_mark = (h - mark_h) / 2.0            # mark optically centred

    s_w = 0.62
    wm_w, wm_h = WM_W * s_w, WM_H * s_w
    x_text = x_mark + mark_h + 96.0

    tagline = "Passive process-attributed network capture for games."
    endorse = "A SHRUGGIETECH PROJECT"
    tag_size, end_size, end_track = 30.0, 19.0, 2.6

    # Text column: wordmark, tagline, endorsement - centred on the same axis
    # as the mark so the composition survives the centre-crop that social
    # platforms apply to preview images.
    tag_drop, end_drop = 74.0, 128.0
    block_h = wm_h + end_drop + end_size * 0.25
    y_wm = (h - block_h) / 2.0

    body = (
        '  <rect width="%d" height="%d" fill="%s"/>\n'
        '  <defs>\n'
        '    <radialGradient id="mesh" cx="26%%" cy="44%%" r="72%%">\n'
        '      <stop offset="0" stop-color="%s" stop-opacity=".12"/>\n'
        '      <stop offset="1" stop-color="%s" stop-opacity="0"/>\n'
        '    </radialGradient>\n  </defs>\n'
        '  <rect width="%d" height="%d" fill="url(#mesh)"/>\n'
        % (w, h, VOID, CYAN, VOID, w, h)
    )
    body += (
        '  <g transform="translate(%.3f %.3f) scale(%.6f)">\n'
        '    <g transform="translate(%d %d)">%s</g>\n  </g>\n'
        % (x_mark, y_mark, s_m, -MARK_BOX[0], -MARK_BOX[1], mark_group())
    )
    body += (
        '  <g transform="translate(%.3f %.3f) scale(%.6f)">%s</g>\n'
        % (x_text, y_wm, s_w, wordmark_group(CYAN))
    )
    body += (
        '  <path fill="%s" d="%s"/>\n'
        % (TEXT, GEIST.path_data(tagline, tag_size, x_text, y_wm + wm_h + tag_drop))
    )
    body += (
        '  <path fill="%s" d="%s"/>'
        % (TEXT_MUTED,
           MONO.path_data(endorse, end_size, x_text, y_wm + wm_h + end_drop, end_track))
    )
    return svg(w, h, "0 0 %d %d" % (w, h), body)


def write(path, content):
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(content)
    return path


def main():
    os.makedirs(SVG, exist_ok=True)
    os.makedirs(FAV, exist_ok=True)
    files = []

    # Horizontal lockups
    files.append(write(f"{SVG}/fragcap-horizontal-dark.svg",
                       horizontal(CYAN, ORANGE, CYAN, VOID)))
    files.append(write(f"{SVG}/fragcap-horizontal-light.svg",
                       horizontal(LIGHT_CYAN, LIGHT_ORANGE, LIGHT_CYAN, LIGHT_SURF)))
    files.append(write(f"{SVG}/fragcap-horizontal-white.svg",
                       horizontal(WHITE, WHITE, WHITE)))
    files.append(write(f"{SVG}/fragcap-horizontal-black.svg",
                       horizontal(BLACK, BLACK, BLACK)))

    # Stacked lockups
    files.append(write(f"{SVG}/fragcap-stacked-dark.svg",
                       stacked(CYAN, ORANGE, CYAN, VOID)))
    files.append(write(f"{SVG}/fragcap-stacked-light.svg",
                       stacked(LIGHT_CYAN, LIGHT_ORANGE, LIGHT_CYAN, LIGHT_SURF)))
    files.append(write(f"{SVG}/fragcap-stacked-white.svg",
                       stacked(WHITE, WHITE, WHITE)))

    # Wordmarks
    files.append(write(f"{SVG}/fragcap-wordmark-cyan.svg", wordmark_only(CYAN)))
    files.append(write(f"{SVG}/fragcap-wordmark-white.svg", wordmark_only(WHITE)))
    files.append(write(f"{SVG}/fragcap-wordmark-black.svg", wordmark_only(BLACK)))

    # Marks
    files.append(write(f"{SVG}/fragcap-mark-color.svg", mark_only(CYAN, ORANGE)))
    files.append(write(f"{SVG}/fragcap-mark-white.svg", mark_only(WHITE, WHITE)))
    files.append(write(f"{SVG}/fragcap-mark-black.svg", mark_only(BLACK, BLACK)))
    files.append(write(f"{SVG}/fragcap-mark-dark-background.svg",
                       mark_only(CYAN, ORANGE, VOID)))
    files.append(write(f"{SVG}/fragcap-mark-light-background.svg",
                       mark_only(LIGHT_CYAN, LIGHT_ORANGE, LIGHT_SURF)))
    files.append(write(f"{SVG}/fragcap-mark-reduced.svg",
                       mark_only(CYAN, ORANGE, small=True)))

    files.append(write(f"{SVG}/fragcap-social-preview.svg", social_preview()))
    files.append(write(f"{FAV}/favicon.svg", mark_only(CYAN, ORANGE, VOID, small=True)))

    for f in files:
        print("  ", os.path.relpath(f, ROOT), os.path.getsize(f), "bytes")
    print("%d SVG files written" % len(files))


if __name__ == "__main__":
    main()
