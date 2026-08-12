`fragcap steam profile` no longer turns installers and redistributables into
capture stages or picks an installer as the terminal client. It drops obvious
non-game executables (installers, redistributables, crash handlers, helper stubs,
and hash-named temp installers) before classifying, keys launcher detection on the
executable name rather than its directory (so a folder named `Launcher` no longer
tags every file under it, including the game client, as a launcher), and orders
the launcher stages so the most launcher-like image leads (issue #64).
