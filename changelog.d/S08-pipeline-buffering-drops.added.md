### Pipeline, buffering, and drop accounting (S08)

`fragcap-core` gains a `pipeline` module implementing specification sections
8.6 and 8.6's data flow together with section 12.4's bounded buffer. It is the
first thing in the project that runs the whole capture path, and the first
producer of fragcap's own loss counters.

**`Pipeline`** composes a `PacketSource`, a `FlowAttributor`, the S03 header
parser, and any number of `Sink` values, all as trait objects. One pass over a
source produces every configured output. Construction validates the
configuration and starts nothing; `run` consumes the pipeline and blocks until
the run ends.

**The bounded buffer** holds 65,536 packets by default, evicts the oldest to
admit the newest, and never waits for a sink to make progress. Section 12.4's
reason is the one that governs: blocking the acquisition side stalls the kernel
buffer behind it and converts a visible fragcap drop into a less visible kernel
drop.

**Drop accounting is real.** `buffer_dropped` advances once per eviction.
`sink_dropped` advances once per write that did not happen, counted per sink
rather than per packet. `kernel_dropped` and `interface_dropped` are relayed
from the backend unaltered. The parser's own counters are collected into the
run. The `CaptureStats` handed to `Sink::finish` is the run's own final value,
and the same value the report carries.

The property asserted throughout is conservation rather than reachability: for
every sink, the packets it received plus the buffer's evictions plus its
refusals equal the packets the pipeline accepted. That identity holds under
every thread interleaving, and it is checked in every pipeline test. A discard
path added later with no counter fails there rather than passing quietly.

**Ending is explicit.** `PipelineReport` carries the statistics, an
`EndReason` naming source exhaustion, an operator stop, a terminal source
failure, or every sink having retired, and a list of sink failures. It is
`#[must_use]` and carries the accounting on the failure path as well as the
clean one, so there is no way to learn the outcome without also being handed
the numbers. `into_result` supplies the ordinary `Result` shape for callers
that want failure to propagate.

**A failed sink is retired, not fatal.** A sink returning an error that
`SinkError::is_countable` rejects stops receiving packets; every subsequent
packet advances `sink_dropped` for it, exactly as a refusal would. Other sinks
keep working, and the run ends only when every sink has retired. Every sink is
flushed and finished regardless, so its output is terminated and carries the
final accounting.

**`StopHandle`** ends a run cooperatively. It is observed between packets, so
stop latency is bounded by the configured read timeout rather than being
unbounded or hidden.

The whole fixture corpus now runs end to end through the real pipeline with
both writers attached, reproducing the committed goldens. `cargo xtask ci`
covers it.

No runtime dependency was added. The workspace still has exactly one.
