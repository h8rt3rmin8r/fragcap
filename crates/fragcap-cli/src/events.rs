// SPDX-License-Identifier: Apache-2.0

//! The structured lifecycle events of specification section 17.5, and their
//! hand-rolled newline-delimited JSON.
//!
//! `serde_json` stays a test-only dependency by policy (slice S07), so the event
//! set, which is small and fixed, is serialized by hand over the sink crate's
//! JSON string escaper rather than by adding a serializer to the runtime graph.
//! Reusing that one escaper is what makes the event strings and the sink output
//! agree on escaping by construction.
//!
//! Each record carries an RFC3339 `Z` timestamp, formatted from a
//! [`SystemTime`] with a small civil-date conversion so no date crate is
//! pulled in.

use std::time::{SystemTime, UNIX_EPOCH};

use fragcap::write_json_string;

/// A lifecycle event, emitted on standard error under `--json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// The session armed: the capture handle is open and the watcher attached.
    SessionArmed { interfaces: Vec<String> },
    /// A stage matched a process.
    StageMatched {
        role: String,
        pid: u32,
        process: String,
    },
    /// A matched stage's process exited.
    StageExited { role: String, pid: u32 },
    /// The capture filter narrowed to this many active endpoints.
    FilterNarrowed { endpoints: usize },
    /// The session completed, carrying the headline counters.
    SessionComplete {
        packets: u64,
        attributed: u64,
        dropped: u64,
        watching_discarded: u64,
        discarded_out_of_window: u64,
        /// Packets excluded because they belong to a process this capture does
        /// not cover (slice S064).
        ///
        /// Carried here and not only in the human summary because `--json`
        /// suppresses that summary entirely: without these two fields a machine
        /// consumer sees a capture that observed thousands of packets, wrote a
        /// few, and accounted for none of the difference. Every discard path has
        /// a named counter on every surface, not just the one a person reads
        /// (P-4).
        scope_discarded: u64,
        /// Packets excluded on scope grounds that carried no attribution, so it
        /// is not known whether they were the capture's.
        scope_unresolved_discarded: u64,
    },
    /// A streaming consumer left, carrying its per-consumer accounting. Distinct
    /// from the capture-wide `dropped` in `session.complete`.
    StreamConsumer {
        transport: String,
        id: String,
        written: u64,
        dropped: u64,
        reason: String,
    },
    /// A ring-mode capture finished, carrying the count of packets evicted from
    /// the rolling window. The sink's own retention accounting, distinct from the
    /// capture-wide `dropped`: an eviction is the operator's declared window scope,
    /// not a capture loss, but it is surfaced so the omission is never silent.
    RingEvicted { evicted: u64 },
    /// A periodic snapshot of the live counters the human status block also
    /// renders (slice S069), for a `--json` consumer watching a long-running
    /// capture. Carries no holder-tally breakdown: that is a human-display aid
    /// (see the S069 `contracts/capture-progress-event.md`), and a `--json`
    /// consumer already has per-packet attribution in the captured file itself.
    ///
    /// `etw`+`windows`-gated because its one real constructor,
    /// `crate::orchestrator::capture_progress_event`, lives inside
    /// `drive_live`, which is gated the same way (an ETW event stream has no
    /// non-Windows meaning); see `lib.rs`'s note on `mod live_status` for why
    /// that keeps `cargo clippy --all-targets --all-features` clean on every
    /// platform rather than leaving this reachable-but-uncalled elsewhere.
    #[cfg(all(feature = "etw", windows))]
    CaptureProgress {
        elapsed_secs: u64,
        packets: u64,
        bytes: u64,
        active_endpoints: usize,
        watching_discarded: u64,
        discarded_out_of_window: u64,
        buffer_dropped: u64,
        sink_dropped: u64,
        scope_discarded: u64,
        scope_unresolved_discarded: u64,
    },
}

