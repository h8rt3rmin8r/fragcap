// SPDX-License-Identifier: Apache-2.0

//! The attribution script format: what a scripted attributor answers, for
//! which flow, in which time window.
//!
//! Specification section 25.3 requires every fixture be paired with a script
//! "declaring what the scripted attributor returns for each flow at each point
//! in time, which is how port reuse and retained attribution become testable".
//! It does not define one. This is that definition, recorded for promotion to
//! specification section 29.
//!
//! # Why not TOML
//!
//! TOML is the better format and the wrong choice here. Adopting it means
//! adopting a parser and its proc-macro dependencies on behalf of slice S05,
//! which owns the profile schema and should choose against the profile's
//! requirements rather than inherit a choice made for a test fixture.
//!
//! This is not a user-facing format. A script ships beside the fixture it
//! describes, is written by the corpus generator, and is read by one type. It
//! is deliberately trivial, and the moment it wants nesting or types it should
//! become TOML rather than growing.
//!
//! # Grammar
//!
//! ```text
//! # comments and blank lines are ignored
//! flow <proto> <local> <remote|*> <window> owner <pid> <name>
//! flow <proto> <local> <remote|*> <window> unowned
//! endpoint <proto> <addr>
//! ```
//!
//! `<window>` is `always` or `<from>..<to>`, half-open, in nanoseconds since
//! the Unix epoch, on the same base as the packet timestamps the fixtures
//! carry. Half-open so that two adjacent windows do not overlap, which is
//! exactly what a port reuse script needs to say.
//!
//! # What cannot be written
//!
//! A UDP entry naming a remote endpoint, and a TCP entry without one. Neither
//! corresponds to anything a socket table can answer: specification section 8.4
//! gives TCP both endpoints and UDP the local one alone. Rejecting them at load
//! is what stops a script demanding behavior the real attributor in S10 must
//! never implement.

use std::fmt;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use fragcap_core::attribution::{Attribution, Fidelity};
use fragcap_core::flow::{AttributionKey, Endpoint, FlowKey, Proto};
use fragcap_core::packet::Timestamp;

/// Why a script would not load.
///
/// Every variant carries its line, because a script is authored by hand as
/// often as it is generated and a parse failure with no location is a hunt.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScriptError {
    /// The first word of the line is not a statement this format defines.
    UnknownStatement { line: usize, found: String },
    /// A statement with the wrong number of words.
    WrongArity {
        line: usize,
        statement: &'static str,
        expected: &'static str,
    },
    /// Not `tcp` or `udp`.
    BadProtocol { line: usize, found: String },
    /// Not an address and port.
    BadAddress { line: usize, found: String },
    /// Not `always` or `<from>..<to>`, or a range that ends before it starts.
    BadWindow { line: usize, found: String },
    /// Not a process identifier.
    BadPid { line: usize, found: String },
    /// A protocol and remote endpoint combination no socket table could answer.
    UnanswerableFlow { line: usize, detail: String },
    /// Two entries could both match one flow at one instant.
    OverlappingWindows { line: usize, other: usize },
    /// The file could not be read.
    Io { detail: String },
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::UnknownStatement { line, found } => {
                write!(f, "line {line}: unknown statement {found:?}")
            }
            ScriptError::WrongArity {
                line,
                statement,
                expected,
            } => write!(f, "line {line}: {statement} expects {expected}"),
            ScriptError::BadProtocol { line, found } => {
                write!(f, "line {line}: {found:?} is not tcp or udp")
            }
            ScriptError::BadAddress { line, found } => {
                write!(f, "line {line}: {found:?} is not an address and port")
            }
            ScriptError::BadWindow { line, found } => {
                write!(f, "line {line}: {found:?} is not always or from..to")
            }
            ScriptError::BadPid { line, found } => {
                write!(f, "line {line}: {found:?} is not a process identifier")
            }
            ScriptError::UnanswerableFlow { line, detail } => {
                write!(f, "line {line}: {detail}")
            }
            ScriptError::OverlappingWindows { line, other } => write!(
                f,
                "line {line}: overlaps line {other}; one flow cannot have two owners at one instant"
            ),
            ScriptError::Io { detail } => write!(f, "cannot read script: {detail}"),
        }
    }
}

