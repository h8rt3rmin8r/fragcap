**2026-08-08** Recorded for promotion to specification section 29: section 13.3
marks `dir` as always present and enumerates three values, `in`, `out`, and
`local`, but `Direction` in `fragcap-core` has two variants and
`CapturedPacket::direction` is optional, leaving a fourth state with no value
in the table. Slice S06 adds `unknown` for it. Omitting the key would break the
guarantee that lets a consumer parse without a presence check. Writing `local`
would be worse: section 12.6 leaves loopback direction undetermined until it
can be resolved from the attributed process's endpoint, so `local` and "not
determined" are different facts, and asserting the first from the second is the
substitution P-9 exists to block. The distinction is not hypothetical. Every
packet in `loopback.fcapng` is attributed and carries `dir=unknown`, which is
the honest record of what the pipeline knows today.

**2026-08-08** Recorded for promotion to specification section 29: section 13.3
presents `role` and `stage` as a pair, both marked "when stage-bound", but
`Attribution` carries them as independent options and its builder sets them
separately, so a role without a stage is representable and will occur. S06
decides each independently. Treating them as a pair would either drop an
observed role or fabricate a stage.

**2026-08-08** Recorded for promotion to specification section 29: section 13.3
names three characters requiring percent-encoding, the semicolon, the equals
sign, and the percent sign, which are the three that break the grammar. S06
also encodes every code point below 0x20 and the code point 0x7F, which break
the containing format: pcapng defines a comment as UTF-8 text, and a reader
meeting a NUL or a newline mid-comment behaves unpredictably. Percent-encoding
is lossless and reversible, so the widening preserves the observation rather
than altering it, which is why it does not conflict with P-9. The alternative
for a process name containing a newline would be stripping or replacing it.

**2026-08-08** Recorded for promotion to specification section 29: section 13.2
populates the Interface Statistics Block from the section 12.4 counters, but
pcapng's standard fields describe losses upstream of the capturing application
and section 12.4 has two counters, `buffer_dropped` and `sink_dropped`, that no
standard field expresses. S06 writes them in an `opt_comment` on that block
under the `fragcap:` sentinel. Writing only the three that fit would satisfy
section 13.2 as written and violate P-4, which makes an uncounted, unsurfaced
discard a defect; overloading `isb_osdrop` would report a fragcap loss as an
operating system loss, which P-9 forbids and which a reader could not detect.
Between a specification sentence and a constitution principle the constitution
wins, and here both can be satisfied at once.

**2026-08-08** Recorded as a known gap rather than resolved: section 12.7 says
the session anchor is written into the capture file, and section 13.2 does not
list it among the blocks. There is no session in S06 and therefore no anchor to
record, and inventing a placement now would fix a format decision on behalf of
the slice that has the data. S08 owns capture start and supplies it.

**2026-08-08** The corpus-driven tests for the writer live in the `fragcap`
facade rather than in `fragcap-sink`. Producing a written capture from a
fixture needs a replay source and a scripted attributor, which are siblings of
`fragcap-sink`, and reaching them from its `tests/` directory would mean a
dev-dependency on a sibling: the edge P-3 exists to prevent. This is recorded
rather than quietly done because it would not have been caught. `cargo xtask
deps` ignores `[dev-dependencies]` by design, and has a test asserting that it
does, so the violation would have passed the mechanical gate and been visible
only to a reviewer who went looking. S04 placed its end-to-end test the same
way for the same reason; the blind spot is worth stating once in a durable
place rather than rediscovering per slice.

**2026-08-08** The Interface Statistics Block timestamp is derived from the
last packet written on that interface, or zero when none was, and the writer
reads no clock anywhere. The block header carries a timestamp field that has to
hold something and the obvious something is the current time. That choice would
have made output differ between runs, so every golden would pass once and fail
afterward, and the natural response to a golden that always fails is to delete
it, which removes the only check in this slice that reaches outside its own
assumptions. Recorded because the defect is invisible in review and expensive
in consequence.

**2026-08-08** pcapng's `epb_flags` option carries a direction field, and S06
does not write it. Section 13.3 places direction in the annotation, and writing
it in both would put the same fact in two places that can disagree. Recorded
because the option is discoverable and the duplication is tempting.

**2026-08-08** Verification against an unmodified analyzer is a documented
manual step in the slice's quickstart, not a gate. Wireshark 4.6.3 on the
development machine reads the goldens and displays the annotations, which is
the actual claim of section 13.1 and P-5 tested on the population it concerns.
It is not wired into continuous integration because the runners are not
guaranteed to have Wireshark, and the constitution is explicit that a check
which did not run must never look like one that passed. Adding Wireshark to the
runner image is left as an option for S18, which owns analyzer integration and
has other reasons to want it. The mandatory check is the structural validator,
which is independent of the writer's encoding code and runs everywhere.