impl Event {
    /// The `event` discriminator string.
    fn kind(&self) -> &'static str {
        match self {
            Event::SessionArmed { .. } => "session.armed",
            Event::StageMatched { .. } => "stage.matched",
            Event::StageExited { .. } => "stage.exited",
            Event::FilterNarrowed { .. } => "filter.narrowed",
            Event::SessionComplete { .. } => "session.complete",
            Event::StreamConsumer { .. } => "stream.consumer",
            Event::RingEvicted { .. } => "ring.evicted",
            #[cfg(all(feature = "etw", windows))]
            Event::CaptureProgress { .. } => "capture.progress",
        }
    }

    /// Render this event as one NDJSON line (no trailing newline), stamped with
    /// `now`.
    pub fn render(&self, now: SystemTime) -> String {
        let mut line = String::from("{\"ts\":");
        write_json_string(&rfc3339_utc(now), &mut line);
        line.push_str(",\"event\":");
        write_json_string(self.kind(), &mut line);
        match self {
            Event::SessionArmed { interfaces } => {
                line.push_str(",\"interfaces\":[");
                for (i, name) in interfaces.iter().enumerate() {
                    if i > 0 {
                        line.push(',');
                    }
                    write_json_string(name, &mut line);
                }
                line.push(']');
            }
            Event::StageMatched { role, pid, process } => {
                line.push_str(",\"role\":");
                write_json_string(role, &mut line);
                line.push_str(",\"pid\":");
                line.push_str(&pid.to_string());
                line.push_str(",\"proc\":");
                write_json_string(process, &mut line);
            }
            Event::StageExited { role, pid } => {
                line.push_str(",\"role\":");
                write_json_string(role, &mut line);
                line.push_str(",\"pid\":");
                line.push_str(&pid.to_string());
            }
            Event::FilterNarrowed { endpoints } => {
                line.push_str(",\"endpoints\":");
                line.push_str(&endpoints.to_string());
            }
            Event::SessionComplete {
                packets,
                attributed,
                dropped,
                watching_discarded,
                discarded_out_of_window,
                scope_discarded,
                scope_unresolved_discarded,
            } => {
                line.push_str(",\"packets\":");
                line.push_str(&packets.to_string());
                line.push_str(",\"attributed\":");
                line.push_str(&attributed.to_string());
                line.push_str(",\"dropped\":");
                line.push_str(&dropped.to_string());
                line.push_str(",\"watching_discarded\":");
                line.push_str(&watching_discarded.to_string());
                line.push_str(",\"discarded_out_of_window\":");
                line.push_str(&discarded_out_of_window.to_string());
                line.push_str(",\"scope_discarded\":");
                line.push_str(&scope_discarded.to_string());
                line.push_str(",\"scope_unresolved_discarded\":");
                line.push_str(&scope_unresolved_discarded.to_string());
            }
            Event::StreamConsumer {
                transport,
                id,
                written,
                dropped,
                reason,
            } => {
                line.push_str(",\"transport\":");
                write_json_string(transport, &mut line);
                line.push_str(",\"id\":");
                write_json_string(id, &mut line);
                line.push_str(",\"written\":");
                line.push_str(&written.to_string());
                line.push_str(",\"dropped\":");
                line.push_str(&dropped.to_string());
                line.push_str(",\"reason\":");
                write_json_string(reason, &mut line);
            }
            Event::RingEvicted { evicted } => {
                line.push_str(",\"evicted\":");
                line.push_str(&evicted.to_string());
            }
            #[cfg(all(feature = "etw", windows))]
            Event::CaptureProgress {
                elapsed_secs,
                packets,
                bytes,
                active_endpoints,
                watching_discarded,
                discarded_out_of_window,
                buffer_dropped,
                sink_dropped,
                scope_discarded,
                scope_unresolved_discarded,
            } => {
                line.push_str(",\"elapsed_secs\":");
                line.push_str(&elapsed_secs.to_string());
                line.push_str(",\"packets\":");
                line.push_str(&packets.to_string());
                line.push_str(",\"bytes\":");
                line.push_str(&bytes.to_string());
                line.push_str(",\"active_endpoints\":");
                line.push_str(&active_endpoints.to_string());
                line.push_str(",\"watching_discarded\":");
                line.push_str(&watching_discarded.to_string());
                line.push_str(",\"discarded_out_of_window\":");
                line.push_str(&discarded_out_of_window.to_string());
                line.push_str(",\"buffer_dropped\":");
                line.push_str(&buffer_dropped.to_string());
                line.push_str(",\"sink_dropped\":");
                line.push_str(&sink_dropped.to_string());
                line.push_str(",\"scope_discarded\":");
                line.push_str(&scope_discarded.to_string());
                line.push_str(",\"scope_unresolved_discarded\":");
                line.push_str(&scope_unresolved_discarded.to_string());
            }
        }
        line.push('}');
        line
    }
}