impl std::error::Error for ScriptError {}

/// When an entry applies.
///
/// Half-open, so a window ending at an instant and another starting at the same
/// instant do not overlap. That is what a port reuse script wants to express,
/// and it would be fiddly to state with closed intervals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Window {
    /// Every instant. A script that never mentions time is entirely this, so a
    /// caller that never sets a clock still resolves.
    Always,
    /// `[from, to)`, nanoseconds since the Unix epoch.
    Range { from: i64, to: i64 },
}

impl Window {
    pub fn contains(&self, at: Timestamp) -> bool {
        match self {
            Window::Always => true,
            Window::Range { from, to } => {
                let t = at.as_nanos();
                t >= *from && t < *to
            }
        }
    }

    /// Whether two windows share any instant.
    fn intersects(&self, other: &Window) -> bool {
        match (self, other) {
            (Window::Always, _) | (_, Window::Always) => true,
            (Window::Range { from: a, to: b }, Window::Range { from: c, to: d }) => a < d && c < b,
        }
    }
}

/// One declaration: a flow, a window, and what to answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptEntry {
    pub proto: Proto,
    pub local: SocketAddr,
    /// `Some` for TCP, `None` for UDP. The asymmetry is not a convenience: it
    /// mirrors [`AttributionKey`], so the double cannot express an attribution
    /// the platform could never supply.
    pub remote: Option<SocketAddr>,
    pub window: Window,
    /// `None` is an explicit declaration that the flow is unattributed, which
    /// reads differently in a fixture from simply not mentioning it.
    pub owner: Option<Attribution>,
    pub line: usize,
}

impl ScriptEntry {
    /// Whether this entry answers for a flow.
    ///
    /// Goes through [`AttributionKey`] rather than comparing endpoints
    /// directly, so the wildcard bind allowance in specification section 8.4
    /// applies here exactly as it will in the real attributor. A test that
    /// passes against a script is therefore a test S10 has to satisfy.
    pub fn matches(&self, key: &FlowKey) -> bool {
        if key.proto != self.proto {
            return false;
        }
        match (key.attribution_key(), self.remote) {
            (AttributionKey::Pair(_, remote), Some(want)) => {
                remote == want && key.attribution_key().local_matches_bind(self.local)
            }
            (AttributionKey::Local(_), None) => {
                key.attribution_key().local_matches_bind(self.local)
            }
            _ => false,
        }
    }

    /// Whether two entries could both answer for some single flow.
    ///
    /// Exact endpoint equality is not enough. An entry bound to a wildcard
    /// address and one bound to a specific address with the same port both
    /// match the same datagram, so grouping by exact local endpoint would let
    /// that ambiguity through.
    fn could_both_match(&self, other: &ScriptEntry) -> bool {
        if self.proto != other.proto || self.remote != other.remote {
            return false;
        }
        if self.local.port() != other.local.port() {
            return false;
        }
        self.local.ip() == other.local.ip()
            || self.local.ip().is_unspecified()
            || other.local.ip().is_unspecified()
    }
}

/// A parsed attribution script.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AttributionScript {
    entries: Vec<ScriptEntry>,
    endpoints: Vec<Endpoint>,
}

impl AttributionScript {
    pub fn parse(text: &str) -> Result<Self, ScriptError> {
        let mut script = AttributionScript::default();

        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let content = raw.trim();
            if content.is_empty() || content.starts_with('#') {
                continue;
            }
            let words: Vec<&str> = content.split_whitespace().collect();
            match words[0] {
                "flow" => script.entries.push(parse_flow(line, &words)?),
                "endpoint" => script.endpoints.push(parse_endpoint(line, &words)?),
                other => {
                    return Err(ScriptError::UnknownStatement {
                        line,
                        found: other.to_string(),
                    })
                }
            }
        }

