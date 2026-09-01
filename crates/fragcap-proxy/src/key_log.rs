// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

const LABELS: &[&str] = &[
    "CLIENT_RANDOM",
    "CLIENT_EARLY_TRAFFIC_SECRET",
    "SERVER_HANDSHAKE_TRAFFIC_SECRET",
    "CLIENT_HANDSHAKE_TRAFFIC_SECRET",
    "SERVER_TRAFFIC_SECRET_0",
    "CLIENT_TRAFFIC_SECRET_0",
    "EXPORTER_SECRET",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyLogStatus {
    pub records: u64,
    pub bytes: u64,
    pub flushes: u64,
    pub failure: Option<String>,
}

struct State {
    file: File,
    status: KeyLogStatus,
}

/// Session-scoped, live-flushed NSS key log for client-facing proxy TLS only.
pub struct SessionKeyLog(Mutex<State>);

impl SessionKeyLog {
    pub fn new(file: File) -> Self {
        Self(Mutex::new(State {
            file,
            status: KeyLogStatus {
                records: 0,
                bytes: 0,
                flushes: 0,
                failure: None,
            },
        }))
    }

    pub fn status(&self) -> KeyLogStatus {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .status
            .clone()
    }
}

impl fmt::Debug for SessionKeyLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionKeyLog")
            .field("status", &self.status())
            .finish()
    }
}

impl rustls::KeyLog for SessionKeyLog {
    fn log(&self, label: &str, client_random: &[u8], secret: &[u8]) {
        if !self.will_log(label) {
            return;
        }
        let mut state = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut line =
            Vec::with_capacity(label.len() + 2 * (client_random.len() + secret.len()) + 3);
        line.extend_from_slice(label.as_bytes());
        line.push(b' ');
        append_hex(&mut line, client_random);
        line.push(b' ');
        append_hex(&mut line, secret);
        line.push(b'\n');
        match state.file.write_all(&line).and_then(|_| state.file.flush()) {
            Ok(()) => {
                state.status.records = state.status.records.saturating_add(1);
                state.status.bytes = state.status.bytes.saturating_add(line.len() as u64);
                state.status.flushes = state.status.flushes.saturating_add(1);
            }
            Err(error) => state.status.failure = Some(error.kind().to_string()),
        }
    }

    fn will_log(&self, label: &str) -> bool {
        LABELS.contains(&label) && self.status().failure.is_none()
    }
}

fn append_hex(out: &mut Vec<u8>, bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize]);
        out.push(HEX[(byte & 0x0f) as usize]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustls::KeyLog;
    use std::io::{Read, Seek, SeekFrom};
    use std::sync::Arc;

    #[test]
    fn writes_complete_nss_records_and_redacts_debug() {
        let mut file = tempfile::tempfile().unwrap();
        let logger = SessionKeyLog::new(file.try_clone().unwrap());
        logger.log("CLIENT_RANDOM", &[0xab, 0xcd], &[1, 2, 3]);
        file.seek(SeekFrom::Start(0)).unwrap();
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        assert_eq!(text, "CLIENT_RANDOM abcd 010203\n");
        assert_eq!(logger.status().records, 1);
        assert!(!format!("{logger:?}").contains("010203"));
        assert!(!logger.will_log("UNKNOWN_FUTURE_SECRET"));
    }

    #[test]
    fn serializes_one_hundred_concurrent_records_without_tearing() {
        let mut file = tempfile::tempfile().unwrap();
        let logger = Arc::new(SessionKeyLog::new(file.try_clone().unwrap()));
        let tasks: Vec<_> = (0_u8..100)
            .map(|value| {
                let logger = Arc::clone(&logger);
                std::thread::spawn(move || {
                    logger.log("CLIENT_RANDOM", &[value; 32], &[value.wrapping_add(1); 48]);
                })
            })
            .collect();
        for task in tasks {
            task.join().unwrap();
        }

        file.seek(SeekFrom::Start(0)).unwrap();
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 100);
        assert!(lines.iter().all(|line| {
            let fields: Vec<_> = line.split_ascii_whitespace().collect();
            fields.len() == 3
                && fields[0] == "CLIENT_RANDOM"
                && fields[1].len() == 64
                && fields[2].len() == 96
                && fields[1..]
                    .iter()
                    .all(|field| field.bytes().all(|byte| byte.is_ascii_hexdigit()))
        }));
        assert_eq!(logger.status().records, 100);
        assert_eq!(logger.status().flushes, 100);
    }

    #[test]
    fn a_write_failure_is_sticky_and_non_secret() {
        let named = tempfile::NamedTempFile::new().unwrap();
        let logger = SessionKeyLog::new(File::open(named.path()).unwrap());
        logger.log("CLIENT_RANDOM", &[0xaa; 32], &[0xbb; 48]);
        let status = logger.status();
        assert_eq!(status.records, 0);
        assert!(status.failure.is_some());
        assert!(!logger.will_log("CLIENT_RANDOM"));
        let debug = format!("{logger:?}");
        assert!(!debug.contains("aaaa"));
        assert!(!debug.contains("bbbb"));
    }
}
