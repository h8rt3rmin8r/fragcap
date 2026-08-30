// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use ring::rand::{SecureRandom, SystemRandom};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

pub const CAPABILITY_BYTES: usize = 32;

/// Opaque, session-specific proof used before any proxy payload is accepted.
pub struct SessionCapability(Zeroizing<[u8; CAPABILITY_BYTES]>);

impl SessionCapability {
    pub fn generate() -> Result<Self, CapabilityError> {
        let mut bytes = Zeroizing::new([0_u8; CAPABILITY_BYTES]);
        SystemRandom::new()
            .fill(bytes.as_mut())
            .map_err(|_| CapabilityError)?;
        Ok(Self(bytes))
    }

    pub fn proof(&self) -> CapabilityProof {
        CapabilityProof(Zeroizing::new(*self.0))
    }

    pub fn authenticates(&self, candidate: &[u8]) -> bool {
        candidate.len() == CAPABILITY_BYTES && self.0.as_slice().ct_eq(candidate).into()
    }
}

impl Clone for SessionCapability {
    fn clone(&self) -> Self {
        Self(Zeroizing::new(*self.0))
    }
}

impl fmt::Debug for SessionCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SessionCapability([REDACTED])")
    }
}

/// Transfer-only proof. Debug output never reveals its bytes.
pub struct CapabilityProof(Zeroizing<[u8; CAPABILITY_BYTES]>);

impl CapabilityProof {
    pub fn as_bytes(&self) -> &[u8; CAPABILITY_BYTES] {
        &self.0
    }
}

impl Clone for CapabilityProof {
    fn clone(&self) -> Self {
        Self(Zeroizing::new(*self.0))
    }
}

impl fmt::Debug for CapabilityProof {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CapabilityProof([REDACTED])")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityError;

impl fmt::Display for CapabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("operating-system entropy was unavailable")
    }
}

impl std::error::Error for CapabilityError {}
