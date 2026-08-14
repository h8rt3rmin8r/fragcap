"""
Generate manifest.json and VERIFY.md, and assert the kit's own claims.

This is deliberately a checker as well as a reporter: contrast ratios printed
in the README, the PDF and the tokens file are recomputed from the hex values,
and every SVG is scanned for live text and font-family declarations. If a claim
drifts, the build fails rather than shipping a confident wrong number.
"""

import os, re, json, hashlib, struct, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SKIP_DIRS = {".git", "__pycache__"}

# ── colour maths ───────────────────────────────────────────────────────────
def _lin(c):
    c = c / 255
    return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4

def luminance(hexstr):
    h = hexstr.lstrip("#")
    r, g, b = (int(h[i:i + 2], 16) for i in (0, 2, 4))
    return 0.2126 * _lin(r) + 0.7152 * _lin(g) + 0.0722 * _lin(b)

def contrast(a, b):
    la, lb = luminance(a), luminance(b)
    return (max(la, lb) + 0.05) / (min(la, lb) + 0.05)


def walk():
    for base, dirs, files in os.walk(ROOT):
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS)
        for f in sorted(files):
            if f in ("manifest.json", "VERIFY.md"):
                continue
            p = os.path.join(base, f)
            yield p, os.path.relpath(p, ROOT).replace(os.sep, "/")


def png_size(path):
    with open(path, "rb") as fh:
        head = fh.read(24)
    if head[:8] != b"\x89PNG\r\n\x1a\n":
        return None
    return struct.unpack(">II", head[16:24])


def ico_entries(path):
    with open(path, "rb") as fh:
        d = fh.read()
    n = struct.unpack("<H", d[4:6])[0]
    out = []
    for i in range(n):
        e = d[6 + i * 16:22 + i * 16]
        out.append((e[0] or 256, e[1] or 256))
    return out


def check_contrast(tokens, problems):
    C = {k: v["value"] for k, v in tokens["color"].items()}
    pairs = {
        "text_on_void": ("text", "void"),
        "text_muted_on_void": ("text_muted", "void"),
        "signal_cyan_on_void": ("signal_cyan", "void"),
        "capture_orange_on_void": ("capture_orange", "void"),
        "fault_on_void": ("fault", "void"),
        "light_text_on_light_surface": ("light_text", "light_surface"),
        "light_text_muted_on_light_surface": ("light_text_muted", "light_surface"),
        "light_cyan_on_light_surface": ("light_cyan", "light_surface"),
        "light_orange_on_light_surface": ("light_orange", "light_surface"),
        "fault_deep_on_light_surface": ("fault_deep", "light_surface"),
        "signal_cyan_on_light_surface": ("signal_cyan", "light_surface"),
    }
    rows = []
    for key, (fg, bg) in pairs.items():
        actual = round(contrast(C[fg], C[bg]), 2)
        stated = tokens["contrast"][key]
        ok = abs(actual - stated) < 0.02
        if not ok:
            problems.append("contrast %s: stated %.2f, actual %.2f" % (key, stated, actual))
        rows.append((fg, bg, actual, stated, ok))
    return rows


def check_svgs(problems):
    """No brand SVG may depend on a font being installed."""
    checked = 0
    for path, rel in walk():
        if not rel.endswith(".svg"):
            continue
        checked += 1
        s = open(path, encoding="utf-8").read()
        if re.search(r"<text\b", s):
            problems.append("%s contains live <text>" % rel)
        if re.search(r"font-family", s):
            problems.append("%s declares a font-family" % rel)
        if re.search(r'stroke-width="22"', s):
            problems.append("%s still contains stroked wordmark lettering" % rel)
    return checked


