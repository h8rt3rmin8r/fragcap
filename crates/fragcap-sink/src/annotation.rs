// SPDX-License-Identifier: Apache-2.0

//! The attribution annotation of specification section 13.3.
//!
//! An annotation is a value here, not a format string. It is derived from a
//! packet by [`Annotation::from_packet`] and rendered by [`Annotation::encode`],
//! and those are two operations rather than one because section 13.5's JSON
//! Lines output carries the same facts in different syntax. Two independent
//! derivations of which keys are present would drift, and the drift would be
//! silent, since each would be internally consistent.
//!
//! [`Annotation::decode`] exists so the grammar has a second implementation to
//! disagree with. A round trip through one function proves nothing.
//!
//! This module knows nothing about pcapng. It produces a string; where that
//! string goes is the writer's business.

use std::fmt;
use std::sync::Arc;

use fragcap_core::attribution::StageId;
use fragcap_core::packet::{AttributionState, CapturedPacket};
use fragcap_core::Direction;

/// The sentinel every annotation begins with.
pub const SENTINEL: &str = "fragcap:";

/// How attribution was obtained. Specification section 13.4.
///
/// Never inferred by a consumer and never inferred by the writer. The value
/// records what the pipeline resolved, because the distinction between an
/// observation and an inference is the thing a reader cannot reconstruct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fidelity {
    /// The endpoint was present in the socket table at the time of resolution.
    Live,
    /// Resolved from the grace period map of specification section 11.4.
    ///
    /// Inferential: an endpoint that closed and was reassigned to a different
    /// process inside the grace period attributes incorrectly. Marking it lets
    /// analysis discount it where precision matters.
    ///
    /// Not produced by this slice. The grace period map arrives with the socket
    /// table attributor; the value exists here so that slice supplies data
    /// rather than widening a grammar.
    Retained,
    /// The packet could not be attributed. Implies no `pid`, `proc`, `role`, or
    /// `stage`.
    None,
}

impl Fidelity {
    fn as_str(self) -> &'static str {
        match self {
            Fidelity::Live => "live",
            Fidelity::Retained => "retained",
            Fidelity::None => "none",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "live" => Some(Fidelity::Live),
            "retained" => Some(Fidelity::Retained),
            "none" => Some(Fidelity::None),
            _ => None,
        }
    }
}

/// Direction as the file records it.
///
/// Distinct from core's [`Direction`], which has two variants, because the file
/// must express two states the type does not. Section 13.3 marks `dir` as
/// always present, so there is no option of omitting it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnotatedDirection {
    In,
    Out,
    /// Both endpoints on the capturing host.
    ///
    /// Not produced by this slice. Section 12.6 leaves loopback direction
    /// undetermined until it can be resolved from the attributed process's
    /// endpoint.
    Local,
    /// The pipeline determined no direction.
    ///
    /// Distinct from [`AnnotatedDirection::Local`] on purpose. "Not determined"
    /// and "loopback" are different facts, and reporting the second from the
    /// first is the substitution constitution P-9 exists to block.
    Unknown,
}

impl AnnotatedDirection {
    fn as_str(self) -> &'static str {
        match self {
            AnnotatedDirection::In => "in",
            AnnotatedDirection::Out => "out",
            AnnotatedDirection::Local => "local",
            AnnotatedDirection::Unknown => "unknown",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "in" => Some(AnnotatedDirection::In),
            "out" => Some(AnnotatedDirection::Out),
            "local" => Some(AnnotatedDirection::Local),
            "unknown" => Some(AnnotatedDirection::Unknown),
            _ => None,
        }
    }
}

impl From<Option<Direction>> for AnnotatedDirection {
    fn from(d: Option<Direction>) -> Self {
        match d {
            Some(Direction::Inbound) => AnnotatedDirection::In,
            Some(Direction::Outbound) => AnnotatedDirection::Out,
            None => AnnotatedDirection::Unknown,
        }
    }
}

