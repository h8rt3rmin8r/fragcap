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
        }
        .render(now);
        assert!(complete.contains("\"packets\":10"));
        assert!(complete.contains("\"attributed\":9"));
        assert!(complete.contains("\"dropped\":0"));
        assert!(complete.contains("\"watching_discarded\":3"));
        assert!(complete.contains("\"discarded_out_of_window\":1"));
    }

    #[test]
    fn every_line_starts_with_the_timestamp_then_the_event() {
        let line = Event::FilterNarrowed { endpoints: 3 }.render(UNIX_EPOCH);
        assert!(line.starts_with("{\"ts\":\"1970-01-01T00:00:00Z\",\"event\":\"filter.narrowed\""));
        assert!(line.contains("\"endpoints\":3"));
        assert!(line.ends_with('}'));
    }
}
