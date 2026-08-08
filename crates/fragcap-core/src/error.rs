// SPDX-License-Identifier: Apache-2.0

//! Failures the behavioral seams can produce.
//!
//! Each enum names its variants rather than being opaque, because a caller has
//! to act differently on different failures: a source that times out with no
//! traffic is a normal condition and the capture loop continues, while a source
//! whose device has disappeared is terminal. Deciding that by inspecting a
//! message string is not a contract.
//!
//! `Display` and `Error` are hand-written rather than derived. See slice S02
//! plan decision D-3: a proc-macro derive would take the workspace's dependency
//! graph from one crate to four to save about forty lines of mechanical code.
//!
//! Every enum is `#[non_exhaustive]`, because slices S09, S15, and S16 each add
//! failure modes that cannot be enumerated now, and without it each addition
//! would be a breaking change for every caller.

use std::error::Error;
use std::fmt;

/// A failure acquiring packets.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceError {
    /// No packet arrived within the timeout. Normal; the capture loop
    /// continues. This is the variant that makes the enum worth having.
    Timeout,
    /// The source will produce no further packets. Terminal.
    Closed,
    /// The capture device is gone, for example an interface that was removed.
    /// Terminal.
    DeviceLost { detail: String },
    /// The backend rejected the filter program.
    FilterRejected { detail: String },
    /// The backend failed in a way fragcap does not model.
    Backend { detail: String },
}

impl SourceError {
    /// Whether a capture loop may continue after this error.
    ///
    /// Only a timeout is recoverable. Everything else means the source will not
    /// produce useful packets again, and continuing would spin.
    pub fn is_recoverable(&self) -> bool {
        matches!(self, SourceError::Timeout)
    }
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceError::Timeout => f.write_str("no packet arrived within the timeout"),
            SourceError::Closed => f.write_str("the packet source is closed"),
            SourceError::DeviceLost { detail } => {
                write!(f, "the capture device is no longer available: {detail}")
            }
            SourceError::FilterRejected { detail } => {
                write!(f, "the backend rejected the filter program: {detail}")
            }
            SourceError::Backend { detail } => write!(f, "capture backend failure: {detail}"),
        }
    }
}

impl Error for SourceError {}

/// A failure resolving or refreshing attribution.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AttrError {
    /// The socket table could not be read this time. Transient; the next
    /// refresh may succeed, and packets in the meantime are retained and marked
    /// rather than dropped.
    RefreshFailed { detail: String },
    /// The platform facility attribution depends on is not available at all.
    /// Terminal for attribution, but not for capture: fragcap continues and
    /// marks every packet unattributed.
    Unavailable { detail: String },
}

impl AttrError {
    /// Whether a later refresh might succeed.
    pub fn is_transient(&self) -> bool {
        matches!(self, AttrError::RefreshFailed { .. })
    }
}

impl fmt::Display for AttrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttrError::RefreshFailed { detail } => {
                write!(f, "could not refresh the socket table: {detail}")
            }
            AttrError::Unavailable { detail } => {
                write!(f, "attribution is unavailable on this platform: {detail}")
            }
        }
    }
}

impl Error for AttrError {}

/// A failure writing output.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SinkError {
    /// The sink could not accept this packet now. The pipeline counts this in
    /// `sink_dropped` rather than aborting the capture, per specification
    /// section 12.4.
    Full,
    /// The sink will accept nothing further.
    Closed,
    /// The write failed.
    Write { detail: String },
}

impl SinkError {
    /// Whether the pipeline should count this and carry on rather than stop.
    pub fn is_countable(&self) -> bool {
        matches!(self, SinkError::Full)
    }
}

impl fmt::Display for SinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SinkError::Full => f.write_str("the sink could not accept the packet"),
            SinkError::Closed => f.write_str("the sink is closed"),
            SinkError::Write { detail } => write!(f, "sink write failed: {detail}"),
        }
    }
}

impl Error for SinkError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_timeout_is_the_only_recoverable_source_error() {
        assert!(SourceError::Timeout.is_recoverable());
        assert!(!SourceError::Closed.is_recoverable());
        assert!(!SourceError::DeviceLost {
            detail: "removed".into()
        }
        .is_recoverable());
    }

    #[test]
    fn a_refresh_failure_is_transient_but_unavailability_is_not() {
        assert!(AttrError::RefreshFailed {
            detail: "busy".into()
        }
        .is_transient());
        assert!(!AttrError::Unavailable {
            detail: "no ETW".into()
        }
        .is_transient());
    }

    #[test]
    fn a_full_sink_is_counted_rather_than_fatal() {
        assert!(SinkError::Full.is_countable());
        assert!(!SinkError::Closed.is_countable());
    }

    #[test]
    fn every_variant_displays_something_useful() {
        let cases: Vec<Box<dyn Error>> = vec![
            Box::new(SourceError::Timeout),
            Box::new(SourceError::Closed),
            Box::new(SourceError::DeviceLost {
                detail: "removed".into(),
            }),
            Box::new(SourceError::FilterRejected {
                detail: "syntax".into(),
            }),
            Box::new(SourceError::Backend {
                detail: "oops".into(),
            }),
            Box::new(AttrError::RefreshFailed {
                detail: "busy".into(),
            }),
            Box::new(AttrError::Unavailable {
                detail: "no ETW".into(),
            }),
            Box::new(SinkError::Full),
            Box::new(SinkError::Closed),
            Box::new(SinkError::Write {
                detail: "disk".into(),
            }),
        ];
        for e in cases {
            let text = e.to_string();
            assert!(!text.is_empty(), "an error must say something");
            assert!(
                !text.contains("Error") || text.len() > 12,
                "a message must be more than a type name: {text}"
            );
        }
    }

    #[test]
    fn detail_reaches_the_message() {
        let e = SourceError::Backend {
            detail: "npcap handle invalid".into(),
        };
        assert!(e.to_string().contains("npcap handle invalid"));
    }

    // The errors are usable as trait objects, which the pipeline needs in order
    // to report a cause chain from a boxed sink.
    #[test]
    fn errors_are_usable_as_std_error_trait_objects() {
        let boxed: Box<dyn Error> = Box::new(SinkError::Full);
        assert!(boxed.source().is_none());
    }
}
