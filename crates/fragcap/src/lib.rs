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
//! Skeleton only. Re-exports arrive as the crates below gain surface.
