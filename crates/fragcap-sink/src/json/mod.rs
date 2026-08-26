// SPDX-License-Identifier: Apache-2.0

//! The JSON Lines writer of specification section 13.5.
//!
//! One object per packet, one object per line, no enclosing array, preceded by
//! a header object and followed by a trailer object. For consumers that do not
//! read pcapng: shell pipelines, log shippers, and anything that would rather
//! match a line than walk a block structure.
//!
//! **It carries the same facts as the pcapng writer, from the same source.**
//! Which attribution keys are present is decided by [`Annotation`], which S06
//! separated from rendering for exactly this reason. If this module ever
//! decided a presence rule for itself the two formats would drift, and the
//! drift would be silent because each would be internally consistent.
//!
//! Three deliberate differences from the pcapng profile, all in rendering:
//! `iface` appears on every record because a line is self-contained; hex is
//! lowercase; and endpoints appear as `src` and `dst` when direction is known
//! and as `local` and `remote` when it is not.

mod escape;
mod number;

use std::io::Write;

use fragcap_core::flow::Direction;
use fragcap_core::packet::CapturedPacket;
use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::Sink;
use fragcap_core::SinkError;

use crate::annotation::{fidelity_str, Annotation};
use crate::error::WriteError;
use escape::write_hex_string;
use number::render_timestamp;

/// The JSON string escaper, re-exported so the command line's structured-event
/// stream (slice S14) can hand-roll NDJSON over the same escaper the sinks use.
pub use escape::write_json_string;

/// The version string declared in the header record.
pub const VERSION: &str = concat!("fragcap/", env!("CARGO_PKG_VERSION"));

/// Whether packet payloads are written.
///
/// Fixed at construction rather than chosen per record. Section 14.1 sets it
/// per sink in the destination specification, and a stream that mixed modes
/// would be uninterpretable: a missing `data` would mean either suppression or
/// a defect, with no way to tell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayloadMode {
    /// `data` carries the payload as lowercase hex.
    WithPayload,
    /// `data` is omitted entirely. Section 13.5's metadata-only stream.
    ///
    /// Omitted rather than empty, because an empty string is what a
    /// zero-length payload renders as, and that is a real observation.
    MetadataOnly,
}

/// Writes packets and their attribution as newline-delimited JSON.
#[derive(Debug)]
pub struct JsonLinesWriter<W: Write> {
    out: W,
    interfaces: Vec<String>,
    mode: PayloadMode,
}

impl<W: Write> JsonLinesWriter<W> {
    /// Begin a stream, writing the header line immediately.
    ///
    /// The interface set is supplied here rather than declared incrementally,
    /// because the header lists it and the header is the first line. There is
    /// no point at which a later declaration could be accommodated.
    ///
    /// At most one interface, for the same reason the pcapng writer takes one.
    /// The slice's data model claimed this format escaped that restriction,
    /// because a JSON record names its interface explicitly where a pcapng
    /// packet block cannot. That was wrong, and review of pull request 9 caught
    /// it: naming the interface is not the problem, choosing it is.
    /// `CapturedPacket` carries no interface identifier and `Sink::write` has
    /// nowhere to pass one, so every packet routes through index 0. A stream
    /// declaring `["Ethernet", "NPF_Loopback"]` would label loopback traffic
    /// `Ethernet` on every record, which is a false statement about every
    /// packet in it rather than a missing field.
    pub fn new(mut out: W, interfaces: &[&str], mode: PayloadMode) -> Result<Self, WriteError> {
        let interfaces: Vec<String> = interfaces.iter().map(|s| (*s).to_owned()).collect();

        let mut line = String::from("{\"type\":\"header\",\"version\":");
        write_json_string(VERSION, &mut line);
        line.push_str(",\"interfaces\":[");
        // Declaration order, from an ordered collection. A set here would make
        // the header vary between runs and every golden unusable.
        for (i, name) in interfaces.iter().enumerate() {
            if i > 0 {
                line.push(',');
            }
            write_json_string(name, &mut line);
        }
        line.push_str("]}\n");
        out.write_all(line.as_bytes())?;

        Ok(JsonLinesWriter {
            out,
            interfaces,
            mode,
        })
    }

