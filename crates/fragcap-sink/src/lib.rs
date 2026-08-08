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
//! What this crate still does not do: JSON Lines output arrives in S07,
//! transports and streaming sinks in S15, and ring mode in S16. The pipeline
//! that drives any of them is S08.
//!
//! The corpus-driven tests for this crate live in the `fragcap` facade rather
//! than here. Writing a fixture needs a replay source and a scripted
//! attributor, which are siblings of this crate, and reaching them from a
//! `tests/` directory would mean a dev-dependency on a sibling: the edge
//! constitution P-3 exists to prevent, and one `cargo xtask deps` does not
//! catch, since it ignores dev-dependencies by design.

pub mod annotation;
pub mod error;
pub mod pcapng;

pub use annotation::{AnnotatedDirection, Annotation, AnnotationError, Fidelity};
pub use error::WriteError;
pub use pcapng::interface::InterfaceDeclaration;
pub use pcapng::PcapngWriter;