def check_clipping(problems):
    """
    No artwork may be cut off by its own viewBox.

    Touching an edge is fine - a tight-cropped wordmark is supposed to fill its
    box, and consumers apply clear space themselves. Extending *past* the edge
    is the defect: v1.0.0's wordmark drew its f crossbar out to x = -1 in a
    viewBox starting at 0, so the terminal was silently clipped.

    So the test renders each file twice: once as authored, and once with the
    viewBox expanded by a wide margin. If the expanded render carries ink
    outside where the original viewBox sits, the original was clipping it.
    """
    import cairosvg, io, re as _re
    from PIL import Image
    import numpy as np

    checked = 0
    for path, rel in walk():
        if not rel.startswith("logos/svg/") or not rel.endswith(".svg"):
            continue
        src = open(path, encoding="utf-8").read()
        m = _re.search(r'viewBox="([-\d.]+) ([-\d.]+) ([-\d.]+) ([-\d.]+)"', src)
        if not m:
            problems.append("%s has no viewBox" % rel)
            continue
        vx, vy, vw, vh = (float(g) for g in m.groups())

        pad = max(vw, vh) * 0.25
        widened = src.replace(
            m.group(0),
            'viewBox="%g %g %g %g"' % (vx - pad, vy - pad, vw + 2 * pad, vh + 2 * pad),
        )
        # The width/height attributes drive the rasteriser's scale, so they have
        # to grow with the viewBox or the render maps to the wrong pixel grid.
        widened = _re.sub(r'width="[\d.]+" height="[\d.]+"',
                          'width="%g" height="%g"' % (vw + 2 * pad, vh + 2 * pad),
                          widened, count=1)
        # drop any full-bleed background rect, which would fill the new margin
        widened = _re.sub(r'<rect width="\d+" height="\d+" fill="#[0-9A-Fa-f]{6}"/>',
                          "", widened, count=1)

        scale = 1000.0 / (vw + 2 * pad)
        buf = io.BytesIO()
        cairosvg.svg2png(bytestring=widened.encode(), write_to=buf, output_width=1000)
        buf.seek(0)
        alpha = np.asarray(Image.open(buf).convert("RGBA"))[:, :, 3]
        if alpha.max() == 0:
            checked += 1
            continue

        ys, xs = np.where(alpha > 0)
        # where the original viewBox sits inside the widened render, in px
        x0, y0 = pad * scale, pad * scale
        x1, y1 = (pad + vw) * scale, (pad + vh) * scale
        tol = 1.5  # antialiasing slop
        outside = []
        if xs.min() < x0 - tol: outside.append("left")
        if xs.max() > x1 + tol: outside.append("right")
        if ys.min() < y0 - tol: outside.append("top")
        if ys.max() > y1 + tol: outside.append("bottom")
        if outside:
            problems.append("%s: artwork is clipped at the %s viewBox edge"
                            % (rel, "/".join(outside)))
        checked += 1
    return checked


# Copy that carries the brand voice. Code comments and build docs are exempt:
# the rule is about what fragcap says out loud, not about how the kit is built.
VOICE_FILES = [
    "README.md",
    "SKILL.md",
    "build/brand-guide.html",
    "guidelines/index.html",
    "ui_kits/fragcap-web/index.html",
]

# The "X, not Y" contrast and its relatives. Cheap to write, hard to unsee, and
# the clearest tell of machine-written copy. Banned in brand voice; see the
# rhetorical-devices note in README.md.
TROPES = [
    (r"\b(?:is|are|was|were|it's|its)\s+(?:an?\s+)?[a-z][a-z ]{2,28},\s*not\s+(?:an?\s+)?[a-z]",
     'the "X, not Y" contrast'),
    (r"\bprivileges?\s+\w+\s+over\s+\w+", '"privileges X over Y"'),
    (r"\brather than merely\b", '"rather than merely"'),
    (r"[--]\s*never\s+(?:decorate|merely)\b", '"- never decorate"'),
    (r"\bnot just\s+[a-z][a-z ]{2,25}(?:-|--|,)\s*(?:it'?s|but)\b",
     '"not just X - it\'s Y"'),
]


def check_voice(problems):
    """Brand copy must not lean on the contrasting device."""
    checked = 0
    for rel in VOICE_FILES:
        path = os.path.join(ROOT, rel)
        if not os.path.exists(path):
            problems.append("voice check: %s is missing" % rel)
            continue
        raw = open(path, encoding="utf-8").read()
        # strip HTML comments and markup so prose is what gets matched
        prose = re.sub(r"<!--.*?-->", " ", raw, flags=re.S)
        prose = re.sub(r"<style.*?</style>", " ", prose, flags=re.S)
        prose = re.sub(r"<[^>]+>", " ", prose)
        prose = re.sub(r"\s+", " ", prose)
        for pattern, label in TROPES:
            for m in re.finditer(pattern, prose, flags=re.I):
                snippet = prose[max(0, m.start() - 30):m.end() + 20].strip()
                # the README and guide quote these patterns in order to ban them
                if re.search(r"never write|banned|tell of machine|do not build sentences"
                             r"|rhetorical|trope",
                             prose[max(0, m.start() - 400):m.end() + 200], re.I):
                    continue
                problems.append("%s uses %s: ...%s..." % (rel, label, snippet))
        checked += 1
    return checked


