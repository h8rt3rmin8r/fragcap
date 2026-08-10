// SPDX-License-Identifier: Apache-2.0

//! Packet acquisition backends implementing the `PacketSource` seam.
//!
//! Kept separate from attribution by constitution principle P-3: the two have
//! different platform requirements, different failure modes, and different
//! upgrade paths, and separating them is what makes the pipeline testable
//! offline.
//!
//! That separation stopped being a claim in slice S04. [`replay::ReplaySource`]
//! reads a recorded capture file, and with the scripted attributor in
//! `fragcap-attr` it makes the whole pipeline a deterministic function from
//! fixture input to output, which is what specification section 25.1 promises.
//! Neither crate depends on the other, and `cargo xtask deps` rejects the edge
//! if one ever tries.
//!

//! Live acquisition arrived in S09 and shares nothing with the replay path: a
//! file is never slow, never drops, and never disappears mid-read. It is
//! compiled only under the `live` feature, which is off by default, so building
//! this crate and running the ordinary check set needs neither the capture
//! driver nor its software development kit.

#[cfg(all(windows, feature = "live"))]
pub mod live;
pub mod pcap;
pub mod replay;

pub use pcap::{PcapReader, ReplayStats};
pub use replay::ReplaySource;
