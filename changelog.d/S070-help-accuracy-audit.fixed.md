<!-- spec-impact: 17.2 -->
`capture --sink`'s help now names every scheme the parser actually accepts
(`file:`, `pcapng:`, `jsonl:`, `pipe:`, `fifo:`, `unix:`, `tcp://`) and every
`,key=value` modifier (`format`, `payload`, `rotate-size`, `rotate-duration`,
`queue`, `timeout`), where it previously named four schemes and none of the
modifiers. `--mode`, `--direction`, `--roles`, and `--wait` now each state
their effective default in `--help`, where none of the four did before.
`docs/fragcap-specification.md` section 17.2 no longer documents `-m`, `-q`,
or `-V` on `capture`, none of which the shipped grammar has ever had; the
specification is corrected to match the shipped grammar rather than the
reverse. `steam list`'s help now names the route from a listed app id to a
capturable target (`targets add --steam <app_id>`), which previously existed
only in a source comment clap never rendered. `capture`'s global
`--quiet`/`--silent`/`--json` flags no longer sit inside the four target-input
options (`--target`, `--id`, `--process`, and the positional selector); the
four are now contiguous on every page, and the globals sort after every
command's own options uniformly. `capture --help` and `targets --help` each
now carry a worked example, drawn from the specification's own section 9.1 and
`README.md`. Roughly two dozen further fields across the whole surface, never
given a one-line `-h` summary at all, now have one, with their fuller
explanation moved behind `--help`.
