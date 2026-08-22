<!-- spec-impact: 15.8, 16.2 -->
The Steam library walk no longer hands a soundtrack's install path to the rest of
the pipeline as if it lived under `steamapps/common/`. A `Music`-typed title
(the walk now reads Steam's own appinfo cache to tell) resolves under
`steamapps/music/` instead, which is where it actually is, and is excluded from
discovery entirely: it has no network behavior and was previously appearing as a
spurious, `ready`-labeled capture target with a warning about a directory that
never existed.

`fragcap targets` now says when a registered target's install folder is gone,
instead of rendering it identically to a healthy row. A title that was
uninstalled but whose manifest lingered, a second library on a disconnected
drive, or a scanned folder that moved, all render with a short note in the
existing warning color (plain text when color is off), and the row is never
offered as the listing's suggested next capture. Nothing is ever removed from
the store because of this: the registration stays exactly as it was.
