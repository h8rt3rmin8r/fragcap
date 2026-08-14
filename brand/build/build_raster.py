"""Rasterise the SVG masters into PNG exports, favicons and the ICO bundle."""

import os, sys, io
import cairosvg
from PIL import Image

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SVG = os.path.join(ROOT, "logos", "svg")
PNG = os.path.join(ROOT, "logos", "png")
FAV = os.path.join(ROOT, "favicons")


def render(src, out, w, h=None, bg=None):
    cairosvg.svg2png(url=src, write_to=out, output_width=w,
                     output_height=h, background_color=bg)
    im = Image.open(out)
    return out, im.size


def render_im(src, w, h=None, bg=None):
    buf = io.BytesIO()
    cairosvg.svg2png(url=src, write_to=buf, output_width=w,
                     output_height=h, background_color=bg)
    buf.seek(0)
    return Image.open(buf).convert("RGBA")


def write_ico(path, entries):
    """Assemble a real multi-resolution ICO from per-size images."""
    import struct

    payloads = []
    for size, im in entries:
        buf = io.BytesIO()
        im.save(buf, format="ICO", sizes=[(size, size)])
        blob = buf.getvalue()
        count = struct.unpack("<H", blob[4:6])[0]
        assert count == 1, "expected a single-entry ICO from Pillow"
        offset = struct.unpack("<I", blob[6 + 12:6 + 16])[0]
        length = struct.unpack("<I", blob[6 + 8:6 + 12])[0]
        payloads.append((size, blob[offset:offset + length]))

    header = struct.pack("<HHH", 0, 1, len(payloads))
    dir_size = 16 * len(payloads)
    offset = 6 + dir_size
    directory, body = b"", b""
    for size, data in payloads:
        directory += struct.pack(
            "<BBBBHHII",
            0 if size >= 256 else size,
            0 if size >= 256 else size,
            0, 0, 1, 32, len(data), offset,
        )
        body += data
        offset += len(data)

    with open(path, "wb") as fh:
        fh.write(header + directory + body)


def main():
    os.makedirs(PNG, exist_ok=True)
    made = []

    # ── Logo PNG exports ───────────────────────────────────────────────────
    # Rendered from the vector masters with straight grayscale antialiasing.
    # The previous social preview was rasterised with subpixel (LCD) hinting,
    # which left red/blue colour fringes baked into a distributable asset.
    exports = [
        ("fragcap-horizontal-dark",  "fragcap-horizontal-dark-2400.png",  2400, 640),
        ("fragcap-horizontal-light", "fragcap-horizontal-light-2400.png", 2400, 640),
        ("fragcap-horizontal-white", "fragcap-horizontal-white-2400.png", 2400, 640),
        ("fragcap-stacked-dark",     "fragcap-stacked-dark-2048.png",     2048, 2048),
        ("fragcap-stacked-light",    "fragcap-stacked-light-2048.png",    2048, 2048),
        ("fragcap-mark-color",       "fragcap-mark-color-1024.png",       1024, 1024),
        ("fragcap-mark-dark-background",  "fragcap-mark-dark-1024.png",   1024, 1024),
        ("fragcap-mark-light-background", "fragcap-mark-light-1024.png",  1024, 1024),
        ("fragcap-wordmark-cyan",    "fragcap-wordmark-cyan-1440.png",    1440, None),
        ("fragcap-wordmark-white",   "fragcap-wordmark-white-1440.png",   1440, None),
        ("fragcap-wordmark-black",   "fragcap-wordmark-black-1440.png",   1440, None),
        ("fragcap-social-preview",   "fragcap-social-preview-1280x640.png", 1280, 640),
    ]
    for stem, name, w, h in exports:
        p, size = render(f"{SVG}/{stem}.svg", f"{PNG}/{name}", w, h)
        made.append((os.path.relpath(p, ROOT), size))

    # ── Favicons ───────────────────────────────────────────────────────────
    # At and below 32 px the four reticle corners and three terminals collapse
    # into noise, so small icons use the reduced mark (F plus terminals) and
    # larger ones use the full mark. Both keep the Void tile so the artwork
    # holds up against light and dark browser chrome.
    reduced = f"{SVG}/fragcap-mark-reduced.svg"
    full = f"{SVG}/fragcap-mark-dark-background.svg"
    small_src = f"{FAV}/favicon.svg"          # reduced mark on a Void tile

    for sz in (16, 32, 48, 256):
        src = small_src if sz <= 32 else full
        p, size = render(src, f"{FAV}/favicon-{sz}x{sz}.png", sz, sz, "#050708")
        made.append((os.path.relpath(p, ROOT), size))

    p, size = render(full, f"{FAV}/apple-touch-icon.png", 180, 180, "#050708")
    made.append((os.path.relpath(p, ROOT), size))
    for sz in (192, 512):
        p, size = render(full, f"{FAV}/android-chrome-{sz}x{sz}.png", sz, sz, "#050708")
        made.append((os.path.relpath(p, ROOT), size))

    # ICO bundle: reduced glyph for the tab-sized entries, full mark above.
    # Pillow's ICO writer only ever emits one directory entry, so each size is
    # encoded separately and the directory is assembled by hand. Every entry
    # therefore carries its own artwork rather than a resample of the 16 px one.
    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    write_ico(
        f"{FAV}/favicon.ico",
        [(s, render_im(small_src if s <= 32 else full, s, s, "#050708").convert("RGB"))
         for s in ico_sizes],
    )
    made.append(("favicons/favicon.ico", "sizes " + ", ".join(str(s) for s in ico_sizes)))

    for name, size in made:
        print("  ", name, size, os.path.getsize(os.path.join(ROOT, name)), "bytes")
    print("%d raster files written" % len(made))


if __name__ == "__main__":
    main()
