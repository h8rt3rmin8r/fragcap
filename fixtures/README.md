# Fixture corpus

The capture files tier 1 tests run against, from specification section 25.3.
Eight captures, each paired with an attribution script declaring what the
scripted attributor answers for its flows.

Together with the replay source and the scripted attributor added in slice S04,
these make the pipeline a deterministic function from fixture input to output,
which is the claim specification section 25.1 makes: the whole thing runs with
no capture driver, no elevated privilege, and no game.

## These are generated, not hand-made

The generator lives in
[`crates/fragcap-capture/tests/corpus.rs`](../crates/fragcap-capture/tests/corpus.rs)
and **is the readable record of what each fixture contains**. The `.pcap` is
its output. A committed binary nobody can read is a test input nobody can
review, and section 25.3 requires these be reviewed before they land.

To change a fixture, edit the generator and regenerate:

```sh
FRAGCAP_UPDATE_FIXTURES=1 cargo test -p fragcap-capture --test corpus
```

Then **read the diff**. A regenerated fixture whose diff nobody looked at is
the same defect as a golden file updated without looking, which section 25.4
names for the goldens and which applies here for the same reason.

Without that variable, the same test checks instead of writing. It runs inside
`cargo xtask ci`, so drift is caught by the ordinary gate rather than by
remembering to run something. It fails if a committed file differs from what
the generator produces, if a capture has no script or a script no capture, if
anything exceeds its size ceiling, or if a fixture stops exercising the
condition section 25.3 states for it.

## What is in them

| Fixture | Exercises |
| --- | --- |
| `tcp-session` | Ordinary TCP flow, both directions |
| `udp-gameplay` | Sustained UDP flow at gameplay cadence |
| `ipv6-mixed` | IPv6 with extension header chains |
| `fragmented` | IP fragmentation, first and subsequent |
| `loopback` | Local traffic, direction ambiguity |
| `malformed` | Truncated and invalid headers |
| `port-reuse` | Same port, different processes, over time |
| `burst` | Sustained rate |

`burst` narrows what section 25.3 says. That section calls for a rate
"exceeding buffer capacity", and the buffer holds 65,536 packets, so a faithful
fixture would run to several megabytes and contradict the same section's
requirement that fixtures be small. Backpressure is a relationship between a
rate and a capacity rather than a property of a file, so this supplies the rate
and S08's test supplies a small capacity. Recorded for promotion to
specification section 29.

`malformed` is named for what its packets contain, not its file structure. The
pcap records are well-formed; the headers inside them are not, so it exercises
the parser's rejection causes. The reader's own skip counters are tested
against byte arrays built in its unit tests, because a committed file that is a
broken capture would confuse every other tool that opens this directory.

## What is not in them

No fixture contains traffic captured from a real game session. Section 25.3
forbids it, and such captures carry account identifiers, session tokens, and
addresses that do not belong in a public repository.

Every address is from a range reserved for documentation, `192.0.2.0/24`,
`198.51.100.0/24`, `203.0.113.0/24`, or `2001:db8::/32`, or is loopback. Link
layer addresses are locally administered so they cannot collide with a real
adapter. Every payload byte is `0xa5` filler.

That last rule is what makes the privacy requirement checkable. "Contains no
session token" is not something a test can evaluate, because no assertion
recognizes what a token looks like. "Every payload byte is filler" is, and it
is strictly stronger: anything that is not filler fails, including whatever
nobody thought to look for.

## Reading one by hand

The corpus is classic pcap, so ordinary tooling opens it:

```sh
tshark -r fixtures/tcp-session.pcap
```

That is a convenience. Nothing in the project depends on a packet analyzer
being installed, and the condition assertions in the corpus test are what
actually hold these files to their description.

## A note on writing a script for a loopback flow

When both endpoints are local there is no local one in the usual sense, so the
parser assigns the flow key's positions by a canonical ordering rather than by
locality (slice S03 decision D-5). A script entry for such a flow has to be
written in that same order or it matches nothing. `loopback.script` carries the
lower endpoint first, and says so.
