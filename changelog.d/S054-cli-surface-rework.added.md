<!-- spec-impact: 15.4, 15.5, 15.7, 16.3, 16.4, 17 -->

### Command line surface rework (slice S054)

The three capture verbs `run`, `tap`, and `watch` collapse into a single `capture`
verb, with no aliases and no deprecation shims. A target is identified by exactly
one of two mutually exclusive, required inputs: `--target <selector>` (a stored
target resolved against `local.db` by an S051 selector) or `--process <image>` (a
raw process image name). Every other flag is orthogonal to the target input, so all
five previously-inexpressible captures become expressible: a named process into a
ring buffer, a named process waited for, a registered title launched under capture,
a profile-equivalent capture with a give-up timeout, and a named process with a ring
buffer. `--launch` requires the resolved `--target` to carry a Steam anchor; a
`--process` capture, or an anchorless target, is refused (exit 2) rather than
silently ignored. The optional path anchors (`--path`, `--path-regex`) the retired
`watch` carried survive on `capture`.

The profile-file surface is retired whole (completing the S051 US5 deferral): the
`profile` command and its subcommands, the AppData profile directory, the
`--profile-dir` global, the file-backed profile provider, and the `--profile`
capture selector are all removed. `schema validate <file>` remains as the general
JSON-artifact validator.

The command namespaces now follow the two stores. A new `catalog` namespace owns
every operation that writes the shipped, disposable `catalog.db`: `import`,
`export`, `seed`, `seed-engine`, `seed-signatures` (moved from `targets`), and a new
`catalog update` that fetches the published catalog behind the `net` feature. The
`targets` namespace keeps only user-store operations, and registering an installed
Steam title moved from `steam profile <app_id>` to `targets add --steam <app_id>`;
the `steam` namespace keeps `steam list`.

`--help` groups the surface under four presentational headings (Capture, Targets,
Environment, Data) that hide nothing, rendered through a custom help template
because the pinned clap 4.5.32 cannot group subcommands natively. A bare `fragcap`
lists the registered targets and prints a footer pointing at `--help`; an explicit
`fragcap targets` prints the same listing without the footer.

No capture, attribution, pipeline, sink, or core code changed, and no dependency was
added or removed; the change is confined to the `fragcap-cli` argument grammar,
dispatch, and assembly seam, plus documentation and the master-specification command
surface (section 17).