        script.check_overlaps()?;
        Ok(script)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ScriptError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|e| ScriptError::Io {
            detail: format!("{}: {e}", path.display()),
        })?;
        Self::parse(&text)
    }

    pub fn entries(&self) -> &[ScriptEntry] {
        &self.entries
    }

    pub fn endpoints(&self) -> &[Endpoint] {
        &self.endpoints
    }

    /// What this script answers for a flow at an instant.
    pub fn resolve(&self, key: &FlowKey, at: Timestamp) -> Option<Attribution> {
        self.entries
            .iter()
            .find(|e| e.window.contains(at) && e.matches(key))
            .and_then(|e| e.owner.clone())
    }

    /// Reject any pair of entries that could both answer for one flow at one
    /// instant.
    ///
    /// Last-one-wins and first-one-wins are both silent, and a script is a
    /// test's statement of intent. Two statements that contradict each other
    /// mean the author believed something untrue about the test they were
    /// writing, and saying so at load is saying so where the mistake was made.
    fn check_overlaps(&self) -> Result<(), ScriptError> {
        for (i, a) in self.entries.iter().enumerate() {
            for b in &self.entries[i + 1..] {
                if a.could_both_match(b) && a.window.intersects(&b.window) {
                    return Err(ScriptError::OverlappingWindows {
                        line: b.line,
                        other: a.line,
                    });
                }
            }
        }
        Ok(())
    }
}

fn parse_proto(line: usize, word: &str) -> Result<Proto, ScriptError> {
    match word {
        "tcp" => Ok(Proto::Tcp),
        "udp" => Ok(Proto::Udp),
        other => Err(ScriptError::BadProtocol {
            line,
            found: other.to_string(),
        }),
    }
}

fn parse_addr(line: usize, word: &str) -> Result<SocketAddr, ScriptError> {
    word.parse().map_err(|_| ScriptError::BadAddress {
        line,
        found: word.to_string(),
    })
}

fn parse_window(line: usize, word: &str) -> Result<Window, ScriptError> {
    if word == "always" {
        return Ok(Window::Always);
    }
    let bad = || ScriptError::BadWindow {
        line,
        found: word.to_string(),
    };
    let (from, to) = word.split_once("..").ok_or_else(bad)?;
    let from: i64 = from.parse().map_err(|_| bad())?;
    let to: i64 = to.parse().map_err(|_| bad())?;
    if to <= from {
        return Err(bad());
    }
    Ok(Window::Range { from, to })
}

fn parse_flow(line: usize, words: &[&str]) -> Result<ScriptEntry, ScriptError> {
    // flow <proto> <local> <remote|*> <window> owner <pid> <name>
    // flow <proto> <local> <remote|*> <window> unowned
    if words.len() < 6 {
        return Err(ScriptError::WrongArity {
            line,
            statement: "flow",
            expected: "<proto> <local> <remote|*> <window> owner <pid> <name> | ... unowned",
        });
    }
    let proto = parse_proto(line, words[1])?;
    let local = parse_addr(line, words[2])?;
    let remote = if words[3] == "*" {
        None
    } else {
        Some(parse_addr(line, words[3])?)
    };

    // The one structural rule. Section 8.4 gives TCP both endpoints and UDP the
    // local one alone, so anything else is a flow no socket table could answer.
    match (proto, remote) {
        (Proto::Udp, Some(_)) => {
            return Err(ScriptError::UnanswerableFlow {
                line,
                detail: "a udp entry cannot name a remote endpoint: the udp socket table \
                         carries none, and inventing one is what specification section 8.4 \
                         forbids"
                    .to_string(),
            })
        }
        (Proto::Tcp, None) => {
            return Err(ScriptError::UnanswerableFlow {
                line,
                detail: "a tcp entry must name a remote endpoint: the tcp socket table \
                         carries one, and matching without it would resolve flows the real \
                         attributor would not"
                    .to_string(),
            })
        }
        _ => {}
    }

    let window = parse_window(line, words[4])?;
    let owner = match words[5] {
        "unowned" => {
            if words.len() != 6 {
                return Err(ScriptError::WrongArity {
                    line,
                    statement: "flow ... unowned",
                    expected: "no words after unowned",
                });
            }
            None
        }
        "owner" => {
            if words.len() != 8 {
                return Err(ScriptError::WrongArity {
                    line,
                    statement: "flow ... owner",
                    expected: "<pid> <name> after owner",
                });
            }
            let pid: u32 = words[6].parse().map_err(|_| ScriptError::BadPid {
                line,
                found: words[6].to_string(),
            })?;
            Some(Attribution::new(pid, words[7], Fidelity::Live))
        }
        other => {
            return Err(ScriptError::UnknownStatement {
                line,
                found: other.to_string(),
            })
        }
    };

    Ok(ScriptEntry {
        proto,
        local,
        remote,
        window,
        owner,
        line,
    })
}

