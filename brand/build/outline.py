"""
Expand fragcap's stroked wordmark/logo paths into filled vector outlines.

The original artwork drew every glyph as an open polyline with
stroke-width=22, stroke-linecap="square", stroke-linejoin="bevel".
That is fragile: it cannot be filled with currentColor, it breaks in any
expand-to-outlines workflow (print, vinyl, embroidery, font tooling), and
different renderers resolve caps/joins slightly differently.

Every segment in the source is a straight line, so the stroke outline is
exact geometry - no curve flattening is involved. Buffering each subpath by
half the stroke width with square caps and bevel joins reproduces the
rendered silhouette exactly, then the union becomes filled paths.
"""

import re
from shapely.geometry import LineString, MultiPolygon, Polygon
from shapely.ops import unary_union

TOKEN = re.compile(r"([MmLlHhVvZz])|(-?\d*\.?\d+)")


def parse_path(d):
    """Parse a straight-line SVG path into a list of subpaths (point lists)."""
    toks = [(c or n) for c, n in TOKEN.findall(d)]
    subpaths, cur = [], []
    x = y = 0.0
    sx = sy = 0.0
    cmd = None
    i = 0

    def flush():
        nonlocal cur
        if len(cur) >= 2:
            subpaths.append(cur)
        cur = []

    while i < len(toks):
        t = toks[i]
        if t.isalpha():
            cmd = t
            i += 1
            if cmd in "Mm":
                nx, ny = float(toks[i]), float(toks[i + 1])
                i += 2
                if cmd == "m":
                    nx, ny = x + nx, y + ny
                flush()
                x, y = nx, ny
                sx, sy = x, y
                cur = [(x, y)]
                # subsequent implicit pairs after M are lineto
                cmd = "L" if cmd == "M" else "l"
            continue

        if cmd in "Ll":
            nx, ny = float(toks[i]), float(toks[i + 1])
            i += 2
            if cmd == "l":
                nx, ny = x + nx, y + ny
        elif cmd in "Hh":
            nx = float(toks[i])
            i += 1
            if cmd == "h":
                nx = x + nx
            ny = y
        elif cmd in "Vv":
            ny = float(toks[i])
            i += 1
            if cmd == "v":
                ny = y + ny
            nx = x
        elif cmd in "Zz":
            i += 1
            nx, ny = sx, sy
        else:
            raise ValueError("unsupported command %r in %r" % (cmd, d))

        x, y = nx, ny
        cur.append((x, y))

    flush()
    return subpaths


def stroke_to_polygons(d, width):
    """Return the filled outline of a stroked path as a shapely geometry."""
    half = width / 2.0
    pieces = []
    for pts in parse_path(d):
        # drop consecutive duplicates; a zero-length subpath has no outline
        clean = [pts[0]]
        for p in pts[1:]:
            if p != clean[-1]:
                clean.append(p)
        if len(clean) < 2:
            continue
        pieces.append(
            LineString(clean).buffer(
                half,
                cap_style=3,   # square - matches stroke-linecap="square"
                join_style=3,  # bevel  - matches stroke-linejoin="bevel"
            )
        )
    return unary_union(pieces)


def fmt(v):
    r = round(v, 2)
    return str(int(r)) if r == int(r) else str(r)


def geom_to_path(geom):
    """Serialise a polygonal geometry to SVG path data (nonzero fill-rule safe)."""
    polys = []
    if isinstance(geom, Polygon):
        polys = [geom]
    elif isinstance(geom, MultiPolygon):
        polys = list(geom.geoms)
    else:
        polys = [g for g in getattr(geom, "geoms", []) if isinstance(g, Polygon)]

    out = []
    for poly in polys:
        rings = [(poly.exterior, False)] + [(r, True) for r in poly.interiors]
        for ring, is_hole in rings:
            coords = list(ring.coords)
            if coords[0] == coords[-1]:
                coords = coords[:-1]
            # exterior counter-clockwise, holes clockwise, so nonzero fill works
            area2 = sum(
                coords[i][0] * coords[(i + 1) % len(coords)][1]
                - coords[(i + 1) % len(coords)][0] * coords[i][1]
                for i in range(len(coords))
            )
            ccw = area2 > 0
            if (is_hole and ccw) or (not is_hole and not ccw):
                coords = coords[::-1]
            seg = "M" + " ".join(
                "%s %s" % (fmt(px), fmt(py)) for px, py in coords
            ) + "Z"
            out.append(seg)
    return "".join(out)


def outline_paths(path_data, width=22, translate=None, scale=None):
    """Expand a list of path 'd' strings into one filled path 'd' string."""
    geom = unary_union([stroke_to_polygons(d, width) for d in path_data])
    if scale or translate:
        from shapely import affinity
        if scale:
            geom = affinity.scale(geom, scale, scale, origin=(0, 0))
        if translate:
            geom = affinity.translate(geom, translate[0], translate[1])
    return geom_to_path(geom), geom
