// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::net::IpAddr;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, ExtendedKeyUsagePurpose, IsCa,
    Issuer, KeyPair, KeyUsagePurpose,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::sign::CertifiedKey;
use zeroize::Zeroizing;

pub struct SessionCertificateAuthority {
    generation: u64,
    params: CertificateParams,
    key: Zeroizing<KeyPair>,
    der: CertificateDer<'static>,
    sha1_thumbprint: String,
    sha256_fingerprint: String,
}

impl fmt::Debug for SessionCertificateAuthority {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionCertificateAuthority")
            .field("generation", &self.generation)
            .field("sha1_thumbprint", &self.sha1_thumbprint)
            .field("sha256_fingerprint", &self.sha256_fingerprint)
            .finish_non_exhaustive()
    }
}

impl SessionCertificateAuthority {
    pub fn generate(
        generation: u64,
        now: SystemTime,
        lifetime: Duration,
    ) -> Result<Self, CertificateError> {
        if lifetime.is_zero() {
            return Err(CertificateError::new("zero-ca-lifetime"));
        }
        let key = Zeroizing::new(KeyPair::generate().map_err(|error| {
            CertificateError::with_detail("ca-key-generation-failed", error.to_string())
        })?);
        let mut params = CertificateParams::new(Vec::<String>::new()).map_err(|error| {
            CertificateError::with_detail("ca-params-failed", error.to_string())
        })?;
        params.not_before = (now - Duration::from_secs(300)).into();
        params.not_after = (now + lifetime).into();
        let mut name = DistinguishedName::new();
        name.push(
            DnType::CommonName,
            format!("fragcap Deep Capture session CA {generation}"),
        );
        name.push(
            DnType::OrganizationName,
            "fragcap authorized local research",
        );
        params.distinguished_name = name;
        params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
        params.key_usages = vec![KeyUsagePurpose::KeyCertSign];
        params.use_authority_key_identifier_extension = true;
        let certificate = params
            .self_signed(&*key)
            .map_err(|error| CertificateError::with_detail("ca-sign-failed", error.to_string()))?;
        let der = certificate.der().clone();
        Ok(Self {
            generation,
            params,
            key,
            sha1_thumbprint: digest_hex(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, der.as_ref()),
            sha256_fingerprint: digest_hex(&ring::digest::SHA256, der.as_ref()),
            der,
        })
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn der(&self) -> &CertificateDer<'static> {
        &self.der
    }
    pub fn sha1_thumbprint(&self) -> &str {
        &self.sha1_thumbprint
    }
    pub fn sha256_fingerprint(&self) -> &str {
        &self.sha256_fingerprint
    }

    /// Persist the CA private key with platform-owned encryption and access control.
    #[cfg(windows)]
    pub fn persist_private_key(&self, path: &Path) -> Result<(), CertificateError> {
        let der = Zeroizing::new(self.key.serialize_der());
        crate::windows::acl::protect_and_write(path, der.as_slice())
    }

    #[cfg(not(windows))]
    pub fn persist_private_key(&self, _: &Path) -> Result<(), CertificateError> {
        Err(CertificateError::new(
            "private-material-platform-unavailable",
        ))
    }
}

