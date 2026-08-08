# Data Model: Replay Source and Fixture Corpus

**Slice**: S04

**Created**: 2026-08-08

The types this slice adds. Signatures are indicative; the contract is in
[contracts/replay-api.md](contracts/replay-api.md).

## In `fragcap-capture`

### `PcapReader`

Decodes a classic pcap file held in memory into a sequence of raw packets.

| Field | Purpose |
| --- | --- |
| `data` | The whole file. Fixtures are capped at 64 KiB, so this is bounded |
| `cursor` | The offset of the next record header |
| `order` | Byte order, from the magic number, never from the host |
| `unit` | Timestamp fraction unit, from the magic number |
| `link_type` | What the file declares its frames are |
| `snaplen` | The file's declared snapshot length, used only for the counter |
| `stats` | [`ReplayStats`](#replaystats) |

Separate from `ReplaySource` because the file format and the seam are different
concerns: this decodes bytes, and the source adapts the result to a trait. The
split is also what lets the reader be tested against byte arrays built in the
test, with no file on disk, which is how every malformed case becomes
constructible.

### `ReplayStats`

One counter per way a record was not delivered as the file described it.

| Counter | Cause | Record delivered |
| --- | --- | --- |
| `truncated_record` | The file ends part way through a record header or its data | no |
| `impossible_length` | A record declares more data than the file can supply | no; reading stops |
| `caplen_exceeds_wire` | Captured length exceeds original on-wire length | yes, both lengths unchanged |
| `caplen_exceeds_snaplen` | Captured length exceeds the file's snapshot length | yes |

`skipped()` sums the two that were not delivered. There is deliberately no
grand total: adding a count of records delivered with a complaint to a count of
records not delivered at all produces a number with no meaning, and the same
rule S02 applied to `CaptureStats` applies here.

The two "delivered: yes" rows are the P-9 cases. The file contradicts itself,
the bytes are nonetheless present, and reconciling the contradiction by
adjusting a length would hide a defect in whatever wrote the file.

### `ReplaySource`

A `PacketSource` backed by a file.

| Field | Purpose |
| --- | --- |
| `reader` | The `PcapReader` |
| `filter` | The last filter program accepted, and never applied |

Behavior against the seam:

| Method | Behavior |
| --- | --- |
| `next_packet` | The next record, ignoring the timeout. `Err(Closed)` at end of file |
| `set_filter` | Stores the program, applies nothing, returns success |
| `stats` | `received` is what was delivered; both drop counts are zero |
| `link_type` | What the file declares |

The timeout is ignored because a file is never slow, and there is no honest
value to wait for. `Err(Closed)` rather than `Ok(None)` at exhaustion, because
`Ok(None)` means "keep going" and would spin a pipeline forever on a finished
fixture.

Storing a filter it does not apply is the least bad of three options. Failing
would break a pipeline that filters unconditionally; applying it would be S13's
work done in the wrong slice; accepting it silently and forgetting it would
leave nothing for a test to inspect.

## In `fragcap-attr`

### `AttributionScript`

The parsed form of a fixture's script.

| Field | Purpose |
| --- | --- |
| `entries` | Flow identity, window, and outcome |
| `endpoints` | What `active_endpoints` reports |

### `ScriptEntry`

| Field | Purpose |
| --- | --- |
| `proto` | TCP or UDP |
| `local` | The local endpoint, which may be a wildcard bind address |
| `remote` | `Some` for TCP, `None` for UDP |
| `window` | Always, or a half-open range of instants |
| `outcome` | An attribution, or explicitly unowned |

`remote` is an `Option` rather than a plain address, and loading rejects the two
combinations that do not correspond to anything a socket table can answer: a
UDP entry naming a remote, and a TCP entry without one. That is the same
asymmetry `AttributionKey` encodes, deliberately mirrored, so the double cannot
express an attribution the real attributor could never make.

### `ScriptError`

Named causes for a script that will not load: an unknown statement, a
malformed endpoint, a malformed window, a protocol and remote combination that
cannot be answered, and overlapping windows for one flow. Each names the line
it was on, because a script is authored by hand as often as generated.

### `ScriptedAttributor`

A `FlowAttributor` backed by a script.

| Field | Purpose |
| --- | --- |
| `script` | The parsed script |

No clock. Behavior against the seam:

| Method | Behavior |
| --- | --- |
| `resolve(key, at)` | The owner whose window contains `at`, or nothing |
| `refresh` | Succeeds, does nothing |
| `active_endpoints` | What the script declares |

The instant is a parameter of `resolve` rather than stored state. The first
draft of this slice put it on an inherent `set_now` instead, reasoning that a
real attributor reads a table that is already current. Review of pull request 7
refuted that twice: specification section 11.4 says capture and socket table
observation are not synchronized, so a real attributor is also answering about
the past; and the pipeline holds a boxed attributor, which can reach no
inherent method, so a time-windowed script would have stayed at the epoch.

## Matching

Resolution goes through the S02 machinery rather than a parallel comparison:

```text
FlowKey ──attribution_key()──▶ AttributionKey
                                    │
                       local_matches_bind(entry.local)
                                    │
                         + remote equality for Pair
                                    │
                            + window contains now
                                    ▼
                          Some(Attribution) | None
```

Using `local_matches_bind` rather than an equality test is what gives the double
the UDP wildcard bind allowance for free, and is why a test that passes against
a script is a test S10 has to satisfy.

## The corpus on disk

```text
fixtures/<name>.pcap      the capture, classic pcap, at most 64 KiB
fixtures/<name>.script    its attribution script, text
```

Eight of each, named in specification section 25.3. The pairing is checked in
both directions: a capture with no script and a script with no capture are both
reported, because either means the corpus no longer describes itself.

The whole directory is at most 256 KiB, asserted rather than judged, so it stays
reviewable and the repository stays small.

## Validation rules

Stated in the order the reader applies them, because the order decides which
counter fires when a file is wrong in more than one way.

1. **The file opens.** It is at least a file header, and its magic is one of
   the four. Anything else is a terminal open failure, not a counter, because
   there is no capture to account for.
2. **The record header fits.** Sixteen bytes remain. Otherwise
   `truncated_record`, and reading stops.
3. **The record data fits.** `caplen` bytes remain. Otherwise
   `impossible_length`, and reading stops, because continuing would resynchronize
   on whatever follows and deliver garbage as packets.
4. **The record is self-consistent.** `caplen` at most `orig_len`, and `caplen`
   at most the file's snapshot length. Either failing is counted; neither stops
   delivery, because the bytes are present and real.
5. **The packet is built.** The timestamp is converted using the unit the magic
   declared, and the two lengths are carried through exactly as recorded.

Steps 2 and 3 stop reading; step 4 never does. That distinction is the whole
difference between a file that cannot supply a record and a file that describes
one badly.
