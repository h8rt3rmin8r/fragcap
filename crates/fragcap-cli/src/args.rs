// SPDX-License-Identifier: Apache-2.0

//! The value grammars for the command line flags of specification section 17.2.
//!
//! Each is a clap `value_parser`: a function from the raw string to a typed
//! value or a usage message. The duration and size grammars delegate to
//! `fragcap_core`, so the command line and a profile parse `30m` and `4mb` the
//! same way. The rest are small parsers local to the command line, because they
//! describe command line syntax (a sink specification, a direction) that has no
//! meaning in a profile.

use std::path::PathBuf;
use std::time::Duration;

use clap::ValueEnum;

use fragcap::core::{duration, size};

/// Parse a duration literal, delegating to the shared core grammar.
pub fn parse_duration(raw: &str) -> Result<Duration, String> {
    duration::parse(raw).map_err(|e| e.to_string())
}

/// Parse a size literal into a byte count, delegating to the shared core
/// grammar.
pub fn parse_size(raw: &str) -> Result<u64, String> {
    size::parse(raw).map_err(|e| e.to_string())
}

/// Which direction of a flow the capture is scoped to.
///
/// Recorded on the effective configuration and surfaced. Full directional
/// filtering of output is a later slice (specification FR-011b); this slice
/// accepts and validates the value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Direction {
    /// Inbound only.
    In,
    /// Outbound only.
    Out,
    /// Both directions.
    Both,
}

impl Direction {
    /// The word this direction is written and surfaced as.
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::In => "in",
            Direction::Out => "out",
            Direction::Both => "both",
        }
    }
}

/// A comma-separated set of role names that scopes which stages trigger and are
/// captured.
///
/// Empty entries are refused rather than silently dropped, because a trailing
/// comma is a mistake and honoring it would scope the capture to fewer roles
/// than the author wrote.
pub fn parse_roles(raw: &str) -> Result<Vec<String>, String> {
    let roles: Vec<String> = raw.split(',').map(str::trim).map(str::to_string).collect();
    if roles.iter().any(|r| r.is_empty()) {
        return Err(format!(
            "`{raw}` has an empty role name; give a comma-separated list of role names"
        ));
    }
    Ok(roles)
}

/// A parsed `--sink` value.
///
/// The transport variants parse here and are refused later as configuration
/// errors naming the slice that delivers them (specification FR-011), rather
/// than at parse time, so the message can name the slice rather than reject the
/// syntax as unknown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SinkSpec {
    /// A pcapng capture file. `file:` or `pcapng:`.
    File(PathBuf),
    /// A JSON Lines metadata file. `jsonl:`.
    JsonLines(PathBuf),
    /// A named pipe transport. `pipe:`. Deferred to slice S15.
    Pipe(String),
    /// A TCP transport. `tcp://host:port`. Deferred to slice S15.
    Tcp(String),
}

/// Parse a `--sink` specification.
///
/// The form is `<scheme>:<target>`, with `tcp://` carrying the authority form.
/// A missing or unknown scheme is a usage error, because guessing one from the
/// target would route a capture to a destination the operator did not name.
pub fn parse_sink(raw: &str) -> Result<SinkSpec, String> {
    if let Some(rest) = raw.strip_prefix("tcp://") {
        if rest.is_empty() {
            return Err("`tcp://` needs a host and port".to_string());
        }
        return Ok(SinkSpec::Tcp(rest.to_string()));
    }
    let Some((scheme, target)) = raw.split_once(':') else {
        return Err(format!(
            "`{raw}` has no sink scheme; expected one of file:, pcapng:, jsonl:, pipe:, tcp://"
        ));
    };
    if target.is_empty() {
        return Err(format!("`{raw}` names a scheme but no target"));
    }
    match scheme {
        "file" | "pcapng" => Ok(SinkSpec::File(PathBuf::from(target))),
        "jsonl" => Ok(SinkSpec::JsonLines(PathBuf::from(target))),
        "pipe" => Ok(SinkSpec::Pipe(target.to_string())),
        other => Err(format!(
            "`{other}:` is not a sink scheme; expected one of file:, pcapng:, jsonl:, pipe:, tcp://"
        )),
    }
}

/// A ring window, either a duration or a size.
///
/// Accepted and validated now; ring mode itself is refused as a configuration
/// error naming slice S16.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingWindow {
    /// A time window.
    Duration(Duration),
    /// A size window, in bytes.
    Size(u64),
}

/// Parse a ring window: a duration literal or a size literal.
///
/// A duration is tried first. A value that is neither is reported with both
/// grammars, because either could have been intended.
pub fn parse_ring(raw: &str) -> Result<RingWindow, String> {
    if let Ok(d) = duration::parse(raw) {
        return Ok(RingWindow::Duration(d));
    }
    match size::parse(raw) {
        Ok(bytes) => Ok(RingWindow::Size(bytes)),
        Err(_) => Err(format!(
            "`{raw}` is not a ring window; expected a duration (for example 30s) or a size \
             (for example 64mb)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_and_size_delegate_to_the_core_grammars() {
        assert_eq!(parse_duration("30s"), Ok(Duration::from_secs(30)));
        assert!(parse_duration("30").is_err());
        assert_eq!(parse_size("4mb"), Ok(4 * 1024 * 1024));
        assert!(parse_size("4").is_err());
    }

    #[test]
    fn roles_split_on_commas_and_reject_empties() {
        assert_eq!(
            parse_roles("client,launcher"),
            Ok(vec!["client".to_string(), "launcher".to_string()])
        );
        assert!(parse_roles("client,").is_err());
        assert!(parse_roles("").is_err());
    }

    #[test]
    fn sink_schemes_parse_to_their_kinds() {
        assert_eq!(
            parse_sink("file:out.fcapng"),
            Ok(SinkSpec::File(PathBuf::from("out.fcapng")))
        );
        assert_eq!(
            parse_sink("pcapng:out.fcapng"),
            Ok(SinkSpec::File(PathBuf::from("out.fcapng")))
        );
        assert_eq!(
            parse_sink("jsonl:out.jsonl"),
            Ok(SinkSpec::JsonLines(PathBuf::from("out.jsonl")))
        );
        assert_eq!(
            parse_sink("pipe:fragcap"),
            Ok(SinkSpec::Pipe("fragcap".to_string()))
        );
        assert_eq!(
            parse_sink("tcp://127.0.0.1:9000"),
            Ok(SinkSpec::Tcp("127.0.0.1:9000".to_string()))
        );
        assert!(
            parse_sink("out.fcapng").is_err(),
            "a bare path has no scheme"
        );
        assert!(parse_sink("bogus:x").is_err());
    }

    #[test]
    fn a_ring_window_accepts_a_duration_or_a_size() {
        assert_eq!(
            parse_ring("30s"),
            Ok(RingWindow::Duration(Duration::from_secs(30)))
        );
        assert_eq!(parse_ring("64mb"), Ok(RingWindow::Size(64 * 1024 * 1024)));
        assert!(parse_ring("nonsense").is_err());
    }
}
