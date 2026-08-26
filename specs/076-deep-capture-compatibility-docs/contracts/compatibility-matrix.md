# Contract: Selected-Target Compatibility Matrix

## Command

```powershell
fragcap targets show <selector>
fragcap targets show --id <stable-id>
```

The existing selector, database override, exit-code, ambiguity, and no-match
contracts remain unchanged.

## Human Output

The existing target fields appear first. The command then prints exactly one
compatibility section.

With no facts:

```text
compatibility:  unknown (no stored evidence)
```

With facts:

```text
compatibility:
  proxy-routing = reached-client | launch=steam-protocol-cold | source=observed-run | freshness=current
  inspectability = metadata-only | source=stale-observation | freshness=stale
```

Rules:

- one output row corresponds to one stored fact;
- key, value, source, and freshness always appear;
- launch appears only when stored;
- all rows remain visible when keys repeat or values conflict;
- note, final executable, path, account, endpoint, and target-version details do
  not appear in this section;
- the section is deterministic for the same fact set;
- no summary says a target is compatible or incompatible.

## Side Effects

The command may read the selected local store. It must not:

- write or refresh facts;
- launch a target or platform;
- start a proxy;
- modify certificate trust;
- perform catalog or internet requests;
- infer facts from target metadata.

## Errors

A fact-store read failure is a command failure and must not be replaced with an
unknown matrix. Unknown means a successful read returned no facts.
