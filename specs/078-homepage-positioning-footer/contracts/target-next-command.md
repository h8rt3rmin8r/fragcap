# Contract: Target Next Command

For a populated human target listing, append exactly:

```text

Next command:  fragcap capture <row>
```

`<row>` is selected by the pre-S078 readiness and install-presence algorithm. S078 changes no precedence.

The line appears after any `Machine:` section. An empty listing keeps `Add one:` and `Scan a folder:` suggestions and emits no `Next command:` line.

When the same listing is produced through bare `fragcap`, the only additional bytes remain:

```text

Run `fragcap --help` to see all commands.
```