    /// Write one packet against a declared interface.
    fn write_packet(
        &mut self,
        interface_id: u32,
        packet: &CapturedPacket,
    ) -> Result<(), WriteError> {
        let idx = interface_id as usize;
        let Some(iface) = self.interfaces.get(idx) else {
            return Err(WriteError::UndeclaredInterface { id: interface_id });
        };

        // Rendered before a byte is written, so a refusal leaves no partial
        // line. A half-written line would make every following line
        // unparseable to a consumer reading sequentially, which is worse than
        // the refusal it came from.
        let ts = render_timestamp(packet.ts.as_nanos())?;

        // The shared derivation. Which keys are present is decided here and
        // nowhere else, so this format cannot disagree with the pcapng one
        // about the same packet.
        let a = Annotation::from_packet(packet, Some(iface));

        let mut line = String::with_capacity(160 + packet.data.as_ref().len() * 2);
        line.push_str("{\"ts\":");
        line.push_str(&ts);

        // Unlike the pcapng profile, which omits this when the capture holds
        // one interface. A JSON line is self-contained by design, and a
        // consumer that split the stream would otherwise lose the interface.
        line.push_str(",\"iface\":");
        write_json_string(iface, &mut line);

        if let Some(pid) = a.pid {
            line.push_str(",\"pid\":");
            line.push_str(&pid.to_string());
        }
        if let Some(proc_name) = &a.process {
            line.push_str(",\"proc\":");
            write_json_string(proc_name, &mut line);
        }
        if let Some(role) = &a.role {
            line.push_str(",\"role\":");
            write_json_string(role, &mut line);
        }
        if let Some(stage) = &a.stage {
            line.push_str(",\"stage\":");
            write_json_string(stage.as_str(), &mut line);
        }
        line.push_str(",\"dir\":");
        write_json_string(a.direction.as_str(), &mut line);
        line.push_str(",\"attr\":");
        write_json_string(fidelity_str(a.fidelity), &mut line);
        if let Some(flow_id) = a.flow_id {
            line.push_str(",\"flow_id\":");
            write_json_string(&flow_id.to_string(), &mut line);
        }

        if let Some(flow) = &packet.flow {
            line.push_str(",\"proto\":");
            write_json_string(proto_str(flow.proto), &mut line);

            // Wire order exists only in combination with direction, because
            // the flow key normalized endpoint position. With no direction it
            // is not merely unavailable, it is unknown to the whole pipeline,
            // and emitting `src` and `dst` anyway would present a coin flip as
            // an observation. The key names carry the distinction.
            match packet.direction {
                Some(Direction::Outbound) => {
                    line.push_str(",\"src\":");
                    write_json_string(&flow.local.to_string(), &mut line);
                    line.push_str(",\"dst\":");
                    write_json_string(&flow.remote.to_string(), &mut line);
                }
                Some(Direction::Inbound) => {
                    line.push_str(",\"src\":");
                    write_json_string(&flow.remote.to_string(), &mut line);
                    line.push_str(",\"dst\":");
                    write_json_string(&flow.local.to_string(), &mut line);
                }
                None => {
                    line.push_str(",\"local\":");
                    write_json_string(&flow.local.to_string(), &mut line);
                    line.push_str(",\"remote\":");
                    write_json_string(&flow.remote.to_string(), &mut line);
                }
            }
        }

        let data = packet.data.as_ref();
        // Both lengths exactly as recorded. A record that contradicts itself
        // is reported, not repaired, as in S04 and S06.
        line.push_str(",\"len\":");
        line.push_str(&data.len().to_string());
        line.push_str(",\"orig_len\":");
        line.push_str(&packet.orig_len.to_string());

        if self.mode == PayloadMode::WithPayload {
            line.push_str(",\"data\":");
            write_hex_string(data, &mut line);
        }

        line.push_str("}\n");
        self.out.write_all(line.as_bytes())?;
        Ok(())
    }

