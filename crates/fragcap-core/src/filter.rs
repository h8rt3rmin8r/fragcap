// SPDX-License-Identifier: Apache-2.0

//! Capture filter programs handed to a packet source.

/// A capture filter to be installed on a [`crate::traits::PacketSource`].
///
/// Minimal on purpose. Slice S13 owns filter management, including the
/// narrowing strategy in specification section 12.3 and the filter gap
/// accounting that goes with it. This type exists now so the
/// [`crate::traits::PacketSource`] signature is expressible, and it will gain
/// structure when the slice that uses it lands.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct FilterProgram {
    expression: String,
}

impl FilterProgram {
    pub fn new(expression: impl Into<String>) -> Self {
        FilterProgram {
            expression: expression.into(),
        }
    }

    /// The filter expression, in the backend's own syntax.
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Whether this program selects everything, which is the state a capture
    /// starts in before narrowing.
    pub fn is_empty(&self) -> bool {
        self.expression.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_expression_round_trips() {
        let f = FilterProgram::new("tcp port 443");
        assert_eq!(f.expression(), "tcp port 443");
        assert!(!f.is_empty());
    }

    #[test]
    fn the_default_selects_everything() {
        assert!(FilterProgram::default().is_empty());
    }
}