/// The attribution facts for one observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation {
    /// Present exactly when the packet was attributed. Never present without
    /// `process`.
    pub pid: Option<u32>,
    /// Present exactly when the packet was attributed.
    pub process: Option<Arc<str>>,
    /// Present when a role is, decided independently of `stage`.
    pub role: Option<Arc<str>>,
    /// Present when a stage is, decided independently of `role`.
    pub stage: Option<StageId>,
    /// Always present.
    pub direction: AnnotatedDirection,
    /// Always present.
    pub fidelity: Fidelity,
    /// Present when the capture holds more than one interface.
    pub interface: Option<Arc<str>>,
}

impl Annotation {
    /// Derive from a packet.
    ///
    /// This is the reusable half. Which keys are present follows from the
    /// packet, never from the caller, so a second output format cannot restate
    /// the rules differently.
    ///
    /// `interface` is supplied by the writer, which is the only party that
    /// knows how many interfaces the capture holds. Passing `None` means a
    /// single-interface capture, where section 13.3 omits the key.
    pub fn from_packet(packet: &CapturedPacket, interface: Option<&str>) -> Self {
        let attr = packet.attribution.as_ref();
        Annotation {
            pid: attr.map(|a| a.pid),
            process: attr.map(|a| Arc::clone(&a.process)),
            role: attr.and_then(|a| a.role.clone()),
            stage: attr.and_then(|a| a.stage.clone()),
            direction: packet.direction.into(),
            fidelity: match packet.attribution_state() {
                // Resolution here is always from a live table: the grace period
                // map does not exist yet. When it does, the pipeline tells the
                // writer which it was rather than the writer guessing.
                AttributionState::Resolved => Fidelity::Live,
                AttributionState::Unresolved | AttributionState::NotAttempted => Fidelity::None,
            },
            interface: interface.map(Arc::from),
        }
    }

    /// Render to the section 13.3 grammar.
    ///
    /// Keys appear in the order of the section 13.3 table, and present keys keep
    /// that relative order regardless of which are absent. The order is fixed
    /// because byte-identical output requires it and a key-value map would not
    /// supply one.
    pub fn encode(&self) -> String {
        let mut out = String::from(SENTINEL);
        let mut first = true;
        let mut pair = |k: &str, v: &str, out: &mut String| {
            if !first {
                out.push(';');
            }
            first = false;
            out.push_str(k);
            out.push('=');
            encode_value_into(v, out);
        };

        if let Some(pid) = self.pid {
            pair("pid", &pid.to_string(), &mut out);
        }
        if let Some(p) = &self.process {
            pair("proc", p, &mut out);
        }
        if let Some(r) = &self.role {
            pair("role", r, &mut out);
        }
        if let Some(s) = &self.stage {
            pair("stage", s.as_str(), &mut out);
        }
        pair("dir", self.direction.as_str(), &mut out);
        pair("attr", self.fidelity.as_str(), &mut out);
        if let Some(i) = &self.interface {
            pair("iface", i, &mut out);
        }
        out
    }

    /// Parse the section 13.3 grammar.
    ///
    /// Liberal about percent-encoding case, because it reads files other tools
    /// may have written. Strict about everything else, because it is the
    /// encoder's round-trip partner and a lenient parser would let an encoder
    /// defect pass.
    pub fn decode(s: &str) -> Result<Self, AnnotationError> {
        let body = s
            .strip_prefix(SENTINEL)
            .ok_or(AnnotationError::MissingSentinel)?;

        let mut pid = None;
        let mut process = None;
        let mut role = None;
        let mut stage = None;
        let mut direction = None;
        let mut fidelity = None;
        let mut interface = None;

        if !body.is_empty() {
            for part in body.split(';') {
                let (key, raw) = part
                    .split_once('=')
                    .ok_or_else(|| AnnotationError::MalformedPair(part.to_string()))?;
                let value = decode_value(raw)?;
                match key {
                    "pid" => {
                        pid = Some(
                            value
                                .parse()
                                .map_err(|_| AnnotationError::BadValue("pid", value.clone()))?,
                        )
                    }
                    "proc" => process = Some(Arc::from(value.as_str())),
                    "role" => role = Some(Arc::from(value.as_str())),
                    "stage" => stage = Some(StageId::new(&value)),
                    "dir" => {
                        direction = Some(
                            AnnotatedDirection::parse(&value)
                                .ok_or_else(|| AnnotationError::BadValue("dir", value.clone()))?,
                        )
                    }
                    "attr" => {
                        fidelity = Some(
                            Fidelity::parse(&value)
                                .ok_or_else(|| AnnotationError::BadValue("attr", value.clone()))?,
                        )
                    }
                    "iface" => interface = Some(Arc::from(value.as_str())),
                    other => return Err(AnnotationError::UnknownKey(other.to_string())),
                }
            }
        }

        Ok(Annotation {
            pid,
            process,
            role,
            stage,
            direction: direction.ok_or(AnnotationError::MissingKey("dir"))?,
            fidelity: fidelity.ok_or(AnnotationError::MissingKey("attr"))?,
            interface,
        })
    }
}