    /// Write the trailer and consume the writer.
    fn write_trailer(mut self, stats: &CaptureStats) -> Result<(), WriteError> {
        let source = stats.source();
        // Every counter comes from the supplied snapshot. Nothing here samples
        // or recomputes, which is what keeps the writer a pure function of its
        // input.
        //
        // Present even at zero, per P-4: omitting a zero would make "nothing
        // was lost" indistinguishable from "this build does not report that".
        let mut line = String::from("{\"type\":\"trailer\"");
        for (key, value) in [
            ("packets", stats.packets_captured),
            ("attributed", stats.packets_attributed),
            ("unattributed", stats.packets_unattributed),
            // The capture-wide sum. This trailer describes the run, and the
            // per-interface breakdown lives in the pcapng writer's Interface
            // Statistics Blocks where the format has somewhere to put it.
            ("kernel_dropped", source.kernel_dropped),
            ("interface_dropped", source.interface_dropped),
            ("buffer_dropped", stats.buffer_dropped),
            ("sink_dropped", stats.sink_dropped),
            ("filter_gaps", stats.filter_gaps),
        ] {
            line.push(',');
            write_json_string(key, &mut line);
            line.push(':');
            line.push_str(&value.to_string());
        }
        line.push_str("}\n");
        self.out.write_all(line.as_bytes())?;
        self.out.flush()?;
        Ok(())
    }
}

fn proto_str(p: fragcap_core::flow::Proto) -> &'static str {
    match p {
        fragcap_core::flow::Proto::Tcp => "tcp",
        fragcap_core::flow::Proto::Udp => "udp",
    }
}

impl<W: Write + Send> Sink for JsonLinesWriter<W> {
    /// Writes against interface 0, as the pcapng writer does and for the same
    /// reason: a `CapturedPacket` carries no interface identifier yet.
    fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError> {
        self.write_packet(packet.interface.index(), packet)
            .map_err(Into::into)
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        self.out
            .flush()
            .map_err(WriteError::from)
            .map_err(Into::into)
    }