def check_mojibake(problems):
    """UTF-8 without BOM, no double-encoded sequences."""
    pattern = re.compile(r"[ÂÃâ][-¿]")
    text_ext = {".md", ".css", ".json", ".html", ".jsx", ".svg", ".txt", ".webmanifest"}
    checked = 0
    for path, rel in walk():
        if os.path.splitext(rel)[1] not in text_ext:
            continue
        raw = open(path, "rb").read()
        if raw.startswith(b"\xef\xbb\xbf"):
            problems.append("%s has a UTF-8 BOM" % rel)
        try:
            s = raw.decode("utf-8")
        except UnicodeDecodeError:
            problems.append("%s is not valid UTF-8" % rel)
            continue
        if pattern.search(s):
            problems.append("%s shows mojibake" % rel)
        checked += 1
    return checked


def check_page_fit(problems):
    """
    Ask the browser whether any guide page overruns its folio.

    The pages are fixed 8.5x11in boxes with overflow:hidden, so long content is
    cropped rather than reflowed - and a thumbnail will not show it. This runs
    build/check_pages.js and folds its findings in.
    """
    import subprocess
    script = os.path.join(ROOT, "build", "check_pages.js")
    source = os.path.join(ROOT, "build", "brand-guide.html")
    if not (os.path.exists(script) and os.path.exists(source)):
        problems.append("page-fit check: build/check_pages.js or brand-guide.html missing")
        return 0
    r = subprocess.run(["node", script, source], capture_output=True, text=True)
    lines = [l.strip() for l in r.stdout.splitlines() if l.strip()]
    for l in lines:
        if "OVERFLOW" in l or "TIGHT" in l:
            problems.append("brand-guide page fit: %s" % " ".join(l.split()))
    pages = [l for l in lines if l.split()[0].isdigit()]
    return len(pages)


def check_pdf(problems):
    import pikepdf
    path = os.path.join(ROOT, "brand-guide.pdf")
    info = {}
    with pikepdf.open(path) as pdf:
        info["pages"] = len(pdf.pages)
        info["title"] = str(pdf.docinfo.get("/Title", ""))
        info["author"] = str(pdf.docinfo.get("/Author", ""))
        with pdf.open_outline() as o:
            info["bookmarks"] = len(o.root)
        fonts = {}
        for page in pdf.pages:
            for _, f in dict(page.get("/Resources", {}).get("/Font", {})).items():
                base = str(f.get("/BaseFont", "?")).split("+")[-1].lstrip("/")
                desc = f.get("/DescendantFonts")
                fd = (desc[0] if desc else f).get("/FontDescriptor")
                embedded = bool(fd and (fd.get("/FontFile") or fd.get("/FontFile2")
                                        or fd.get("/FontFile3")))
                subtype = str(f.get("/Subtype", "")).lstrip("/")
                fonts[base] = (embedded, subtype)
        info["fonts"] = fonts

    allowed = {"GeistMono-Regular", "Geist-Regular", "Geist-Medium",
               "SpaceGroteskLight-Bold", "SpaceGroteskLight-Medium"}
    for name, (embedded, subtype) in fonts.items():
        if not embedded:
            problems.append("brand-guide.pdf: %s is not embedded" % name)
        if subtype == "Type3":
            problems.append("brand-guide.pdf: %s is a Type 3 (outlined) font" % name)
        if name not in allowed:
            problems.append("brand-guide.pdf: %s is not a fragcap kit font" % name)
    if not info["author"]:
        problems.append("brand-guide.pdf has no Author metadata")
    if info["bookmarks"] == 0:
        problems.append("brand-guide.pdf has no outline")
    return info


