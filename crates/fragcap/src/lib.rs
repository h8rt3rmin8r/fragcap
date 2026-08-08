// SPDX-License-Identifier: Apache-2.0

//! Facade crate for fragcap.
//!
//! The library is the product; the command line tool is one consumer of it.
//! Anything reachable through the CLI is reachable through this crate.
//!
//! Depends on `fragcap-core` directly, in addition to the mid-level crates.
//! The dependency diagram in specification section 8.3 omits that edge, but a
//! facade that re-exports core types needs core as a direct dependency. The
//! edge violates neither stated rule: it is not a dependency on the binary
//! crate, and it is not a sibling-level dependency. Recorded as decision D-1.
//!
//! Re-exports arrive as the crates below gain surface. Slice S04 is the first
//! to add any: the tier 1 substrate that makes specification section 25.1's
//! claim true, namely that the whole pipeline runs with no capture driver, no
//! elevated privilege, and no game.

/// Types, traits, and the header parser.
pub mod core {
    pub use fragcap_core::attribution::{Attribution, StageId};
    pub use fragcap_core::error::{AttrError, SinkError, SourceError};
    pub use fragcap_core::flow::{AttributionKey, Direction, Endpoint, FlowKey, Proto};
    pub use fragcap_core::link::LinkType;
    pub use fragcap_core::packet::{
        AttributionState, CapturedPacket, Payload, RawPacket, Timestamp,
    };
    pub use fragcap_core::parse::{HeaderParser, InterfaceAddrs, ParseOutcome, ParseReject};
    pub use fragcap_core::stats::{CaptureStats, ParseStats, SourceStats};
    pub use fragcap_core::traits::{Dissector, FlowAttributor, PacketSource, ProcessWatcher, Sink};
}

/// Packet acquisition.
pub mod capture {
    pub use fragcap_capture::pcap::{PcapReader, ReplayStats};
    pub use fragcap_capture::replay::ReplaySource;
}

/// Flow attribution.
pub mod attr {
    pub use fragcap_attr::script::{AttributionScript, ScriptEntry, ScriptError, Window};
    pub use fragcap_attr::scripted::ScriptedAttributor;
}

/// Output sinks.
pub mod sink {
    pub use fragcap_sink::annotation::{
        AnnotatedDirection, Annotation, AnnotationError, Fidelity, SENTINEL,
    };
    pub use fragcap_sink::error::WriteError;
    pub use fragcap_sink::pcapng::interface::InterfaceDeclaration;
    pub use fragcap_sink::pcapng::{PcapngWriter, PROFILE_COMMENT, USER_APPL};
}

pub use crate::core::*;
pub use attr::{AttributionScript, ScriptedAttributor};
pub use capture::{PcapReader, ReplaySource, ReplayStats};
pub use sink::{
    AnnotatedDirection, Annotation, AnnotationError, Fidelity, InterfaceDeclaration, PcapngWriter,
    WriteError,
};
