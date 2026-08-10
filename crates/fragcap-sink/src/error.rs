// SPDX-License-Identifier: Apache-2.0

//! What a writer refuses, and why.
//!
//! Every variant here is a refusal, not a discard. A packet this crate declines
//! to write is reported to the caller so the pipeline can count it, because a
//! sink that silently skipped a packet would produce a capture that is quietly
//! short, which constitution P-4 treats as the one defect class that corrupts
//! every other conclusion drawn from the output.

use std::error::Error;
use std::fmt;

use fragcap_core::error::SinkError;

/// A condition that stopped the writer from recording an observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteError {
    /// A packet named an interface that was never declared.
    ///
    /// Not recoverable by inventing one. An interface carries a link type, a
    /// snap length, and a name, none of which the writer could supply without
    /// making them up, and a fabricated interface declaration is an assertion
    /// about the capture that nobody observed.
    UndeclaredInterface { id: u32 },

    /// A timestamp that predates the Unix epoch.
    ///
    /// pcapng timestamps are unsigned, so the value has no representation.
    /// Clamping to zero would record the observation at a time it did not
    /// happen, and wrapping would place it roughly half a million years in the
    /// future. Both are unrecoverable by a reader, so the writer refuses
    /// instead. See specification section 12.7 and constitution P-9.
    TimestampBeforeEpoch { nanos: i64 },

    /// An interface was declared after a packet had already been written.
    ///
    /// This replaces S06's blanket refusal of a second interface, and the
    /// narrower rule is the one that was actually needed. Section 13.3 writes
    /// the annotation `iface` key only in a multi-interface capture, so whether
    /// the key appears has to be settled before the first packet block is
    /// written; a declaration arriving later would leave every earlier block
    /// without a key it retrospectively needed, and pcapng blocks are not
    /// revisable.
    ///
    /// Declaring every interface up front costs a caller nothing, because
    /// selection settles the whole set before capture starts.
    InterfaceAfterPacket,

    /// An option value longer than a pcapng option length field can express.
    ///
    /// The field is 16 bits. Truncating the value would silently alter what was
    /// recorded, which is the same defect class as a silent drop.
    OptionTooLong { code: u16, len: usize },

    /// The underlying writer failed.
    ///
    /// Blocks already written stay valid; pcapng is a sequence of
    /// self-delimiting blocks, so a reader gets everything up to the failure.
    Io { detail: String },
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::UndeclaredInterface { id } => {
                write!(f, "no interface with identifier {id} was declared")
            }
            WriteError::TimestampBeforeEpoch { nanos } => write!(
                f,
                "timestamp {nanos} ns predates the Unix epoch and pcapng cannot represent it"
            ),
            WriteError::InterfaceAfterPacket => f.write_str(
                "an interface was declared after a packet was written; declare them all first",
            ),
            WriteError::OptionTooLong { code, len } => write!(
                f,
                "option {code} value is {len} bytes, over the 65535 byte limit"
            ),
            WriteError::Io { detail } => write!(f, "write failed: {detail}"),
        }
    }
}

impl Error for WriteError {}

impl From<std::io::Error> for WriteError {
    fn from(e: std::io::Error) -> Self {
        WriteError::Io {
            detail: e.to_string(),
        }
    }
}

impl From<WriteError> for SinkError {
    /// Every writer failure maps to [`SinkError::Write`], never to
    /// [`SinkError::Full`].
    ///
    /// The distinction is load-bearing: `Full` is the variant the pipeline
    /// counts in `sink_dropped` and carries on from, and none of these
    /// conditions is a transient backpressure signal. A pre-epoch timestamp
    /// will not succeed on a retry, and reporting it as a countable drop would
    /// turn a defect in the input into a statistic.
    fn from(e: WriteError) -> Self {
        SinkError::Write {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_write_error_is_countable_as_backpressure() {
        // If any of these ever mapped to `Full`, the pipeline would count a
        // structural defect as an ordinary drop and keep going.
        let cases = [
            WriteError::UndeclaredInterface { id: 3 },
            WriteError::TimestampBeforeEpoch { nanos: -1 },
            WriteError::OptionTooLong {
                code: 1,
                len: 70_000,
            },
            WriteError::InterfaceAfterPacket,
            WriteError::Io {
                detail: "disk full".into(),
            },
        ];
        for case in cases {
            let mapped: SinkError = case.into();
            assert!(!mapped.is_countable(), "a writer refusal is not a drop");
        }
    }

    #[test]
    fn messages_name_the_offending_value() {
        // A reviewer reading a failure should not have to open the writer to
        // learn which interface or which timestamp was at fault.
        assert!(WriteError::UndeclaredInterface { id: 7 }
            .to_string()
            .contains('7'));
        assert!(WriteError::TimestampBeforeEpoch { nanos: -5 }
            .to_string()
            .contains("-5"));
        assert!(WriteError::OptionTooLong {
            code: 1,
            len: 70_000
        }
        .to_string()
        .contains("70000"));
    }
}
