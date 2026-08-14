"""
A starter icon set.

The brand specifies iconography precisely - simple line icons, 1.5-2 px
strokes, square or lightly chamfered terminals, minimal rounding, direct
technical symbols - and then shipped no icons at all, so the rule had nothing
to measure against. These six are the symbols the written guide names:
filter, file, clock, interface, endpoint, search.

Drawn on a 24 grid with a 1.5 stroke and butt caps, using currentColor so they
inherit whatever semantic token the surrounding element sets.
"""

import os

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
OUT = os.path.join(ROOT, "icons")

ICONS = {
    # funnel with a square mouth - a filter, not a cocktail glass
    "filter": '<path d="M3.5 4.5h17l-6.5 8v6.5l-4 2v-8.5z"/>',
    # capture file with a folded corner
    "file": '<path d="M5.5 2.5h9l5 5v14h-14z"/><path d="M14.5 2.5v5h5"/>'
            '<path d="M8.5 12.5h7M8.5 16.5h7"/>',
    # timing mark
    "clock": '<circle cx="12" cy="12" r="9"/><path d="M12 6.5V12l4 2.5"/>',
    # network interface - a port with two lanes
    "interface": '<rect x="2.5" y="6.5" width="19" height="11"/>'
                 '<path d="M6.5 10.5h11M6.5 13.5h7"/>',
    # endpoint - a node on a link
    "endpoint": '<path d="M2.5 12h6M15.5 12h6"/><rect x="8.5" y="8.5" width="7" height="7"/>',
    # search across a trace
    "search": '<circle cx="10.5" cy="10.5" r="6.5"/><path d="M15.5 15.5l5.5 5.5"/>',
}

TEMPLATE = (
    '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" '
    'viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" '
    'stroke-linecap="butt" stroke-linejoin="miter" role="img" '
    'aria-label="%s">%s</svg>\n'
)


def main():
    os.makedirs(OUT, exist_ok=True)
    for name, body in ICONS.items():
        path = os.path.join(OUT, "%s.svg" % name)
        with open(path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(TEMPLATE % (name, body))
        print("  icons/%s.svg" % name, os.path.getsize(path), "bytes")

    readme = os.path.join(OUT, "README.md")
    with open(readme, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(
            "# fragcap icons\n\n"
            "Six starter icons on a 24 grid: filter, file, clock, interface,\n"
            "endpoint, search. They are the symbols the brand guide names.\n\n"
            "Stroke is 1.5 with butt caps and mitre joins, and the stroke uses\n"
            "`currentColor`, so an icon inherits whatever semantic token its\n"
            "container sets (`--fc-fg`, `--fc-accent`, `--fc-warning`, ...).\n\n"
            "Extend the set with direct technical symbols only. Never add\n"
            "mascots, weapons, controllers, shields, hooded figures or generic\n"
            "circuit-board decoration - see the prohibited treatments in\n"
            "`README.md`.\n"
        )
    print("  icons/README.md", os.path.getsize(readme), "bytes")


if __name__ == "__main__":
    main()