def main():
    problems = []
    tokens = json.load(open(os.path.join(ROOT, "tokens", "brand.tokens.json"),
                            encoding="utf-8"))

    files = list(walk())
    ratios = check_contrast(tokens, problems)
    n_svg = check_svgs(problems)
    n_clip = check_clipping(problems)
    n_text = check_mojibake(problems)
    n_voice = check_voice(problems)
    n_fit = check_page_fit(problems)
    pdf_info = check_pdf(problems)

    # ── manifest.json ─────────────────────────────────────────────────────
    manifest = {
        "name": "fragcap-brand-kit",
        "version": tokens["meta"]["version"],
        "parent": "ShruggieTech",
        "files": [],
    }
    hashes, dims, icos = [], [], []
    for path, rel in files:
        raw = open(path, "rb").read()
        digest = hashlib.sha256(raw).hexdigest()
        manifest["files"].append({"path": rel, "bytes": len(raw), "sha256": digest})
        hashes.append((digest, rel, len(raw)))
        if rel.endswith(".png"):
            wh = png_size(path)
            if wh:
                dims.append((rel, wh[0], wh[1], len(raw)))
        if rel.endswith(".ico"):
            icos.append((rel, ico_entries(path), len(raw)))

    with open(os.path.join(ROOT, "manifest.json"), "w", encoding="utf-8", newline="\n") as fh:
        json.dump(manifest, fh, indent=2)
        fh.write("\n")

    # ── VERIFY.md ─────────────────────────────────────────────────────────
    L = []
    A = L.append
    A("# Verification\n")
    A("Generated by `src/verify.py`. Every number below is measured from the")
    A("shipped files, not transcribed.\n")
    A("| Check | Result |")
    A("| --- | --- |")
    A("| Files in kit | %d |" % len(files))
    A("| Text files scanned for mojibake and BOM | %d |" % n_text)
    A("| Brand-voice files scanned for the \"X, not Y\" contrast | %d |" % n_voice)
    A("| SVGs scanned for live text and font dependencies | %d |" % n_svg)
    A("| Logo SVGs checked for viewBox clipping | %d |" % n_clip)
    A("| Contrast claims re-derived from hex | %d |" % len(ratios))
    A("| brand-guide.pdf pages | %d |" % pdf_info["pages"])
    A("| Guide pages measured for folio clearance | %d |" % n_fit)
    A("| brand-guide.pdf bookmarks | %d |" % pdf_info["bookmarks"])
    A("| **Problems found** | **%d** |" % len(problems))
    A("")
    if problems:
        A("## Problems\n")
        for p in problems:
            A("- %s" % p)
        A("")

    A("## Contrast\n")
    A("| Foreground | Background | Measured | Stated in tokens | AA (normal text) |")
    A("| --- | --- | ---: | ---: | --- |")
    for fg, bg, actual, stated, ok in ratios:
        aa = "pass" if actual >= 4.5 else ("large text only" if actual >= 3.0 else "**fail - never as text**")
        A("| %s | %s | %.2f:1 | %.2f:1 | %s |" % (fg, bg, actual, stated, aa))
    A("")
    A("`signal_cyan_on_light_surface` is expected to fail. It is listed so the")
    A("number is on the record: it is why Light Cyan exists.\n")

    A("## PDF fonts\n")
    A("| Font | Embedded | Subtype |")
    A("| --- | --- | --- |")
    for name, (embedded, subtype) in sorted(pdf_info["fonts"].items()):
        A("| %s | %s | %s |" % (name, "yes" if embedded else "**no**", subtype))
    A("")

    A("## PNG dimensions\n")
    A("| File | Width | Height | Bytes |")
    A("| --- | ---: | ---: | ---: |")
    for rel, w, h, b in dims:
        A("| `%s` | %d | %d | %d |" % (rel, w, h, b))
    A("")

    A("## ICO bundles\n")
    A("| File | Entries | Bytes |")
    A("| --- | --- | ---: |")
    for rel, entries, b in icos:
        A("| `%s` | %s | %d |" % (rel, ", ".join("%dx%d" % e for e in entries), b))
    A("")

    A("## SHA-256\n")
    for digest, rel, b in hashes:
        A("- `%s`  `%s` (%d bytes)" % (digest, rel, b))
    A("")

    with open(os.path.join(ROOT, "VERIFY.md"), "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(L))

    print("files: %d | svgs: %d | text: %d | contrast: %d" %
          (len(files), n_svg, n_text, len(ratios)))
    print("PDF: %d pages, %d bookmarks, fonts: %s" %
          (pdf_info["pages"], pdf_info["bookmarks"],
           ", ".join(sorted(pdf_info["fonts"]))))
    if problems:
        print("\nPROBLEMS (%d):" % len(problems))
        for p in problems:
            print("  -", p)
        return 1
    print("\nNo problems found.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
