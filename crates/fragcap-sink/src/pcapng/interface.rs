// SPDX-License-Identifier: Apache-2.0

//! Interface declarations and the identifiers assigned to them.

use std::sync::Arc;

use fragcap_core::LinkType;

/// One capture interface, as declared by the caller before any packet
/// references it.
///
/// Declared rather than inferred. Inferring an interface from the first packet
/// would mean the writer guessing a link type and a name it was never told,
/// which is a fabricated statement about the capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceDeclaration {
    pub link_type: LinkType,
    /// Declared, never enforced against packet contents. A packet longer than
    /// this is written as recorded; see the writer.
    pub snap_len: u32,
    pub name: Arc<str>,
}

impl InterfaceDeclaration {
    pub fn new(link_type: LinkType, snap_len: u32, name: impl AsRef<str>) -> Self {
        InterfaceDeclaration {
            link_type,
            snap_len,
            name: Arc::from(name.as_ref()),
        }
    }
}

/// A declaration plus the state the writer keeps for it.
#[derive(Clone, Debug)]
pub(crate) struct DeclaredInterface {
    pub(crate) decl: InterfaceDeclaration,
    /// The timestamp of the last packet written against this interface, in
    /// microseconds. `None` until one is.
    ///
    /// This is the Interface Statistics Block's timestamp. Deriving it from the
    /// data rather than from a clock is what keeps the writer a pure function
    /// of its input, which the golden comparison depends on.
    pub(crate) last_ts_micros: Option<u64>,
}

impl DeclaredInterface {
    pub(crate) fn new(decl: InterfaceDeclaration) -> Self {
        DeclaredInterface {
            decl,
            last_ts_micros: None,
        }
    }
}