fn parse_endpoint(line: usize, words: &[&str]) -> Result<Endpoint, ScriptError> {
    // endpoint <proto> <addr>
    if words.len() != 3 {
        return Err(ScriptError::WrongArity {
            line,
            statement: "endpoint",
            expected: "<proto> <addr>",
        });
    }
    Ok(Endpoint::new(
        parse_addr(line, words[2])?,
        parse_proto(line, words[1])?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(s: &str) -> SocketAddr {
        s.parse().expect("test address must parse")
    }

    fn tcp_key() -> FlowKey {
        FlowKey::new(
            Proto::Tcp,
            addr("192.0.2.10:51000"),
            addr("198.51.100.5:443"),
        )
    }

    fn udp_key() -> FlowKey {
        FlowKey::new(
            Proto::Udp,
            addr("192.0.2.10:30000"),
            addr("198.51.100.5:5055"),
        )
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let s = AttributionScript::parse(
            "# a comment\n\n   \n# another\nflow tcp 192.0.2.10:51000 198.51.100.5:443 always owner 42 game.exe\n",
        )
        .expect("the script loads");
        assert_eq!(s.entries().len(), 1);
    }

    #[test]
    fn an_always_entry_resolves_at_any_instant() {
        let s = AttributionScript::parse(
            "flow tcp 192.0.2.10:51000 198.51.100.5:443 always owner 42 game.exe",
        )
        .expect("the script loads");
        for t in [i64::MIN, 0, 1_700_000_000_000_000_000, i64::MAX] {
            let got = s.resolve(&tcp_key(), Timestamp::from_nanos(t));
            assert_eq!(got.expect("always means always").pid, 42);
        }
    }

    #[test]
    fn an_unmentioned_flow_resolves_to_nothing() {
        let s = AttributionScript::parse(
            "flow tcp 192.0.2.10:51000 198.51.100.5:443 always owner 42 game.exe",
        )
        .expect("the script loads");
        let other = FlowKey::new(
            Proto::Tcp,
            addr("192.0.2.10:51001"),
            addr("198.51.100.5:443"),
        );
        assert_eq!(s.resolve(&other, Timestamp::from_nanos(0)), None);
    }

    #[test]
    fn an_unowned_entry_resolves_to_nothing() {
        let s =
            AttributionScript::parse("flow tcp 192.0.2.10:51000 198.51.100.5:443 always unowned")
                .expect("the script loads");
        assert_eq!(s.resolve(&tcp_key(), Timestamp::from_nanos(0)), None);
        assert_eq!(s.entries().len(), 1, "it is a declaration, not an absence");
    }

    // FR-021 and SC-006. The reason the time dimension exists.
    #[test]
    fn port_reuse_resolves_to_two_owners_in_two_windows() {
        let s = AttributionScript::parse(
            "flow tcp 192.0.2.10:51000 198.51.100.5:443 100..200 owner 1 first.exe\n\
             flow tcp 192.0.2.10:51000 198.51.100.5:443 200..300 owner 2 second.exe\n",
        )
        .expect("abutting windows do not overlap");
        let at = |t: i64| s.resolve(&tcp_key(), Timestamp::from_nanos(t));
        assert_eq!(at(99), None, "before the first window");
        assert_eq!(at(100).expect("in the first").pid, 1);
        assert_eq!(at(199).expect("still the first").pid, 1);
        assert_eq!(at(200).expect("half-open, so the second").pid, 2);
        assert_eq!(at(299).expect("still the second").pid, 2);
        assert_eq!(at(300), None, "after the last window");
    }

    // FR-021a. The two combinations that must not load.
    #[test]
    fn a_udp_entry_naming_a_remote_endpoint_does_not_load() {
        let e = AttributionScript::parse(
            "flow udp 192.0.2.10:30000 198.51.100.5:5055 always owner 1 game.exe",
        )
        .expect_err("the udp socket table carries no remote");
        assert!(matches!(e, ScriptError::UnanswerableFlow { line: 1, .. }));
        assert!(e.to_string().contains("8.4"));
    }

    #[test]
    fn a_tcp_entry_without_a_remote_endpoint_does_not_load() {
        let e = AttributionScript::parse("flow tcp 192.0.2.10:51000 * always owner 1 game.exe")
            .expect_err("the tcp socket table carries both endpoints");
        assert!(matches!(e, ScriptError::UnanswerableFlow { line: 1, .. }));
    }

    // FR-021b and SC-006a. The allowance the real attributor must also make.
    #[test]
    fn a_udp_wildcard_bind_matches_a_specific_interface_address() {
        let s = AttributionScript::parse("flow udp 0.0.0.0:30000 * always owner 7 game.exe")
            .expect("the script loads");
        let got = s.resolve(&udp_key(), Timestamp::from_nanos(0));
        assert_eq!(
            got.expect("a wildcard bind owns the datagram").pid,
            7,
            "section 8.4 requires both the wildcard and the specific address match"
        );
    }

    #[test]
    fn a_wildcard_bind_on_a_different_port_does_not_match() {
        let s = AttributionScript::parse("flow udp 0.0.0.0:30001 * always owner 7 game.exe")
            .expect("the script loads");
        assert_eq!(s.resolve(&udp_key(), Timestamp::from_nanos(0)), None);
    }

    #[test]
    fn a_tcp_entry_does_not_take_the_wildcard_allowance() {
        let s = AttributionScript::parse(
            "flow tcp 0.0.0.0:51000 198.51.100.5:443 always owner 7 g.exe",
        )
        .expect("the script loads");
        assert_eq!(
            s.resolve(&tcp_key(), Timestamp::from_nanos(0)),
            None,
            "tcp carries both endpoints, so an exact local match is available"
        );
    }

    #[test]
    fn protocols_do_not_cross_match() {
        let s = AttributionScript::parse("flow udp 192.0.2.10:51000 * always owner 7 game.exe")
            .expect("the script loads");
        assert_eq!(s.resolve(&tcp_key(), Timestamp::from_nanos(0)), None);
    }

    // FR-024.
    #[test]
    fn intersecting_windows_for_one_flow_do_not_load() {
        let e = AttributionScript::parse(
            "flow tcp 192.0.2.10:51000 198.51.100.5:443 100..250 owner 1 a.exe\n\
             flow tcp 192.0.2.10:51000 198.51.100.5:443 200..300 owner 2 b.exe\n",
        )
        .expect_err("they share instants 200 to 250");
        assert_eq!(e, ScriptError::OverlappingWindows { line: 2, other: 1 });
    }

    #[test]
    fn always_alongside_a_window_for_one_flow_does_not_load() {
        let e = AttributionScript::parse(
            "flow tcp 192.0.2.10:51000 198.51.100.5:443 always owner 1 a.exe\n\
             flow tcp 192.0.2.10:51000 198.51.100.5:443 200..300 owner 2 b.exe\n",
        )
        .expect_err("always intersects everything");
        assert!(matches!(e, ScriptError::OverlappingWindows { .. }));
    }

    #[test]
    fn a_wildcard_bind_overlapping_a_specific_bind_does_not_load() {
        // Exact endpoint grouping would miss this: both entries answer for the
        // same datagram, so the ambiguity is real even though the locals differ.
        let e = AttributionScript::parse(
            "flow udp 0.0.0.0:30000 * always owner 1 a.exe\n\
             flow udp 192.0.2.10:30000 * always owner 2 b.exe\n",
        )
        .expect_err("both match a datagram on 192.0.2.10:30000");
        assert!(matches!(e, ScriptError::OverlappingWindows { .. }));
    }

    #[test]
    fn different_flows_may_share_a_window() {
        AttributionScript::parse(
            "flow tcp 192.0.2.10:51000 198.51.100.5:443 always owner 1 a.exe\n\
             flow tcp 192.0.2.10:51001 198.51.100.5:443 always owner 2 b.exe\n\
             flow udp 192.0.2.10:30000 * always owner 3 c.exe\n",
        )
        .expect("distinct flows do not overlap each other");
    }

    // FR-023. The statement the analyze gate found had no coverage at all.
    #[test]
    fn endpoints_are_declared_and_reported() {
        let s = AttributionScript::parse(
            "endpoint tcp 192.0.2.10:51000\nendpoint udp 192.0.2.10:30000\n",
        )
        .expect("the script loads");
        assert_eq!(s.endpoints().len(), 2);
        assert_eq!(
            s.endpoints()[0],
            Endpoint::new(addr("192.0.2.10:51000"), Proto::Tcp)
        );
        assert_eq!(s.endpoints()[1].proto, Proto::Udp);
    }

    #[test]
    fn a_malformed_endpoint_names_its_line() {
        let e = AttributionScript::parse("# note\nendpoint tcp not-an-address")
            .expect_err("that is not an address");
        assert_eq!(
            e,
            ScriptError::BadAddress {
                line: 2,
                found: "not-an-address".to_string()
            }
        );
    }

    #[test]
    fn every_malformed_statement_names_its_line_and_cause() {
        let cases: Vec<(&str, &str)> = vec![
            ("wibble tcp", "unknown statement"),
            ("flow", "expects"),
            (
                "flow sctp 192.0.2.10:1 198.51.100.5:2 always unowned",
                "not tcp or udp",
            ),
            (
                "flow tcp nope 198.51.100.5:2 always unowned",
                "not an address",
            ),
            (
                "flow tcp 192.0.2.10:1 198.51.100.5:2 soon unowned",
                "not always",
            ),
            (
                "flow tcp 192.0.2.10:1 198.51.100.5:2 always owner x a.exe",
                "not a process identifier",
            ),
            (
                "flow tcp 192.0.2.10:1 198.51.100.5:2 always maybe 1 a.exe",
                "unknown statement",
            ),
            (
                "flow tcp 192.0.2.10:1 198.51.100.5:2 always unowned extra",
                "expects",
            ),
            (
                "flow tcp 192.0.2.10:1 198.51.100.5:2 always owner 1",
                "expects",
            ),
            ("endpoint tcp", "expects"),
            ("endpoint sctp 192.0.2.10:1", "not tcp or udp"),
        ];
        for (text, expected) in cases {
            let e = AttributionScript::parse(text)
                .expect_err(&format!("{text:?} must not load"))
                .to_string();
            assert!(
                e.contains(expected),
                "{text:?} produced {e:?}, which does not mention {expected:?}"
            );
            assert!(
                e.starts_with("line 1:"),
                "{text:?} produced {e:?}, which does not name its line"
            );
        }
    }

    #[test]
    fn a_window_that_ends_before_it_starts_does_not_load() {
        let e = AttributionScript::parse("flow tcp 192.0.2.10:1 198.51.100.5:2 300..100 unowned")
            .expect_err("a backwards window is not a window");
        assert!(matches!(e, ScriptError::BadWindow { line: 1, .. }));
    }

    #[test]
    fn an_empty_script_loads_and_answers_nothing() {
        let s = AttributionScript::parse("# nothing here\n").expect("an empty script is valid");
        assert!(s.entries().is_empty());
        assert!(s.endpoints().is_empty());
        assert_eq!(s.resolve(&tcp_key(), Timestamp::from_nanos(0)), None);
    }

    #[test]
    fn loading_a_missing_file_names_it() {
        let e = AttributionScript::load("fixtures/definitely-not-here.script")
            .expect_err("a missing script cannot load");
        assert!(e.to_string().contains("definitely-not-here.script"));
    }

    #[test]
    fn windows_are_half_open_at_both_ends() {
        let w = Window::Range { from: 10, to: 20 };
        assert!(!w.contains(Timestamp::from_nanos(9)));
        assert!(w.contains(Timestamp::from_nanos(10)));
        assert!(w.contains(Timestamp::from_nanos(19)));
        assert!(!w.contains(Timestamp::from_nanos(20)));
    }

    #[test]
    fn abutting_windows_do_not_intersect_but_overlapping_ones_do() {
        let a = Window::Range { from: 0, to: 10 };
        let b = Window::Range { from: 10, to: 20 };
        let c = Window::Range { from: 9, to: 20 };
        assert!(!a.intersects(&b));
        assert!(a.intersects(&c));
        assert!(Window::Always.intersects(&a));
        assert!(a.intersects(&Window::Always));
    }
}
