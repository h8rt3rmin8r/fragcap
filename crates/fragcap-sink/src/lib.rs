// SPDX-License-Identifier: Apache-2.0

//! Output sinks and transports.
//!
//! Slice S06 filled this crate with the pcapng writer of specification sections
//! 13.1 through 13.4. It is the first thing in the project that produces a
//! file, and the format it produces is the product claim: attribution travels
//! in an ordinary pcapng comment, so an analyzer that has never heard of
//! fragcap opens the capture and displays it with no plugin and no
//! configuration. Constitution P-5 is why, and it decides every open question
//! here.
//!
//! [`annotation`] is deliberately separate from [`pcapng`]. Deriving which keys
//! an annotation carries is shared with the JSON Lines writer that S07 brings;
//! rendering it as a `fragcap:` string is not. Two independent derivations
//! would drift, and the drift would be silent because each would be internally
//! consistent.
//!
//! Slice S07 added the second format, [`json`], for consumers that do not read
//! pcapng. It is the test of whether the split above was real: the JSON writer
//! reads an `Annotation` and renders it, and restates no presence rule. What
//! differs between the two is confined to rendering, where a reviewer can see
//! it: `iface` on every JSON record because a line is self-contained, lowercase
//! hex, and endpoints named for what is known about them.
//!
//! What this crate still does not do: transports and streaming sinks arrive in
//! S15, and ring mode in S16.
//!
//! The pipeline that drives these arrived in S08 and lives in
//! `fragcap_core::pipeline`. Both writers are now fed by it over the whole
//! fixture corpus, and the statistics they write into their trailing blocks are
//! the run's own rather than a snapshot a test composed by hand. That change
//! found one defect immediately: the S07 corpus helper folded packets that
//! produced no flow key into `packets_unattributed`, and the `malformed`
//! golden had been carrying the wrong count ever since. The writers were
//! faithful; what they were handed was not.
//!
//! The corpus-driven tests for this crate live in the `fragcap` facade rather
//! than here. Writing a fixture needs a replay source and a scripted
//! attributor, which are siblings of this crate, and reaching them from a
//! `tests/` directory would mean a dev-dependency on a sibling: the edge
//! constitution P-3 exists to prevent, and one `cargo xtask deps` does not
//! catch, since it ignores dev-dependencies by design.

pub mod annotation;
pub mod error;
pub mod json;
pub mod pcapng;

pub use annotation::{AnnotatedDirection, Annotation, AnnotationError, Fidelity};
pub use error::WriteError;
pub use json::{JsonLinesWriter, PayloadMode};
pub use pcapng::interface::InterfaceDeclaration;
pub use pcapng::PcapngWriter;
