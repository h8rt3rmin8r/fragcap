# Golden output files

One `.fcapng` per fixture in the corpus above this directory. Each is the exact
bytes the S06 pcapng writer produces when fed that fixture and its attribution
script.

These are generated, not hand-made. The generator lives in
`crates/fragcap/tests/goldens.rs` and is the readable record of how each golden
was produced; these files are the record of what the writer produced last time
somebody looked.

## What they are for

A writer verified only by its own reader has proven that two functions agree.
The goldens are the check that reaches past that: they are bytes a human read
once and a machine compares on every run afterward, so a change in output is
visible to a reviewer who was not present when it happened.

They also make later slices cheap. S07, S08, and everything that fans out to a
sink can assert against a known file instead of re-deriving what correct output
looks like.

## Regenerating

Only when the output format changed on purpose:

```sh
FRAGCAP_UPDATE_GOLDENS=1 cargo test -p fragcap --test goldens
```

Then confirm the drift check passes on the regenerated set:

```sh
cargo test -p fragcap --test goldens
```

## Read the diff

A regenerated golden is a change to the on-disk format, which is the most
inherited thing this project produces. Read the diff before committing it.

A golden that changed without an intended format change is a defect, not a
stale file. Regenerating to turn a red test green destroys the only evidence
that something moved, which is the failure mode this mechanism exists to
prevent.

## Verifying by hand

These files are ordinary pcapng and open in any analyzer:

```sh
capinfos fixtures/goldens/tcp-session.fcapng
tshark -r fixtures/goldens/tcp-session.fcapng -T fields -e frame.comment
```

The second command prints one annotation per packet. That output is
specification section 13.1 and constitution P-5 demonstrated on the tooling the
claim is about. See `specs/005-pcapng-writer-annotations/quickstart.md`.