fn digest_hex(algorithm: &'static ring::digest::Algorithm, bytes: &[u8]) -> String {
    ring::digest::digest(algorithm, bytes)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect()
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CertificateIdentity {
    Dns(String),
    Ip(IpAddr),
}

impl CertificateIdentity {
    pub fn parse(value: &str) -> Result<Self, CertificateError> {
        if let Ok(ip) = value.parse::<IpAddr>() {
            return Ok(Self::Ip(ip));
        }
        if value.contains('*')
            || !value.is_ascii()
            || value.is_empty()
            || value.ends_with('.')
            || value.split('.').any(|part| {
                part.is_empty()
                    || part.starts_with('-')
                    || part.ends_with('-')
                    || !part
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            })
        {
            return Err(CertificateError::new("invalid-certificate-identity"));
        }
        Ok(Self::Dns(value.to_ascii_lowercase()))
    }

    fn text(&self) -> String {
        match self {
            Self::Dns(name) => name.clone(),
            Self::Ip(ip) => ip.to_string(),
        }
    }
}

pub struct LeafCertificate {
    pub identity: CertificateIdentity,
    pub generation: u64,
    pub der: CertificateDer<'static>,
    pub certified_key: Arc<CertifiedKey>,
    pub byte_cost: usize,
    issued_at: SystemTime,
}

pub struct LeafCache {
    max_entries: NonZeroUsize,
    max_bytes: NonZeroUsize,
    lifetime: Duration,
    policy_generation: u64,
    entries: HashMap<CertificateIdentity, Arc<LeafCertificate>>,
    lru: VecDeque<CertificateIdentity>,
    bytes: usize,
    evictions: u64,
}

impl LeafCache {
    pub fn new(
        max_entries: NonZeroUsize,
        max_bytes: NonZeroUsize,
        lifetime: Duration,
        policy_generation: u64,
    ) -> Result<Self, CertificateError> {
        if lifetime.is_zero() {
            return Err(CertificateError::new("zero-leaf-lifetime"));
        }
        Ok(Self {
            max_entries,
            max_bytes,
            lifetime,
            policy_generation,
            entries: HashMap::new(),
            lru: VecDeque::new(),
            bytes: 0,
            evictions: 0,
        })
    }

    pub fn certificate_for(
        &mut self,
        authority: &SessionCertificateAuthority,
        identity: CertificateIdentity,
        now: SystemTime,
    ) -> Result<Arc<LeafCertificate>, CertificateError> {
        let before = self.entries.len();
        self.entries.retain(|_, entry| {
            entry.generation == authority.generation
                && now.duration_since(entry.issued_at).unwrap_or_default() < self.lifetime
        });
        self.evictions = self
            .evictions
            .saturating_add(before.saturating_sub(self.entries.len()) as u64);
        self.recount();
        if let Some(existing) = self.entries.get(&identity).cloned() {
            self.touch(&identity);
            return Ok(existing);
        }
        let leaf = Arc::new(issue_leaf(authority, identity.clone(), now, self.lifetime)?);
        if leaf.byte_cost > self.max_bytes.get() {
            return Err(CertificateError::new("leaf-exceeds-cache-byte-limit"));
        }
        while self.entries.len() >= self.max_entries.get()
            || self.bytes + leaf.byte_cost > self.max_bytes.get()
        {
            self.evict_oldest();
        }
        self.bytes += leaf.byte_cost;
        self.lru.push_back(identity.clone());
        self.entries.insert(identity, Arc::clone(&leaf));
        Ok(leaf)
    }

    pub fn rotate_policy(&mut self, generation: u64) {
        if generation != self.policy_generation {
            self.policy_generation = generation;
            self.evictions = self.evictions.saturating_add(self.entries.len() as u64);
            self.entries.clear();
            self.lru.clear();
            self.bytes = 0;
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
    pub fn evictions(&self) -> u64 {
        self.evictions
    }
    fn touch(&mut self, identity: &CertificateIdentity) {
        self.lru.retain(|item| item != identity);
        self.lru.push_back(identity.clone());
    }
    fn evict_oldest(&mut self) {
        if let Some(identity) = self.lru.pop_front() {
            if let Some(entry) = self.entries.remove(&identity) {
                self.bytes = self.bytes.saturating_sub(entry.byte_cost);
                self.evictions = self.evictions.saturating_add(1);
            }
        }
    }
    fn recount(&mut self) {
        self.lru
            .retain(|identity| self.entries.contains_key(identity));
        self.bytes = self.entries.values().map(|entry| entry.byte_cost).sum();
    }
}

fn issue_leaf(
    authority: &SessionCertificateAuthority,
    identity: CertificateIdentity,
    now: SystemTime,
    lifetime: Duration,
) -> Result<LeafCertificate, CertificateError> {
    let leaf_key = Zeroizing::new(KeyPair::generate().map_err(|error| {
        CertificateError::with_detail("leaf-key-generation-failed", error.to_string())
    })?);
    let mut params = CertificateParams::new(vec![identity.text()])
        .map_err(|error| CertificateError::with_detail("leaf-params-failed", error.to_string()))?;
    params.not_before = (now - Duration::from_secs(300)).into();
    params.not_after = (now + lifetime)
        .min(SystemTime::from(authority.params.not_after))
        .into();
    params.is_ca = IsCa::ExplicitNoCa;
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    params.use_authority_key_identifier_extension = true;
    let issuer = Issuer::from_params(&authority.params, &*authority.key);
    let certificate = params
        .signed_by(&*leaf_key, &issuer)
        .map_err(|error| CertificateError::with_detail("leaf-sign-failed", error.to_string()))?;
    let der = certificate.der().clone();
    let key_der = Zeroizing::new(leaf_key.serialize_der());
    let private = Zeroizing::new(PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        key_der.to_vec(),
    )));
    let signer = rustls::crypto::ring::sign::any_supported_type(&private).map_err(|error| {
        CertificateError::with_detail("leaf-key-unsupported", error.to_string())
    })?;
    let byte_cost = der.len() + key_der.len();
    Ok(LeafCertificate {
        identity,
        generation: authority.generation,
        der: der.clone(),
        certified_key: Arc::new(CertifiedKey::new(vec![der], signer)),
        byte_cost,
        issued_at: now,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateError {
    pub code: &'static str,
    pub detail: String,
}
impl CertificateError {
    fn new(code: &'static str) -> Self {
        Self {
            code,
            detail: code.to_string(),
        }
    }
    fn with_detail(code: &'static str, detail: String) -> Self {
        Self { code, detail }
    }
    #[cfg(windows)]
    pub(crate) fn platform(code: &'static str, os_error: u32) -> Self {
        Self {
            code,
            detail: format!("Windows error 0x{os_error:08X}"),
        }
    }
    #[cfg(windows)]
    pub(crate) fn io(code: &'static str, error: std::io::Error) -> Self {
        Self {
            code,
            detail: error.to_string(),
        }
    }
}
impl fmt::Display for CertificateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}
impl std::error::Error for CertificateError {}