    fn finish(self: Box<Self>, stats: &CaptureStats) -> Result<(), SinkError> {
        (*self).write_trailer(stats).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_core::attribution::{Attribution, Fidelity, StageId};
    use fragcap_core::flow::{FlowKey, Proto};
    use fragcap_core::interface::InterfaceId;
    use fragcap_core::packet::{Payload, RawPacket, Timestamp};
    use fragcap_core::stats::SourceStats;
    use serde_json::Value;

    fn packet(ts_nanos: i64, len: usize) -> CapturedPacket {
        let raw = RawPacket::new(
            Timestamp::from_nanos(ts_nanos),
            Payload::from(vec![0x3fu8; len]),
            len as u32,
        );
        CapturedPacket::from_raw(raw, InterfaceId::default())
    }

    fn flow() -> FlowKey {
        FlowKey::new(
            Proto::Udp,
            "192.0.2.10:51834".parse().unwrap(),
            "198.51.100.7:24100".parse().unwrap(),
        )
    }

    fn render(packets: &[CapturedPacket], mode: PayloadMode) -> String {
        let mut buf = Vec::new();
        {
            let mut w = JsonLinesWriter::new(&mut buf, &["eth0"], mode).unwrap();
            for p in packets {
                w.write(p).unwrap();
            }
            Box::new(w).finish(&CaptureStats::default()).unwrap();
        }
        String::from_utf8(buf).unwrap()
    }

    fn lines(s: &str) -> Vec<Value> {
        s.lines()
            .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("{l} must parse: {e}")))
            .collect()
    }

    /// Key order as it appears in the emitted text.
    ///
    /// Read from the raw line rather than from a parsed value on purpose:
    /// `serde_json::Map` is a `BTreeMap` by default and hands back keys
    /// alphabetically, which would make an order test pass no matter what this
    /// writer emitted. That is the same property research R-1 gives as a reason
    /// not to use the crate for output.
    fn key_order(line: &str) -> Vec<String> {
        let mut keys = Vec::new();
        let bytes = line.as_bytes();
        let (mut i, mut in_string, mut escaped, mut start) = (0usize, false, false, 0usize);
        let mut depth = 0i32;
        while i < bytes.len() {
            let c = bytes[i];
            if in_string {
                if escaped {
                    escaped = false;
                } else if c == b'\\' {
                    escaped = true;
                } else if c == b'"' {
                    in_string = false;
                    // A key is a string at object depth 1 followed by a colon.
                    if depth == 1 && bytes.get(i + 1) == Some(&b':') {
                        keys.push(line[start + 1..i].to_string());
                    }
                }
            } else {
                match c {
                    b'"' => {
                        in_string = true;
                        start = i;
                    }
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => depth -= 1,
                    _ => {}
                }
            }
            i += 1;
        }
        keys
    }

    // --- stream shape -------------------------------------------------------

    #[test]
    fn the_stream_is_one_object_per_line_with_no_array() {
        let s = render(
            &[packet(1_000_000, 4), packet(2_000_000, 4)],
            PayloadMode::WithPayload,
        );
        assert!(!s.starts_with('['), "no enclosing array");
        assert!(!s.contains("},{"), "no separating commas between records");
        assert_eq!(s.lines().count(), 4, "header, two packets, trailer");
        assert!(s.ends_with('\n'), "the last line is terminated too");
        assert!(!s.contains("\n\n"), "no blank lines");
        for v in lines(&s) {
            assert!(v.is_object());
        }
    }

    #[test]
    fn the_header_comes_first_and_declares_the_interface_set() {
        let s = render(&[], PayloadMode::WithPayload);
        let first = s.lines().next().unwrap();
        assert!(
            first.starts_with("{\"type\":\"header\""),
            "type must be first so a consumer dispatches on it: {first}"
        );
        let v: Value = serde_json::from_str(first).unwrap();
        assert_eq!(v["type"], "header");
        assert_eq!(v["version"], "fragcap/0.6.0");
        assert_eq!(v["interfaces"], serde_json::json!(["eth0"]));
    }

    #[test]
    fn the_interface_set_is_an_array_even_with_one_member() {
        // An array rather than a scalar, so the slice that lifts the
        // single-interface restriction adds members instead of changing a
        // consumer's parse.
        let mut buf = Vec::new();
        JsonLinesWriter::new(&mut buf, &["zeta"], PayloadMode::WithPayload).unwrap();
        let v: Value = serde_json::from_str(String::from_utf8(buf).unwrap().trim()).unwrap();
        assert_eq!(v["interfaces"], serde_json::json!(["zeta"]));
    }

    #[test]
    fn a_second_interface_is_accepted_and_each_packet_labelled_with_its_own() {
        // S06 and S07 refused this, and the reason was real at the time:
        // `CapturedPacket` carried no interface identifier, so every packet
        // would have routed to the first declaration and been labelled with it.
        // S09 supplied the identifier, so the refusal is replaced by working
        // support rather than by deleting a check.
        let mut buf = Vec::new();
        let mut w = JsonLinesWriter::new(
            &mut buf,
            &["Ethernet", "NPF_Loopback"],
            PayloadMode::MetadataOnly,
        )
        .expect("two interfaces are now legitimate");

        for id in [0u32, 1] {
            let mut packet = CapturedPacket::from_raw(
                RawPacket::new(Timestamp::from_nanos(0), Payload::new(), 0),
                InterfaceId::new(id),
            );
            packet.flow = None;
            w.write(&packet).expect("both interfaces are declared");
        }
        drop(w);

        let out = String::from_utf8(buf).expect("the writer emits UTF-8");
        let rows: Vec<&str> = out.lines().collect();
        assert!(
            rows[1].contains("\"iface\":\"Ethernet\""),
            "got {}",
            rows[1]
        );
        assert!(
            rows[2].contains("\"iface\":\"NPF_Loopback\""),
            "got {}",
            rows[2]
        );
    }

    #[test]
    fn a_stream_with_no_interfaces_is_still_a_valid_header() {
        let mut buf = Vec::new();
        JsonLinesWriter::new(&mut buf, &[], PayloadMode::WithPayload).unwrap();
        let v: Value = serde_json::from_str(String::from_utf8(buf).unwrap().trim()).unwrap();
        assert_eq!(v["interfaces"], serde_json::json!([]));
    }

    #[test]
    fn no_packet_record_carries_a_type_key() {
        // The consumer dispatch contract. A packet carrying `type` would be
        // read as metadata by anything dispatching on the first key, and
        // nothing else in the stream would look wrong.
        let s = render(&[packet(1_000_000, 4)], PayloadMode::WithPayload);
        let records = lines(&s);
        assert!(records[1].get("type").is_none(), "got {}", records[1]);
        assert_eq!(
            key_order(s.lines().nth(1).unwrap())[0],
            "ts",
            "ts is first on a packet record"
        );
    }

    // --- key order and presence ---------------------------------------------

    #[test]
    fn keys_appear_in_specification_order() {
        let mut p = packet(1_754_500_000_123_456_000, 3);
        p.flow = Some(flow());
        p.direction = Some(Direction::Outbound);
        p.attribution = Some(
            Attribution::new(7412, "eso64.exe", Fidelity::Live)
                .with_role("client")
                .with_stage(StageId::new("play")),
        );
        let s = render(&[p], PayloadMode::WithPayload);
        assert_eq!(
            key_order(s.lines().nth(1).unwrap()),
            [
                "ts", "iface", "pid", "proc", "role", "stage", "dir", "attr", "proto", "src",
                "dst", "len", "orig_len", "data"
            ]
        );
    }

    #[test]
    fn absent_keys_do_not_disturb_the_order_of_present_ones() {
        let s = render(&[packet(1_000_000, 2)], PayloadMode::WithPayload);
        assert_eq!(
            key_order(s.lines().nth(1).unwrap()),
            ["ts", "iface", "dir", "attr", "len", "orig_len", "data"]
        );
    }

    #[test]
    fn identity_keys_appear_as_a_pair_and_stage_independently_of_role() {
        let role_only = Attribution::new(1, "a.exe", Fidelity::Live).with_role("client");
        let mut p = packet(1_000_000, 2);
        p.attribution = Some(role_only);
        let v = &lines(&render(&[p], PayloadMode::WithPayload))[1];
        assert_eq!(v["pid"], 1);
        assert_eq!(v["proc"], "a.exe");
        assert_eq!(v["role"], "client");
        assert!(v.get("stage").is_none(), "a role must not imply a stage");

        let stage_only =
            Attribution::new(1, "a.exe", Fidelity::Live).with_stage(StageId::new("play"));
        let mut p = packet(1_000_000, 2);
        p.attribution = Some(stage_only);
        let v = &lines(&render(&[p], PayloadMode::WithPayload))[1];
        assert!(v.get("role").is_none(), "a stage must not imply a role");
        assert_eq!(v["stage"], "play");
    }

    #[test]
    fn dir_and_attr_are_present_in_every_state() {
        for p in [packet(1_000_000, 2), {
            let mut p = packet(2_000_000, 2);
            p.attribution = Some(Attribution::new(1, "a.exe", Fidelity::Live));
            p
        }] {
            let v = &lines(&render(&[p], PayloadMode::WithPayload))[1];
            assert!(v.get("dir").is_some());
            assert!(v.get("attr").is_some());
        }
    }

    #[test]
    fn an_unattributed_packet_is_written_with_no_identity_keys() {
        let v = &lines(&render(&[packet(1_000_000, 2)], PayloadMode::WithPayload))[1];
        assert_eq!(v["attr"], "none");
        for k in ["pid", "proc", "role", "stage"] {
            assert!(v.get(k).is_none(), "{k} present on an unattributed packet");
        }
    }

    #[test]
    fn iface_is_present_on_every_record_unlike_the_pcapng_profile() {
        let v = &lines(&render(&[packet(1_000_000, 2)], PayloadMode::WithPayload))[1];
        assert_eq!(v["iface"], "eth0");
    }

    // --- endpoints ----------------------------------------------------------

    #[test]
    fn a_known_direction_yields_wire_order() {
        let mut out = packet(1_000_000, 2);
        out.flow = Some(flow());
        out.direction = Some(Direction::Outbound);
        let v = &lines(&render(&[out], PayloadMode::WithPayload))[1];
        assert_eq!(v["src"], "192.0.2.10:51834");
        assert_eq!(v["dst"], "198.51.100.7:24100");

        let mut inb = packet(1_000_000, 2);
        inb.flow = Some(flow());
        inb.direction = Some(Direction::Inbound);
        let v = &lines(&render(&[inb], PayloadMode::WithPayload))[1];
        assert_eq!(
            v["src"], "198.51.100.7:24100",
            "inbound reverses wire order"
        );
        assert_eq!(v["dst"], "192.0.2.10:51834");
    }

    #[test]
    fn an_unknown_direction_yields_position_not_wire_order() {
        let mut p = packet(1_000_000, 2);
        p.flow = Some(flow());
        p.direction = None;
        let v = &lines(&render(&[p], PayloadMode::WithPayload))[1];
        assert_eq!(v["dir"], "unknown");
        assert_eq!(v["local"], "192.0.2.10:51834");
        assert_eq!(v["remote"], "198.51.100.7:24100");
        assert!(v.get("src").is_none(), "wire order must not be guessed");
        assert!(v.get("dst").is_none());
    }

    #[test]
    fn no_record_carries_both_endpoint_pairs() {
        for dir in [Some(Direction::Outbound), Some(Direction::Inbound), None] {
            let mut p = packet(1_000_000, 2);
            p.flow = Some(flow());
            p.direction = dir;
            let v = &lines(&render(&[p], PayloadMode::WithPayload))[1];
            let wire = v.get("src").is_some();
            let position = v.get("local").is_some();
            assert!(wire ^ position, "exactly one endpoint pair, got {v}");
        }
    }

    #[test]
    fn no_flow_key_means_no_protocol_and_no_endpoints() {
        let v = &lines(&render(&[packet(1_000_000, 2)], PayloadMode::WithPayload))[1];
        for k in ["proto", "src", "dst", "local", "remote"] {
            assert!(v.get(k).is_none(), "{k} present without a flow key");
        }
    }

    // --- lengths and payload ------------------------------------------------

    #[test]
    fn lengths_are_written_exactly_as_recorded() {
        let raw = RawPacket::new(Timestamp::from_nanos(0), Payload::from(vec![0u8; 40]), 8);
        let v = &lines(&render(
            &[CapturedPacket::from_raw(raw, InterfaceId::default())],
            PayloadMode::WithPayload,
        ))[1];
        assert_eq!(v["len"], 40);
        assert_eq!(
            v["orig_len"], 8,
            "a contradiction is reported, not repaired"
        );
    }

    #[test]
    fn metadata_only_omits_exactly_one_key() {
        let mut p = packet(1_000_000, 4);
        p.flow = Some(flow());
        p.direction = Some(Direction::Outbound);
        let with = lines(&render(&[p.clone()], PayloadMode::WithPayload))[1].clone();
        let without = lines(&render(&[p], PayloadMode::MetadataOnly))[1].clone();

        assert!(with.get("data").is_some());
        assert!(without.get("data").is_none(), "omitted, not emptied");

        let mut trimmed = with.as_object().unwrap().clone();
        trimmed.remove("data");
        assert_eq!(
            &trimmed,
            without.as_object().unwrap(),
            "payload mode changes exactly one key"
        );
    }

    #[test]
    fn a_zero_length_payload_is_an_empty_string_not_an_omission() {
        let v = &lines(&render(&[packet(1_000_000, 0)], PayloadMode::WithPayload))[1];
        assert_eq!(v["data"], "");
        assert_eq!(
            v["len"], 0,
            "len disambiguates the two reasons data can be absent"
        );
    }

    // --- trailer ------------------------------------------------------------

    fn stats_with_everything() -> CaptureStats {
        CaptureStats {
            packets_captured: 1_000,
            packets_attributed: 900,
            packets_unattributed: 100,
            buffer_dropped: 5,
            sink_dropped: 3,
            filter_gaps: 2,
            sources: vec![(
                InterfaceId::default(),
                SourceStats {
                    received: 1_010,
                    kernel_dropped: 7,
                    interface_dropped: 3,
                },
            )],
            ..Default::default()
        }
    }

    #[test]
    fn the_trailer_carries_every_counter() {
        let mut buf = Vec::new();
        {
            let w = JsonLinesWriter::new(&mut buf, &["eth0"], PayloadMode::WithPayload).unwrap();
            Box::new(w).finish(&stats_with_everything()).unwrap();
        }
        let s = String::from_utf8(buf).unwrap();
        let last = s.lines().last().unwrap();
        assert!(
            last.starts_with("{\"type\":\"trailer\""),
            "type first: {last}"
        );
        let v: Value = serde_json::from_str(last).unwrap();
        assert_eq!(v["packets"], 1_000);
        assert_eq!(v["attributed"], 900);
        assert_eq!(v["unattributed"], 100);
        assert_eq!(v["kernel_dropped"], 7);
        assert_eq!(v["interface_dropped"], 3);
        assert_eq!(v["buffer_dropped"], 5);
        assert_eq!(v["sink_dropped"], 3);
        assert_eq!(v["filter_gaps"], 2);
    }

    #[test]
    fn counters_are_present_even_at_zero() {
        let s = render(&[], PayloadMode::WithPayload);
        let v: Value = serde_json::from_str(s.lines().last().unwrap()).unwrap();
        for k in [
            "packets",
            "attributed",
            "unattributed",
            "kernel_dropped",
            "interface_dropped",
            "buffer_dropped",
            "sink_dropped",
            "filter_gaps",
        ] {
            assert_eq!(v[k], 0, "{k} must be present and zero, not absent");
        }
    }

    #[test]
    fn a_writer_dropped_without_finishing_leaves_no_trailer() {
        let mut buf = Vec::new();
        {
            let mut w =
                JsonLinesWriter::new(&mut buf, &["eth0"], PayloadMode::WithPayload).unwrap();
            w.write(&packet(1_000_000, 4)).unwrap();
        }
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s.lines().count(), 2, "header and the packet");
        for v in lines(&s) {
            assert!(v.is_object(), "every written line is still complete");
        }
        assert!(
            !s.contains("trailer"),
            "the absent trailer is how a consumer detects truncation"
        );
    }

    // --- refusals -----------------------------------------------------------

    #[test]
    fn a_refused_record_writes_no_partial_line() {
        let mut buf = Vec::new();
        let mut w = JsonLinesWriter::new(&mut buf, &["eth0"], PayloadMode::WithPayload).unwrap();
        let before = w.out.len();
        let err = w.write_packet(0, &packet(-1_000, 4)).unwrap_err();
        assert_eq!(err, WriteError::TimestampBeforeEpoch { nanos: -1_000 });
        assert_eq!(
            w.out.len(),
            before,
            "a half-written line would break every following line"
        );
    }

    #[test]
    fn an_undeclared_interface_is_refused() {
        let mut buf = Vec::new();
        let mut w = JsonLinesWriter::new(&mut buf, &["eth0"], PayloadMode::WithPayload).unwrap();
        assert_eq!(
            w.write_packet(3, &packet(1_000, 4)),
            Err(WriteError::UndeclaredInterface { id: 3 })
        );
    }

    // --- determinism --------------------------------------------------------

    #[test]
    fn the_same_input_produces_the_same_bytes() {
        let mut p = packet(1_754_500_000_123_456_000, 8);
        p.flow = Some(flow());
        p.direction = Some(Direction::Outbound);
        p.attribution = Some(Attribution::new(7412, "eso64.exe", Fidelity::Live));
        let once = render(std::slice::from_ref(&p), PayloadMode::WithPayload);
        let twice = render(std::slice::from_ref(&p), PayloadMode::WithPayload);
        assert_eq!(once, twice);
    }

    #[test]
    fn the_writer_reads_no_environment() {
        let p = packet(1_000_000, 4);
        let before = render(std::slice::from_ref(&p), PayloadMode::WithPayload);
        std::env::set_var("FRAGCAP_PROBE", "changed");
        std::env::set_var("TZ", "Pacific/Kiritimati");
        let after = render(std::slice::from_ref(&p), PayloadMode::WithPayload);
        std::env::remove_var("FRAGCAP_PROBE");
        std::env::remove_var("TZ");
        assert_eq!(before, after, "output must not depend on ambient state");
    }
}