/// Whether a character has to be escaped.
///
/// The three the specification names break the grammar. The control characters
/// break the containing format: pcapng defines a comment as UTF-8 text, and a
/// reader meeting a NUL or a newline mid-comment behaves unpredictably.
/// Percent-encoding is lossless and reversible, so widening the set preserves
/// the observation rather than altering it.
fn must_escape(c: char) -> bool {
    matches!(c, ';' | '=' | '%') || (c as u32) < 0x20 || c == '\u{7F}'
}

fn encode_value_into(value: &str, out: &mut String) {
    for c in value.chars() {
        if must_escape(c) {
            let mut buf = [0u8; 4];
            for b in c.encode_utf8(&mut buf).as_bytes() {
                out.push('%');
                out.push_str(&format!("{b:02X}"));
            }
        } else {
            out.push(c);
        }
    }
}

fn decode_value(raw: &str) -> Result<String, AnnotationError> {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return Err(AnnotationError::MalformedEscape(raw.to_string()));
            }
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3])
                .map_err(|_| AnnotationError::MalformedEscape(raw.to_string()))?;
            let b = u8::from_str_radix(hex, 16)
                .map_err(|_| AnnotationError::MalformedEscape(raw.to_string()))?;
            out.push(b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| AnnotationError::MalformedEscape(raw.to_string()))
}

/// What a malformed annotation looks like.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnnotationError {
    MissingSentinel,
    MalformedPair(String),
    MalformedEscape(String),
    UnknownKey(String),
    BadValue(&'static str, String),
    MissingKey(&'static str),
}

impl fmt::Display for AnnotationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnnotationError::MissingSentinel => {
                write!(f, "annotation does not begin with `{SENTINEL}`")
            }
            AnnotationError::MalformedPair(p) => write!(f, "`{p}` is not a key=value pair"),
            AnnotationError::MalformedEscape(v) => {
                write!(f, "`{v}` contains a malformed percent escape")
            }
            AnnotationError::UnknownKey(k) => write!(f, "unknown annotation key `{k}`"),
            AnnotationError::BadValue(k, v) => write!(f, "`{v}` is not a valid `{k}` value"),
            AnnotationError::MissingKey(k) => write!(f, "annotation is missing required key `{k}`"),
        }
    }
}

