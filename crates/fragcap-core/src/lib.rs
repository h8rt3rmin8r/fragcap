// SPDX-License-Identifier: Apache-2.0

//! Core types, traits, and pipeline vocabulary for fragcap.
//!
//! This crate is the vocabulary the other seven are written in. Most of it
//! fixes the shape of the seams the later slices are built against: nothing
//! here captures a packet, resolves an attribution, or writes a file.
//!
//! Two modules are behavior. [`parse`] arrived in slice S03 and turns a frame
//! into the identity of the conversation it belongs to. It is here rather than
//! in `fragcap-capture` because the capture thread that calls it belongs to
//! the pipeline, which specification section 8.2 places in this crate, and
//! because parsing is arithmetic over a byte slice with no platform surface
//! for constitution P-2 to object to.
//!
//! [`pipeline`] arrived in slice S08 and is the thing that argument was about.
//! It composes a source, the parser, an attributor, and a set of sinks across
//! two threads with a bounded buffer between them, per specification sections
//! 8.6 and 12.4. It is here for the same reason `parse` is, and it is what
//! finally produces the [`stats::CaptureStats`] values the writers had been
//! handed by hand since S06. Threads and mutexes are standard library
//! facilities with no platform surface, which `cargo xtask neutral` proves
//! rather than asserts.
//!
//! [`duration`] arrived in slice S05 and is the third. It parses a duration
//! literal as an operator writes it. It is here rather than in
//! `fragcap-profile` because three slices need the same grammar (a profile's
//! `capture.duration`, the command line's `--duration` and `--wait`, and the
//! ring window), section 8.3 forbids a crate below the facade depending on a
//! sibling, and two implementations of `30m` that disagree would produce a
//! capture of the wrong length.
//!
//! # Platform neutrality
//!
//! Constitution P-2: this crate takes no platform-specific dependency, no I/O
//! crate, and no capture library. Continuous integration proves it by building
//! this crate for a target where no capture backend exists. Its one external
//! dependency, `bytes`, is pure Rust with no platform surface.
//!
//! # What the type system is doing here
//!
//! Three constitution principles are enforced structurally rather than by
//! documentation, because a rule that lives only in prose is a rule someone
//! violates without reading it.
//!
//! **P-1, passive observation.** Nothing in these traits obliges an implementor
//! to open a process handle, inject code, or hook a function. Process ancestry
//! arrives as creation-time events, and flow ownership arrives from the socket
//! table, both from outside the target.
//!
//! **P-4, no silent loss.** [`stats::CaptureStats`] carries one named counter
//! per discard cause, named as specification section 12.4 names them, and every
//! total is a method rather than a field so it cannot drift from its parts. An
//! unattributed packet is retained and marked, never dropped, and
//! [`packet::AttributionState`] is what marks it.
//!
//! **P-9, the instrument does not lie.** No public operation in this crate
//! alters, masks, truncates, reorders, or withholds an observed field. The one
//! place the temptation arises is truncation, and [`packet::RawPacket`] keeps
//! the original on-wire length beside the possibly shorter payload so a
//! shortened capture says so. There is deliberately no microsecond conversion
//! on [`packet::Timestamp`]: the single lossy conversion happens at the output
//! boundary in slice S06, so there is one site to inspect rather than many.
//!
//! Specification section 8.4 additionally requires that an implementation never
//! invent a remote endpoint for a UDP socket table entry, because that produces
//! confident wrong attributions rather than honest coarse ones.
//! [`flow::AttributionKey`] has no variant that could express one.
//!
//! # What is not settled
//!
//! Types whose documentation names a later slice are expected to change when
//! that slice lands: [`filter::FilterProgram`] at S13, [`process::ProcessEvent`]
//! and [`process::ProcessRecord`] at S11, [`link::LinkType`] when S09 discovers
//! what the capture backend actually reports. The five traits in [`traits`] are
//! the part intended to survive to 1.0.0 unchanged.

pub mod attribution;
pub mod duration;
pub mod error;
pub mod filter;
pub mod flow;
pub mod link;
pub mod packet;
pub mod parse;
pub mod pipeline;
pub mod process;
pub mod stats;
pub mod traits;

pub use attribution::{Attribution, Fidelity, StageId};
pub use duration::DurationError;
pub use error::{AttrError, SinkError, SourceError};
pub use filter::FilterProgram;
pub use flow::{AttributionKey, Direction, Endpoint, FlowKey, Proto};
pub use link::LinkType;
pub use packet::{AttributionState, CapturedPacket, Payload, RawPacket, Timestamp};
pub use parse::{HeaderParser, InterfaceAddrs, ParseOutcome, ParseReject};
pub use pipeline::{
    ConfigError, EndReason, Pipeline, PipelineConfig, PipelineError, PipelineReport, SinkFailure,
    StopHandle,
};
pub use process::{ProcessEvent, ProcessRecord};
pub use stats::{CaptureStats, ParseStats, SourceStats};
pub use traits::{Dissector, FlowAttributor, PacketSource, ProcessWatcher, Sink};
