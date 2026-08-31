// SPDX-License-Identifier: Apache-2.0

//! Bounded, incremental observers for streaming application protocols.

use flate2::{Decompress, FlushDecompress, Status};

use crate::BodyDirection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamingOutcome {
    Complete,
    IntentionallyOmitted,
    Partial,
    Malformed,
    Limit,
    UnsupportedCompression,
    InvalidUtf8,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebSocketMessageKind {
    Text,
    Binary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebSocketFrame {
    pub direction: BodyDirection,
    pub sequence: u64,
    pub fin: bool,
    pub rsv1: bool,
    pub opcode: u8,
    pub masked: bool,
    pub masking_key: Option<[u8; 4]>,
    pub declared_len: u64,
    pub wire_payload: Vec<u8>,
    pub close_code: Option<u16>,
    pub close_reason: Vec<u8>,
    pub close_reason_utf8: Option<bool>,
    pub outcome: StreamingOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebSocketMessage {
    pub direction: BodyDirection,
    pub sequence: u64,
    pub first_frame: u64,
    pub frame_count: u64,
    pub kind: WebSocketMessageKind,
    pub compressed: bool,
    pub observed_len: u64,
    pub payload: Vec<u8>,
    pub outcome: StreamingOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SseField {
    pub sequence: u64,
    pub line: u64,
    pub name: Vec<u8>,
    pub value: Vec<u8>,
    pub comment: bool,
    pub observed_len: u64,
    pub outcome: StreamingOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SseEvent {
    pub sequence: u64,
    pub first_line: u64,
    pub last_line: u64,
    pub event_type: Vec<u8>,
    pub data: Vec<u8>,
    pub last_event_id: Vec<u8>,
    pub retry_ms: Option<u64>,
    pub observed_len: u64,
    pub outcome: StreamingOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrpcMessage {
    pub direction: BodyDirection,
    pub sequence: u64,
    pub compressed: bool,
    pub declared_len: u32,
    pub payload: Vec<u8>,
    pub encoding: Option<Vec<u8>>,
    pub outcome: StreamingOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamingEvent {
    WebSocketFrame(WebSocketFrame),
    WebSocketMessage(WebSocketMessage),
    WebSocketTerminal {
        direction: BodyDirection,
        outcome: StreamingOutcome,
    },
    SseField(SseField),
    SseEvent(SseEvent),
    SseTerminal {
        outcome: StreamingOutcome,
    },
    GrpcCall {
        method: Vec<u8>,
        content_type: Vec<u8>,
        encoding: Option<Vec<u8>>,
    },
    GrpcMessage(GrpcMessage),
    GrpcTerminal {
        direction: BodyDirection,
        status: Option<Vec<u8>>,
        message: Option<Vec<u8>>,
        status_details: Option<Vec<u8>>,
        outcome: StreamingOutcome,
    },
}

struct FragmentedMessage {
    kind: WebSocketMessageKind,
    compressed: bool,
    first_frame: u64,
    frame_count: u64,
    bytes: Vec<u8>,
    observed: u64,
    limited: bool,
}

pub struct WebSocketObserver {
    direction: BodyDirection,
    expect_masked: bool,
    compression: bool,
    max_frame: usize,
    max_message: usize,
    buffer: Vec<u8>,
    skip: u64,
    frame_sequence: u64,
    message_sequence: u64,
    fragmented: Option<FragmentedMessage>,
    decompressor: Option<Decompress>,
    no_context_takeover: bool,
    retired: bool,
}

impl WebSocketObserver {
    pub fn new(
        direction: BodyDirection,
        expect_masked: bool,
        compression: bool,
        max_frame: usize,
        max_message: usize,
    ) -> Self {
        Self {
            direction,
            expect_masked,
            compression,
            max_frame,
            max_message,
            buffer: Vec::new(),
            skip: 0,
            frame_sequence: 0,
            message_sequence: 0,
            fragmented: None,
            decompressor: compression.then(|| Decompress::new(false)),
            no_context_takeover: false,
            retired: false,
        }
    }

    pub fn with_no_context_takeover(mut self, value: bool) -> Self {
        self.no_context_takeover = value;
        self
    }

    pub fn feed(&mut self, mut input: &[u8]) -> Vec<StreamingEvent> {
        let mut events = Vec::new();
        if self.retired {
            return events;
        }
        let input_bound = self.max_frame.saturating_add(14).max(14);
        if input.len() > input_bound {
            for chunk in input.chunks(input_bound) {
                events.extend(self.feed(chunk));
            }
            return events;
        }
        if self.skip > 0 {
            let skipped = input.len().min(self.skip as usize);
            self.skip -= skipped as u64;
            input = &input[skipped..];
        }
        self.buffer.extend_from_slice(input);
        loop {
            if self.buffer.len() < 2 {
                break;
            }
            let first = self.buffer[0];
            let second = self.buffer[1];
            let fin = first & 0x80 != 0;
            let rsv1 = first & 0x40 != 0;
            let opcode = first & 0x0f;
            let masked = second & 0x80 != 0;
            let marker = second & 0x7f;
            let length_bytes = match marker {
                126 => 2,
                127 => 8,
                _ => 0,
            };
            let mask_bytes = if masked { 4 } else { 0 };
            let header_len = 2 + length_bytes + mask_bytes;
            if self.buffer.len() < header_len {
                break;
            }
            let declared = match marker {
                126 => u16::from_be_bytes([self.buffer[2], self.buffer[3]]) as u64,
                127 => u64::from_be_bytes(self.buffer[2..10].try_into().expect("eight bytes")),
                value => value as u64,
            };
            let key_start = 2 + length_bytes;
            let key = masked.then(|| {
                self.buffer[key_start..key_start + 4]
                    .try_into()
                    .expect("four bytes")
            });
            self.frame_sequence = self.frame_sequence.saturating_add(1);
            let sequence = self.frame_sequence;
            let invalid_header = (marker == 126 && declared < 126)
                || (marker == 127 && (declared < 65_536 || declared & (1 << 63) != 0))
                || masked != self.expect_masked
                || first & 0x30 != 0
                || rsv1 && (!self.compression || opcode == 0 || opcode >= 8)
                || !matches!(opcode, 0 | 1 | 2 | 8 | 9 | 10)
                || (opcode >= 8 && (!fin || declared > 125));
            if invalid_header || declared > self.max_frame as u64 {
                let outcome = if declared > self.max_frame as u64 {
                    StreamingOutcome::Limit
                } else {
                    StreamingOutcome::Malformed
                };
                events.push(StreamingEvent::WebSocketFrame(WebSocketFrame {
                    direction: self.direction,
                    sequence,
                    fin,
                    rsv1,
                    opcode,
                    masked,
                    masking_key: key,
                    declared_len: declared,
                    wire_payload: Vec::new(),
                    close_code: None,
                    close_reason: Vec::new(),
                    close_reason_utf8: None,
                    outcome,
                }));
                self.buffer.drain(..header_len);
                let available = self.buffer.len().min(declared as usize);
                self.buffer.drain(..available);
                self.skip = declared.saturating_sub(available as u64);
                if invalid_header {
                    self.retired = true;
                    events.push(StreamingEvent::WebSocketTerminal {
                        direction: self.direction,
                        outcome,
                    });
                }
                break;
            }
            let total = header_len + declared as usize;
            if self.buffer.len() < total {
                self.frame_sequence = self.frame_sequence.saturating_sub(1);
                break;
            }
            let wire_payload = self.buffer[header_len..total].to_vec();
            self.buffer.drain(..total);
            let mut payload = wire_payload.clone();
            if let Some(mask) = key {
                for (index, byte) in payload.iter_mut().enumerate() {
                    *byte ^= mask[index % 4];
                }
            }
            let (close_code, close_reason, close_reason_utf8, frame_outcome) =
                websocket_close(&payload, opcode);
            events.push(StreamingEvent::WebSocketFrame(WebSocketFrame {
                direction: self.direction,
                sequence,
                fin,
                rsv1,
                opcode,
                masked,
                masking_key: key,
                declared_len: declared,
                wire_payload,
                close_code,
                close_reason,
                close_reason_utf8,
                outcome: frame_outcome,
            }));
            if opcode < 8 {
                self.observe_data_frame(opcode, fin, rsv1, payload, sequence, &mut events);
            }
        }
        events
    }

    fn observe_data_frame(
        &mut self,
        opcode: u8,
        fin: bool,
        compressed: bool,
        payload: Vec<u8>,
        frame: u64,
        events: &mut Vec<StreamingEvent>,
    ) {
        if opcode == 0 {
            let Some(message) = self.fragmented.as_mut() else {
                self.retired = true;
                events.push(StreamingEvent::WebSocketTerminal {
                    direction: self.direction,
                    outcome: StreamingOutcome::Malformed,
                });
                return;
            };
            message.frame_count = message.frame_count.saturating_add(1);
            message.observed = message.observed.saturating_add(payload.len() as u64);
            append_bounded(
                &mut message.bytes,
                &payload,
                self.max_message,
                &mut message.limited,
            );
            if fin {
                let message = self.fragmented.take().expect("fragmented message exists");
                self.finish_message(message, events);
            }
            return;
        }
        if self.fragmented.is_some() {
            self.retired = true;
            events.push(StreamingEvent::WebSocketTerminal {
                direction: self.direction,
                outcome: StreamingOutcome::Malformed,
            });
            return;
        }
        let kind = if opcode == 1 {
            WebSocketMessageKind::Text
        } else {
            WebSocketMessageKind::Binary
        };
        let mut message = FragmentedMessage {
            kind,
            compressed,
            first_frame: frame,
            frame_count: 1,
            bytes: Vec::new(),
            observed: payload.len() as u64,
            limited: false,
        };
        append_bounded(
            &mut message.bytes,
            &payload,
            self.max_message,
            &mut message.limited,
        );
        if fin {
            self.finish_message(message, events);
        } else {
            self.fragmented = Some(message);
        }
    }

    fn finish_message(&mut self, mut message: FragmentedMessage, events: &mut Vec<StreamingEvent>) {
        let mut outcome = if message.limited {
            StreamingOutcome::Limit
        } else {
            StreamingOutcome::Complete
        };
        if message.compressed && !message.limited {
            let decompressor = self
                .decompressor
                .as_mut()
                .expect("compressed observer has a decompressor");
            match inflate_message(decompressor, &message.bytes, self.max_message) {
                Ok(bytes) => message.bytes = bytes,
                Err(value) => {
                    message.bytes.clear();
                    outcome = value;
                }
            }
            if self.no_context_takeover {
                self.decompressor = Some(Decompress::new(false));
            }
        }
        if message.kind == WebSocketMessageKind::Text
            && outcome == StreamingOutcome::Complete
            && std::str::from_utf8(&message.bytes).is_err()
        {
            outcome = StreamingOutcome::InvalidUtf8;
        }
        self.message_sequence = self.message_sequence.saturating_add(1);
        events.push(StreamingEvent::WebSocketMessage(WebSocketMessage {
            direction: self.direction,
            sequence: self.message_sequence,
            first_frame: message.first_frame,
            frame_count: message.frame_count,
            kind: message.kind,
            compressed: message.compressed,
            observed_len: message.observed,
            payload: message.bytes,
            outcome,
        }));
    }

    pub fn finish(&mut self, outcome: StreamingOutcome) -> StreamingEvent {
        let terminal = if self.fragmented.is_some() || !self.buffer.is_empty() || self.skip > 0 {
            StreamingOutcome::Partial
        } else {
            outcome
        };
        self.retired = true;
        StreamingEvent::WebSocketTerminal {
            direction: self.direction,
            outcome: terminal,
        }
    }
}

fn append_bounded(target: &mut Vec<u8>, bytes: &[u8], limit: usize, limited: &mut bool) {
    let keep = bytes.len().min(limit.saturating_sub(target.len()));
    target.extend_from_slice(&bytes[..keep]);
    *limited |= keep != bytes.len();
}

fn inflate_message(
    decompressor: &mut Decompress,
    input: &[u8],
    limit: usize,
) -> Result<Vec<u8>, StreamingOutcome> {
    let mut source = Vec::with_capacity(input.len() + 4);
    source.extend_from_slice(input);
    source.extend_from_slice(&[0, 0, 0xff, 0xff]);
    let mut output = vec![0; limit.saturating_add(1)];
    match decompressor.decompress(&source, &mut output, FlushDecompress::Sync) {
        Ok(Status::Ok | Status::StreamEnd | Status::BufError) => {
            let written = decompressor.total_out() as usize;
            if written > limit {
                Err(StreamingOutcome::Limit)
            } else {
                output.truncate(written);
                Ok(output)
            }
        }
        Err(_) => Err(StreamingOutcome::Malformed),
    }
}

pub struct SseObserver {
    max_line: usize,
    max_event: usize,
    line: Vec<u8>,
    line_number: u64,
    field_sequence: u64,
    event_sequence: u64,
    first_event_line: u64,
    data: Vec<u8>,
    event_type: Vec<u8>,
    last_event_id: Vec<u8>,
    retry_ms: Option<u64>,
    pending_cr: bool,
    preamble: Vec<u8>,
    started: bool,
    skipping_line: bool,
    event_limited: bool,
}

impl SseObserver {
    pub fn new(max_line: usize, max_event: usize) -> Self {
        Self {
            max_line,
            max_event,
            line: Vec::new(),
            line_number: 0,
            field_sequence: 0,
            event_sequence: 0,
            first_event_line: 1,
            data: Vec::new(),
            event_type: Vec::new(),
            last_event_id: Vec::new(),
            retry_ms: None,
            pending_cr: false,
            preamble: Vec::new(),
            started: false,
            skipping_line: false,
            event_limited: false,
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) -> Vec<StreamingEvent> {
        if !self.started {
            let bom = [0xef, 0xbb, 0xbf];
            let mut consumed = 0;
            while consumed < bytes.len() && !self.started {
                self.preamble.push(bytes[consumed]);
                consumed += 1;
                if self.preamble == bom {
                    self.preamble.clear();
                    self.started = true;
                } else if !bom.starts_with(&self.preamble) {
                    self.started = true;
                }
            }
            if !self.started {
                return Vec::new();
            }
            let prefix = std::mem::take(&mut self.preamble);
            let mut events = self.feed_started(&prefix);
            events.extend(self.feed_started(&bytes[consumed..]));
            return events;
        }
        self.feed_started(bytes)
    }

    fn feed_started(&mut self, bytes: &[u8]) -> Vec<StreamingEvent> {
        let mut events = Vec::new();
        for &byte in bytes {
            if self.pending_cr {
                self.pending_cr = false;
                if byte == b'\n' {
                    continue;
                }
            }
            if byte == b'\r' || byte == b'\n' {
                self.process_line(&mut events);
                self.pending_cr = byte == b'\r';
            } else if !self.skipping_line {
                if self.line.len() == self.max_line {
                    self.skipping_line = true;
                    self.line.clear();
                } else {
                    self.line.push(byte);
                }
            }
        }
        events
    }

    fn process_line(&mut self, events: &mut Vec<StreamingEvent>) {
        self.line_number = self.line_number.saturating_add(1);
        if self.skipping_line {
            self.field_sequence = self.field_sequence.saturating_add(1);
            events.push(StreamingEvent::SseField(SseField {
                sequence: self.field_sequence,
                line: self.line_number,
                name: Vec::new(),
                value: Vec::new(),
                comment: false,
                observed_len: 0,
                outcome: StreamingOutcome::Limit,
            }));
            self.event_limited = true;
            self.skipping_line = false;
            return;
        }
        let line = std::mem::take(&mut self.line);
        if line.is_empty() {
            self.dispatch(events, StreamingOutcome::Complete);
            self.first_event_line = self.line_number.saturating_add(1);
            return;
        }
        let comment = line.first() == Some(&b':');
        let (name, mut value) = if comment {
            (Vec::new(), line[1..].to_vec())
        } else if let Some(split) = line.iter().position(|byte| *byte == b':') {
            (line[..split].to_vec(), line[split + 1..].to_vec())
        } else {
            (line, Vec::new())
        };
        if value.first() == Some(&b' ') {
            value.remove(0);
        }
        let valid = std::str::from_utf8(&name).is_ok() && std::str::from_utf8(&value).is_ok();
        self.field_sequence = self.field_sequence.saturating_add(1);
        events.push(StreamingEvent::SseField(SseField {
            sequence: self.field_sequence,
            line: self.line_number,
            name: name.clone(),
            value: value.clone(),
            comment,
            observed_len: value.len() as u64,
            outcome: if valid {
                StreamingOutcome::Complete
            } else {
                StreamingOutcome::InvalidUtf8
            },
        }));
        if comment || !valid {
            return;
        }
        match name.as_slice() {
            b"data" => {
                append_bounded(
                    &mut self.data,
                    &value,
                    self.max_event,
                    &mut self.event_limited,
                );
                append_bounded(
                    &mut self.data,
                    b"\n",
                    self.max_event,
                    &mut self.event_limited,
                );
            }
            b"event" => self.event_type = value,
            b"id" if !value.contains(&0) => self.last_event_id = value,
            b"retry" if value.iter().all(u8::is_ascii_digit) => {
                self.retry_ms = std::str::from_utf8(&value)
                    .ok()
                    .and_then(|text| text.parse().ok());
            }
            _ => {}
        }
    }

    fn dispatch(&mut self, events: &mut Vec<StreamingEvent>, terminal: StreamingOutcome) {
        if self.data.is_empty() && !self.event_limited {
            self.event_type.clear();
            return;
        }
        if self.data.last() == Some(&b'\n') {
            self.data.pop();
        }
        self.event_sequence = self.event_sequence.saturating_add(1);
        let observed_len = self.data.len() as u64;
        events.push(StreamingEvent::SseEvent(SseEvent {
            sequence: self.event_sequence,
            first_line: self.first_event_line,
            last_line: self.line_number,
            event_type: std::mem::take(&mut self.event_type),
            data: std::mem::take(&mut self.data),
            last_event_id: self.last_event_id.clone(),
            retry_ms: self.retry_ms,
            observed_len,
            outcome: if std::mem::take(&mut self.event_limited) {
                StreamingOutcome::Limit
            } else {
                terminal
            },
        }));
    }

    pub fn finish(&mut self, outcome: StreamingOutcome) -> Vec<StreamingEvent> {
        let mut events = Vec::new();
        if !self.line.is_empty() || self.skipping_line {
            self.process_line(&mut events);
        }
        self.dispatch(&mut events, StreamingOutcome::Partial);
        events.push(StreamingEvent::SseTerminal { outcome });
        events
    }
}

pub struct GrpcObserver {
    direction: BodyDirection,
    max_message: usize,
    buffer: Vec<u8>,
    skip: u64,
    sequence: u64,
    encoding: Option<Vec<u8>>,
    initial_status: Option<Vec<u8>>,
    initial_message: Option<Vec<u8>>,
    initial_status_details: Option<Vec<u8>>,
}

impl GrpcObserver {
    pub fn new(direction: BodyDirection, max_message: usize) -> Self {
        Self {
            direction,
            max_message,
            buffer: Vec::new(),
            skip: 0,
            sequence: 0,
            encoding: None,
            initial_status: None,
            initial_message: None,
            initial_status_details: None,
        }
    }

    pub fn with_encoding(mut self, encoding: Option<Vec<u8>>) -> Self {
        self.encoding = encoding;
        self
    }

    pub fn with_terminal_metadata(
        mut self,
        status: Option<Vec<u8>>,
        message: Option<Vec<u8>>,
        status_details: Option<Vec<u8>>,
    ) -> Self {
        self.initial_status = status;
        self.initial_message = message;
        self.initial_status_details = status_details;
        self
    }

    pub fn feed(&mut self, mut bytes: &[u8]) -> Vec<StreamingEvent> {
        let mut events = Vec::new();
        let input_bound = self.max_message.saturating_add(5).max(5);
        if bytes.len() > input_bound {
            for chunk in bytes.chunks(input_bound) {
                events.extend(self.feed(chunk));
            }
            return events;
        }
        if self.skip > 0 {
            let skipped = bytes.len().min(self.skip as usize);
            self.skip -= skipped as u64;
            bytes = &bytes[skipped..];
        }
        self.buffer.extend_from_slice(bytes);
        loop {
            if self.buffer.len() < 5 {
                break;
            }
            let flag = self.buffer[0];
            let length = u32::from_be_bytes(self.buffer[1..5].try_into().expect("four bytes"));
            self.sequence = self.sequence.saturating_add(1);
            if flag > 1 || length as usize > self.max_message {
                let outcome = if flag > 1 {
                    StreamingOutcome::Malformed
                } else {
                    StreamingOutcome::Limit
                };
                events.push(StreamingEvent::GrpcMessage(GrpcMessage {
                    direction: self.direction,
                    sequence: self.sequence,
                    compressed: flag == 1,
                    declared_len: length,
                    payload: Vec::new(),
                    encoding: self.encoding.clone(),
                    outcome,
                }));
                self.buffer.drain(..5);
                let available = self.buffer.len().min(length as usize);
                self.buffer.drain(..available);
                self.skip = u64::from(length).saturating_sub(available as u64);
                break;
            }
            let total = 5 + length as usize;
            if self.buffer.len() < total {
                self.sequence = self.sequence.saturating_sub(1);
                break;
            }
            let payload = self.buffer[5..total].to_vec();
            self.buffer.drain(..total);
            events.push(StreamingEvent::GrpcMessage(GrpcMessage {
                direction: self.direction,
                sequence: self.sequence,
                compressed: flag == 1,
                declared_len: length,
                payload,
                encoding: self.encoding.clone(),
                outcome: if flag == 1 && self.encoding.is_none() {
                    StreamingOutcome::Malformed
                } else {
                    StreamingOutcome::Complete
                },
            }));
        }
        events
    }

    pub fn finish(
        &self,
        status: Option<Vec<u8>>,
        message: Option<Vec<u8>>,
        status_details: Option<Vec<u8>>,
        outcome: StreamingOutcome,
    ) -> StreamingEvent {
        let status = status.or_else(|| self.initial_status.clone());
        let message = message.or_else(|| self.initial_message.clone());
        let status_details = status_details.or_else(|| self.initial_status_details.clone());
        let missing_status = status.is_none();
        StreamingEvent::GrpcTerminal {
            direction: self.direction,
            status,
            message,
            status_details,
            outcome: if self.direction == BodyDirection::Response && missing_status {
                StreamingOutcome::Partial
            } else if self.buffer.is_empty() && self.skip == 0 {
                outcome
            } else {
                StreamingOutcome::Partial
            },
        }
    }
}

fn websocket_close(
    payload: &[u8],
    opcode: u8,
) -> (Option<u16>, Vec<u8>, Option<bool>, StreamingOutcome) {
    if opcode != 8 {
        return (None, Vec::new(), None, StreamingOutcome::Complete);
    }
    if payload.len() == 1 {
        return (None, Vec::new(), None, StreamingOutcome::Malformed);
    }
    if payload.is_empty() {
        return (None, Vec::new(), Some(true), StreamingOutcome::Complete);
    }
    let code = u16::from_be_bytes([payload[0], payload[1]]);
    let reason = payload[2..].to_vec();
    let reason_valid = std::str::from_utf8(&reason).is_ok();
    let code_valid = matches!(code, 1000..=1014 | 3000..=4999) && !matches!(code, 1004..=1006);
    (
        Some(code),
        reason,
        Some(reason_valid),
        if code_valid && reason_valid {
            StreamingOutcome::Complete
        } else {
            StreamingOutcome::Malformed
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compress, Compression, FlushCompress};

    #[test]
    fn websocket_reassembles_masked_fragmented_text_across_chunks() {
        let mut observer = WebSocketObserver::new(BodyDirection::Request, true, false, 1024, 1024);
        let first = [0x01, 0x82, 1, 2, 3, 4, b'h' ^ 1, b'e' ^ 2];
        let second = [0x80, 0x83, 5, 6, 7, 8, b'l' ^ 5, b'l' ^ 6, b'o' ^ 7];
        let mut events = observer.feed(&first[..3]);
        events.extend(observer.feed(&first[3..]));
        events.extend(observer.feed(&second));
        assert!(events.iter().any(|event| matches!(event, StreamingEvent::WebSocketMessage(message) if message.payload == b"hello")));
    }

    #[test]
    fn websocket_rejects_unmasked_client_frame() {
        let mut observer = WebSocketObserver::new(BodyDirection::Request, true, false, 32, 32);
        let events = observer.feed(&[0x81, 1, b'x']);
        assert!(events.iter().any(|event| matches!(
            event,
            StreamingEvent::WebSocketTerminal {
                outcome: StreamingOutcome::Malformed,
                ..
            }
        )));
    }

    #[test]
    fn websocket_decodes_permessage_deflate() {
        let mut compressor = Compress::new(Compression::fast(), false);
        let mut compressed = Vec::with_capacity(128);
        compressor
            .compress_vec(b"compressed text", &mut compressed, FlushCompress::Sync)
            .expect("compress message");
        assert!(compressed.ends_with(&[0, 0, 0xff, 0xff]));
        compressed.truncate(compressed.len() - 4);
        let key = [9, 8, 7, 6];
        let mut frame = vec![0xc1, 0x80 | compressed.len() as u8];
        frame.extend_from_slice(&key);
        frame.extend(
            compressed
                .iter()
                .enumerate()
                .map(|(index, byte)| byte ^ key[index % 4]),
        );
        let mut observer = WebSocketObserver::new(BodyDirection::Request, true, true, 1024, 1024);
        let events = observer.feed(&frame);
        assert!(events.iter().any(|event| matches!(event, StreamingEvent::WebSocketMessage(message) if message.payload == b"compressed text" && message.compressed)));
    }

    #[test]
    fn sse_parses_fields_and_reconnect_metadata_incrementally() {
        let mut observer = SseObserver::new(64, 128);
        let mut events = observer.feed(b"id: 7\r");
        events.extend(observer.feed(b"\nevent: tick\ndata: a\ndata: b\nretry: 500\n\n"));
        assert!(events.iter().any(|event| matches!(event, StreamingEvent::SseEvent(value) if value.data == b"a\nb" && value.last_event_id == b"7" && value.retry_ms == Some(500))));
    }

    #[test]
    fn sse_reports_line_and_event_limits() {
        let mut observer = SseObserver::new(4, 3);
        let events = observer.feed(b"toolong\ndata: value\n\n");
        assert!(events.iter().any(|event| matches!(event, StreamingEvent::SseField(value) if value.outcome == StreamingOutcome::Limit)));
        assert!(events.iter().any(|event| matches!(event, StreamingEvent::SseEvent(value) if value.outcome == StreamingOutcome::Limit)));
    }

    #[test]
    fn sse_handles_split_bom_and_ten_thousand_events() {
        let mut observer = SseObserver::new(64, 64);
        assert!(observer.feed(&[0xef]).is_empty());
        assert!(observer.feed(&[0xbb]).is_empty());
        let mut count = observer
            .feed(&[0xbf])
            .into_iter()
            .filter(|event| matches!(event, StreamingEvent::SseEvent(_)))
            .count();
        for _ in 0..10_000 {
            count += observer
                .feed(b"data: tick\n\n")
                .into_iter()
                .filter(|event| matches!(event, StreamingEvent::SseEvent(_)))
                .count();
        }
        assert_eq!(count, 10_000);
    }

    #[test]
    fn websocket_records_control_and_close_semantics() {
        let mut observer = WebSocketObserver::new(BodyDirection::Response, false, false, 128, 128);
        let mut events = observer.feed(b"\x89\x01x\x8a\x01y\x88\x05\x03\xe8bye");
        events.push(observer.finish(StreamingOutcome::Complete));
        assert!(events.iter().any(
            |event| matches!(event, StreamingEvent::WebSocketFrame(value) if value.opcode == 9)
        ));
        assert!(events.iter().any(
            |event| matches!(event, StreamingEvent::WebSocketFrame(value) if value.opcode == 10)
        ));
        assert!(events.iter().any(|event| matches!(event, StreamingEvent::WebSocketFrame(value) if value.close_code == Some(1000) && value.close_reason == b"bye")));
    }

    #[test]
    fn grpc_parses_multiple_envelopes_across_chunks() {
        let mut observer = GrpcObserver::new(BodyDirection::Response, 32);
        let mut events = observer.feed(&[0, 0, 0]);
        events.extend(observer.feed(&[0, 2, 1, 2, 1, 0, 0, 0, 1, 9]));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, StreamingEvent::GrpcMessage(_)))
                .count(),
            2
        );
    }

    #[test]
    fn grpc_reports_oversized_without_buffering_payload() {
        let mut observer = GrpcObserver::new(BodyDirection::Request, 2);
        let events = observer.feed(&[0, 0, 0, 0, 3, 1, 2, 3]);
        assert!(
            matches!(&events[0], StreamingEvent::GrpcMessage(value) if value.outcome == StreamingOutcome::Limit && value.payload.is_empty())
        );
    }

    #[test]
    fn grpc_reports_invalid_compression_flag() {
        let mut observer = GrpcObserver::new(BodyDirection::Request, 8);
        let events = observer.feed(&[2, 0, 0, 0, 0]);
        assert!(
            matches!(&events[0], StreamingEvent::GrpcMessage(value) if value.outcome == StreamingOutcome::Malformed)
        );
    }
}
