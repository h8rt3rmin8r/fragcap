Live capture (`run`, `tap`, and analyzer capture) now detects when it is not
running with Administrator rights and refuses before touching the capture driver,
explaining how to re-launch elevated, instead of failing later with a lower-level
driver error. Offline and read-only commands still run without elevation.
