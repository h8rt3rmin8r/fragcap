<!-- spec-impact: 17.6 -->
The shared terminal color predicate `use_color()` (`doctor` and `targets`)
now takes an explicit stream (`Stdout` or `Stderr`) rather than always testing
standard output, so a stderr-only surface cannot be silently gated on the
wrong stream's terminal-ness. `doctor` and `targets` are unaffected; both pass
`Stdout` and behave exactly as before.