impl std::error::Error for AnnotationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_core::attribution::Attribution;
    use fragcap_core::flow::{FlowKey, Proto};
    use fragcap_core::packet::{Payload, RawPacket, Timestamp};

    fn packet(attr: Option<Attribution>, dir: Option<Direction>, flow: bool) -> CapturedPacket {
        let raw = RawPacket::new(Timestamp::from_parts(1, 0), Payload::from(vec![0u8; 4]), 4);
        let mut p = CapturedPacket::from_raw(raw);
        if flow {
            p.flow = Some(FlowKey::new(
                Proto::Tcp,
                "192.0.2.1:1000".parse().unwrap(),
                "198.51.100.1:80".parse().unwrap(),
            ));
        }
        p.direction = dir;
        p.attribution = attr;
        p
    }

    fn full() -> Annotation {
        Annotation {
            pid: Some(7412),
            process: Some(Arc::from("eso64.exe")),
            role: Some(Arc::from("client")),
            stage: Some(StageId::new("play")),
            direction: AnnotatedDirection::Out,
            fidelity: Fidelity::Live,
            interface: Some(Arc::from("Ethernet 2")),
        }
    }

    // --- grammar: rendering -------------------------------------------------

    #[test]
    fn the_specification_example_renders_exactly() {
        // Specification section 13.3 prints this string. If this test fails the
        // documentation is wrong or the encoder is.
        let a = Annotation {
            pid: Some(7412),
            process: Some(Arc::from("eso64.exe")),
            role: Some(Arc::from("client")),
            stage: None,
            direction: AnnotatedDirection::Out,
            fidelity: Fidelity::Live,
            interface: None,
        };
        assert_eq!(
            a.encode(),
            "fragcap:pid=7412;proc=eso64.exe;role=client;dir=out;attr=live"
        );
    }

    #[test]
    fn keys_appear_in_table_order_with_absent_keys_skipped() {
        let encoded = full().encode();
        let keys: Vec<&str> = encoded
            .strip_prefix(SENTINEL)
            .unwrap()
            .split(';')
            .map(|p| p.split_once('=').unwrap().0)
            .collect();
        assert_eq!(
            keys,
            ["pid", "proc", "role", "stage", "dir", "attr", "iface"]
        );

        // Dropping the middle keys must not reorder the rest.
        let mut sparse = full();
        sparse.role = None;
        sparse.stage = None;
        sparse.interface = None;
        let encoded = sparse.encode();
        let keys: Vec<&str> = encoded
            .strip_prefix(SENTINEL)
            .unwrap()
            .split(';')
            .map(|p| p.split_once('=').unwrap().0)
            .collect();
        assert_eq!(keys, ["pid", "proc", "dir", "attr"]);
    }

    #[test]
    fn every_key_is_lowercase_ascii() {
        for part in full().encode().strip_prefix(SENTINEL).unwrap().split(';') {
            let key = part.split_once('=').unwrap().0;
            assert!(
                key.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "key `{key}` is not lowercase ASCII"
            );
        }
    }

    #[test]
    fn every_annotation_begins_with_the_sentinel() {
        assert!(full().encode().starts_with(SENTINEL));
    }

    // --- grammar: escaping --------------------------------------------------

    #[test]
    fn the_three_reserved_characters_are_escaped() {
        let mut a = full();
        a.process = Some(Arc::from("od;d=na%me.exe"));
        assert!(a.encode().contains("proc=od%3Bd%3Dna%25me.exe"));
    }

    #[test]
    fn control_characters_are_escaped() {
        // A newline or a NUL inside a pcapng comment is what breaks a reader,
        // and the specification's three characters do not cover it.
        for (c, esc) in [
            ('\n', "%0A"),
            ('\0', "%00"),
            ('\u{7F}', "%7F"),
            ('\t', "%09"),
        ] {
            let mut a = full();
            a.process = Some(Arc::from(format!("a{c}b").as_str()));
            let encoded = a.encode();
            assert!(
                encoded.contains(&format!("proc=a{esc}b")),
                "{c:?} should render as {esc}, got {encoded}"
            );
        }
    }

    #[test]
    fn escapes_are_uppercase_on_output() {
        let mut a = full();
        a.process = Some(Arc::from(";"));
        assert!(
            a.encode().contains("%3B"),
            "uppercase hex keeps goldens stable"
        );
    }

    #[test]
    fn the_decoder_accepts_either_escape_case() {
        // It reads files other tools wrote, so rejecting lowercase would be
        // wrong even though this writer never emits it.
        let upper = Annotation::decode("fragcap:proc=a%3Bb;dir=out;attr=none").unwrap();
        let lower = Annotation::decode("fragcap:proc=a%3bb;dir=out;attr=none").unwrap();
        assert_eq!(upper.process.as_deref(), Some("a;b"));
        assert_eq!(upper, lower);
    }

    #[test]
    fn ordinary_characters_are_not_escaped() {
        let mut a = full();
        a.process = Some(Arc::from("eso64.exe"));
        assert!(a.encode().contains("proc=eso64.exe"));
    }

    // --- grammar: decoding --------------------------------------------------

    #[test]
    fn a_missing_sentinel_is_rejected() {
        assert_eq!(
            Annotation::decode("pid=1;dir=out;attr=live"),
            Err(AnnotationError::MissingSentinel)
        );
    }

    #[test]
    fn an_unknown_key_is_rejected() {
        assert_eq!(
            Annotation::decode("fragcap:dir=out;attr=none;wat=1"),
            Err(AnnotationError::UnknownKey("wat".into()))
        );
    }

    #[test]
    fn a_malformed_escape_is_rejected() {
        assert!(matches!(
            Annotation::decode("fragcap:proc=a%ZZb;dir=out;attr=none"),
            Err(AnnotationError::MalformedEscape(_))
        ));
        assert!(matches!(
            Annotation::decode("fragcap:proc=trailing%;dir=out;attr=none"),
            Err(AnnotationError::MalformedEscape(_))
        ));
    }

    #[test]
    fn a_pair_without_an_equals_is_rejected() {
        assert!(matches!(
            Annotation::decode("fragcap:dir=out;attr=none;lonely"),
            Err(AnnotationError::MalformedPair(_))
        ));
    }

    #[test]
    fn a_missing_required_key_is_rejected() {
        assert_eq!(
            Annotation::decode("fragcap:pid=1;proc=a;attr=live"),
            Err(AnnotationError::MissingKey("dir"))
        );
        assert_eq!(
            Annotation::decode("fragcap:pid=1;proc=a;dir=out"),
            Err(AnnotationError::MissingKey("attr"))
        );
    }

    #[test]
    fn a_bad_enumerated_value_is_rejected() {
        assert!(matches!(
            Annotation::decode("fragcap:dir=sideways;attr=none"),
            Err(AnnotationError::BadValue("dir", _))
        ));
        assert!(matches!(
            Annotation::decode("fragcap:dir=out;attr=probably"),
            Err(AnnotationError::BadValue("attr", _))
        ));
    }

    // --- grammar: round trip ------------------------------------------------

    #[test]
    fn every_key_round_trips() {
        let a = full();
        assert_eq!(Annotation::decode(&a.encode()).unwrap(), a);
    }

    #[test]
    fn every_reserved_character_round_trips() {
        for c in [';', '=', '%', '\n', '\0', '\u{7F}', '\u{1}'] {
            let mut a = full();
            a.process = Some(Arc::from(format!("x{c}y").as_str()));
            let round = Annotation::decode(&a.encode()).unwrap();
            assert_eq!(round, a, "{c:?} did not survive the round trip");
        }
    }

    #[test]
    fn multi_byte_characters_round_trip() {
        let mut a = full();
        a.process = Some(Arc::from("\u{30b2}\u{30fc}\u{30e0}.exe"));
        assert_eq!(Annotation::decode(&a.encode()).unwrap(), a);
    }

    #[test]
    fn every_direction_and_fidelity_value_round_trips() {
        for d in [
            AnnotatedDirection::In,
            AnnotatedDirection::Out,
            AnnotatedDirection::Local,
            AnnotatedDirection::Unknown,
        ] {
            for fid in [Fidelity::Live, Fidelity::Retained, Fidelity::None] {
                let a = Annotation {
                    pid: None,
                    process: None,
                    role: None,
                    stage: None,
                    direction: d,
                    fidelity: fid,
                    interface: None,
                };
                assert_eq!(Annotation::decode(&a.encode()).unwrap(), a);
            }
        }
    }

    #[test]
    fn an_empty_value_keeps_its_key() {
        // Omitting `proc` would report that the packet was not attributed,
        // which is a different fact about the observation.
        let mut a = full();
        a.process = Some(Arc::from(""));
        let encoded = a.encode();
        assert!(encoded.contains("proc=;"), "got {encoded}");
        assert_eq!(Annotation::decode(&encoded).unwrap(), a);
    }

    // --- derivation ---------------------------------------------------------

    #[test]
    fn identity_keys_are_present_exactly_when_attributed() {
        let attributed = Annotation::from_packet(
            &packet(Some(Attribution::new(7412, "eso64.exe")), None, true),
            None,
        );
        assert_eq!(attributed.pid, Some(7412));
        assert_eq!(attributed.process.as_deref(), Some("eso64.exe"));

        let unattributed = Annotation::from_packet(&packet(None, None, true), None);
        assert!(unattributed.pid.is_none());
        assert!(unattributed.process.is_none());
    }

    #[test]
    fn identity_keys_are_never_present_individually() {
        for p in [
            packet(Some(Attribution::new(1, "a.exe")), None, true),
            packet(None, None, true),
            packet(None, None, false),
        ] {
            let a = Annotation::from_packet(&p, None);
            assert_eq!(
                a.pid.is_some(),
                a.process.is_some(),
                "pid and proc must appear together"
            );
        }
    }

    #[test]
    fn role_and_stage_are_decided_independently() {
        // Section 13.3 presents them as a pair. `Attribution` does not, and the
        // type is what the data actually looks like.
        let role_only = Attribution::new(1, "a.exe").with_role("client");
        let a = Annotation::from_packet(&packet(Some(role_only), None, true), None);
        assert_eq!(a.role.as_deref(), Some("client"));
        assert!(a.stage.is_none(), "a role must not imply a stage");

        let stage_only = Attribution::new(1, "a.exe").with_stage(StageId::new("play"));
        let a = Annotation::from_packet(&packet(Some(stage_only), None, true), None);
        assert!(a.role.is_none(), "a stage must not imply a role");
        assert_eq!(a.stage.as_ref().map(|s| s.as_str()), Some("play"));

        let both = Attribution::new(1, "a.exe")
            .with_role("client")
            .with_stage(StageId::new("play"));
        let a = Annotation::from_packet(&packet(Some(both), None, true), None);
        assert!(a.role.is_some() && a.stage.is_some());
    }

    #[test]
    fn direction_maps_without_inventing_loopback() {
        let cases = [
            (Some(Direction::Inbound), AnnotatedDirection::In),
            (Some(Direction::Outbound), AnnotatedDirection::Out),
            (None, AnnotatedDirection::Unknown),
        ];
        for (core, expected) in cases {
            let a = Annotation::from_packet(&packet(None, core, true), None);
            assert_eq!(a.direction, expected);
            assert_ne!(
                a.direction,
                AnnotatedDirection::Local,
                "an undetermined direction is not loopback"
            );
        }
    }

    #[test]
    fn unattributed_packets_carry_no_identity_keys() {
        for p in [packet(None, None, true), packet(None, None, false)] {
            let encoded = Annotation::from_packet(&p, None).encode();
            assert!(encoded.contains("attr=none"), "got {encoded}");
            for key in ["pid=", "proc=", "role=", "stage="] {
                assert!(!encoded.contains(key), "{key} present in {encoded}");
            }
        }
    }

    #[test]
    fn dir_and_attr_are_present_in_every_state() {
        // The guarantee that lets a consumer parse without a presence check.
        let cases = [
            packet(
                Some(Attribution::new(1, "a.exe")),
                Some(Direction::Inbound),
                true,
            ),
            packet(None, Some(Direction::Outbound), true),
            packet(None, None, false),
        ];
        for p in cases {
            let encoded = Annotation::from_packet(&p, None).encode();
            assert!(
                encoded.contains(";dir=") || encoded.contains("fragcap:dir="),
                "{encoded}"
            );
            assert!(encoded.contains("attr="), "{encoded}");
        }
    }

    #[test]
    fn iface_appears_only_when_the_writer_supplies_one() {
        let p = packet(None, None, true);
        assert!(!Annotation::from_packet(&p, None)
            .encode()
            .contains("iface="));
        assert!(Annotation::from_packet(&p, Some("Ethernet 2"))
            .encode()
            .contains("iface=Ethernet 2"));
    }

    #[test]
    fn fidelity_follows_the_attribution_state_and_is_never_upgraded() {
        let resolved = packet(Some(Attribution::new(1, "a.exe")), None, true);
        assert_eq!(
            Annotation::from_packet(&resolved, None).fidelity,
            Fidelity::Live
        );

        // A flow key with no attribution resolved, and no flow key at all, are
        // different parse outcomes but the same fidelity: nothing was learned.
        for p in [packet(None, None, true), packet(None, None, false)] {
            assert_eq!(Annotation::from_packet(&p, None).fidelity, Fidelity::None);
        }
    }

    #[test]
    fn derivation_is_usable_without_rendering() {
        // What S07 needs: the key set follows from the packet, and turning it
        // into bytes is somebody else's step.
        let a = Annotation::from_packet(&packet(None, None, false), None);
        assert_eq!(a.fidelity, Fidelity::None);
        assert_eq!(a.direction, AnnotatedDirection::Unknown);
        assert!(a.pid.is_none());
    }
}