/// Format a `SystemTime` as an RFC3339 UTC timestamp with a `Z` suffix and
/// second resolution.
///
/// Second resolution is enough for a lifecycle record; the point is a
/// standard, sortable, timezone-unambiguous stamp, not sub-second precision.
pub fn rfc3339_utc(now: SystemTime) -> String {
    let secs = now
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (hour, minute, second) = (rem / 3_600, (rem % 3_600) / 60, rem % 60);
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Days since the Unix epoch to a civil `(year, month, day)`.
///
/// Howard Hinnant's public-domain algorithm. Kept here rather than reached for
/// through a date crate, which would be a runtime dependency for one small
/// formatter.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (year + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn at_epoch(secs: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(secs)
    }

    #[test]
    fn the_epoch_formats_as_the_start_of_1970() {
        assert_eq!(rfc3339_utc(UNIX_EPOCH), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn a_known_instant_formats_correctly() {
        // 2021-01-01T00:00:00Z is 1609459200 seconds after the epoch.
        assert_eq!(rfc3339_utc(at_epoch(1_609_459_200)), "2021-01-01T00:00:00Z");
        // One more second, to catch a fencepost in the time-of-day split.
        assert_eq!(rfc3339_utc(at_epoch(1_609_459_201)), "2021-01-01T00:00:01Z");
    }

    #[test]
    fn each_event_renders_its_kind_and_fields() {
        let now = UNIX_EPOCH;
        let armed = Event::SessionArmed {
            interfaces: vec!["eth0".to_string()],
        }
        .render(now);
        assert!(armed.contains("\"event\":\"session.armed\""));
        assert!(armed.contains("\"interfaces\":[\"eth0\"]"));

        let matched = Event::StageMatched {
            role: "client".to_string(),
            pid: 4242,
            process: "game.exe".to_string(),
        }
        .render(now);
        assert!(matched.contains("\"event\":\"stage.matched\""));
        assert!(matched.contains("\"role\":\"client\""));
        assert!(matched.contains("\"pid\":4242"));
        assert!(matched.contains("\"proc\":\"game.exe\""));

        let complete = Event::SessionComplete {
            packets: 10,
            attributed: 9,
            dropped: 0,
            watching_discarded: 3,
            discarded_out_of_window: 1,
            scope_discarded: 0,
            scope_unresolved_discarded: 0,
        }
        .render(now);
        assert!(complete.contains("\"packets\":10"));
        assert!(complete.contains("\"attributed\":9"));
        assert!(complete.contains("\"dropped\":0"));
        assert!(complete.contains("\"watching_discarded\":3"));
        assert!(complete.contains("\"discarded_out_of_window\":1"));

        let ring = Event::RingEvicted { evicted: 17 }.render(now);
        assert!(ring.contains("\"event\":\"ring.evicted\""));
        assert!(ring.contains("\"evicted\":17"));
    }

    // `Event::CaptureProgress` is `etw`+`windows`-gated (see its own doc
    // comment), so this test is too; the other event variants' tests above
    // run on every platform.
    #[cfg(all(feature = "etw", windows))]
    #[test]
    fn capture_progress_renders_its_kind_and_fields() {
        let now = UNIX_EPOCH;
        let progress = Event::CaptureProgress {
            elapsed_secs: 135,
            packets: 4102,
            bytes: 812_004,
            active_endpoints: 3,
            watching_discarded: 0,
            discarded_out_of_window: 0,
            buffer_dropped: 0,
            sink_dropped: 0,
            scope_discarded: 0,
            scope_unresolved_discarded: 0,
        }
        .render(now);
        assert!(progress.contains("\"event\":\"capture.progress\""));
        assert!(progress.contains("\"elapsed_secs\":135"));
        assert!(progress.contains("\"packets\":4102"));
        assert!(progress.contains("\"bytes\":812004"));
        assert!(progress.contains("\"active_endpoints\":3"));
    }

    #[test]
    fn every_line_starts_with_the_timestamp_then_the_event() {
        let line = Event::FilterNarrowed { endpoints: 3 }.render(UNIX_EPOCH);
        assert!(line.starts_with("{\"ts\":\"1970-01-01T00:00:00Z\",\"event\":\"filter.narrowed\""));
        assert!(line.contains("\"endpoints\":3"));
        assert!(line.ends_with('}'));
    }
}
