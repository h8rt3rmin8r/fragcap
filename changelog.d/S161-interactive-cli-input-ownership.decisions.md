<!-- spec-impact: 17.2, 17.7, 26.3 -->

**2026-09-29** Keep one shared buffered stdin source but hold its mutex only during each bounded authorization read. Passing a command-wide locked reader through every CLI path would broaden unrelated signatures, while a second raw console handle could split buffered input from exact plan authorization. Prompt I/O errors stop before later actions and are reported separately from a completed negative answer. The Print Screen report remains an independent observation without a demonstrated fragcap-specific cause.
