// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::net::SocketAddr;

use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use ring::rand::{SecureRandom, SystemRandom};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

pub const CAPABILITY_BYTES: usize = 32;
pub const PROXY_USERNAME: &str = "fragcap";

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

    pub fn proxy_password(&self) -> Zeroizing<String> {
        Zeroizing::new(URL_SAFE_NO_PAD.encode(self.0.as_slice()))
    }

    pub fn proxy_url(&self, endpoint: SocketAddr) -> Zeroizing<String> {
        Zeroizing::new(format!(
            "http://{PROXY_USERNAME}:{}@{endpoint}",
            self.proxy_password().as_str()
        ))
    }

    pub fn proxy_authorization(&self) -> Zeroizing<String> {
        let credentials = Zeroizing::new(format!(
            "{PROXY_USERNAME}:{}",
            self.proxy_password().as_str()
        ));
        Zeroizing::new(format!("Basic {}", STANDARD.encode(credentials.as_bytes())))
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProxyAuthorizationError {
    Missing,
    Duplicate,
    Malformed,
    Refused,
}

impl SessionCapability {
    pub fn authenticates_proxy_authorization(
        &self,
        value: Option<&[u8]>,
    ) -> Result<(), ProxyAuthorizationError> {
        let value = value.ok_or(ProxyAuthorizationError::Missing)?;
        let encoded = value
            .strip_prefix(b"Basic ")
            .ok_or(ProxyAuthorizationError::Malformed)?;
        let decoded = Zeroizing::new(
            STANDARD
                .decode(encoded)
                .map_err(|_| ProxyAuthorizationError::Malformed)?,
        );
        let separator = decoded
            .iter()
            .position(|byte| *byte == b':')
            .ok_or(ProxyAuthorizationError::Malformed)?;
        if &decoded[..separator] != PROXY_USERNAME.as_bytes() {
            return Err(ProxyAuthorizationError::Refused);
        }
        let proof = Zeroizing::new(
            URL_SAFE_NO_PAD
                .decode(&decoded[separator + 1..])
                .map_err(|_| ProxyAuthorizationError::Malformed)?,
        );
        if self.authenticates(proof.as_slice()) {
            Ok(())
        } else {
            Err(ProxyAuthorizationError::Refused)
        }
    }
}
