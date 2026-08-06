// SPDX-License-Identifier: Apache-2.0

//! Packet acquisition backends implementing the `PacketSource` seam.
//!
//! Kept separate from attribution by constitution principle P-3: the two have
//! different platform requirements, different failure modes, and different
//! upgrade paths, and separating them is what makes the pipeline testable
//! offline.
//!
//! Skeleton only. Live acquisition arrives in S09.
