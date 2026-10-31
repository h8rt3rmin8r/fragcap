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
    Issuer, KeyPair, KeyUsagePurpose, SigningKey,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::sign::CertifiedKey;
use zeroize::Zeroizing;

pub struct SessionCertificateAuthority {
    generation: u64,
    not_before: SystemTime,
    not_after: SystemTime,
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
        let not_before = now - Duration::from_secs(300);
        let not_after = now + lifetime;
        params.not_before = not_before.into();
        params.not_after = not_after.into();
        let mut name = DistinguishedName::new();
        name.push(
            DnType::CommonName,
            format!("fragcap Deep Capture CA {generation}"),
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
            not_before,
            not_after,
            key,
            sha1_thumbprint: digest_hex(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, der.as_ref()),
            sha256_fingerprint: digest_hex(&ring::digest::SHA256, der.as_ref()),
            der,
        })
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn not_before(&self) -> SystemTime {
        self.not_before
    }
    pub fn not_after(&self) -> SystemTime {
        self.not_after
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

    /// Reload an exact installation root and signer. The public key, CA flag,
    /// validity, and key usage are checked before the signer can issue leaves.
    pub fn load_installation(
        generation: u64,
        der: &[u8],
        private_der: &[u8],
    ) -> Result<Self, CertificateError> {
        let (remaining, certificate) =
            x509_parser::parse_x509_certificate(der).map_err(|error| {
                CertificateError::with_detail("installation-ca-invalid", error.to_string())
            })?;
        if !remaining.is_empty()
            || !certificate.is_ca()
            || certificate.subject() != certificate.issuer()
            || certificate.verify_signature(None).is_err()
        {
            return Err(CertificateError::new("installation-ca-invalid"));
        }
        let key = Zeroizing::new(KeyPair::try_from(private_der).map_err(|error| {
            CertificateError::with_detail("installation-key-invalid", error.to_string())
        })?);
        if key.public_key_raw() != certificate.public_key().subject_public_key.data.as_ref() {
            return Err(CertificateError::new("installation-key-mismatch"));
        }
        let challenge = ring::digest::digest(&ring::digest::SHA256, der);
        let signature = key.sign(challenge.as_ref()).map_err(|error| {
            CertificateError::with_detail("installation-sign-failed", error.to_string())
        })?;
        ring::signature::UnparsedPublicKey::new(
            &ring::signature::ECDSA_P256_SHA256_ASN1,
            key.public_key_raw(),
        )
        .verify(challenge.as_ref(), &signature)
        .map_err(|_| CertificateError::new("installation-sign-verify-failed"))?;
        let not_before = SystemTime::from(certificate.validity().not_before.to_datetime());
        let not_after = SystemTime::from(certificate.validity().not_after.to_datetime());
        if not_after <= SystemTime::now() || not_before > SystemTime::now() {
            return Err(CertificateError::new("installation-ca-expired"));
        }
        let der = CertificateDer::from(der.to_vec());
        Issuer::from_ca_cert_der(&der, &*key).map_err(|error| {
            CertificateError::with_detail("installation-ca-invalid", error.to_string())
        })?;
        Ok(Self {
            generation,
            not_before,
            not_after,
            key,
            sha1_thumbprint: digest_hex(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, der.as_ref()),
            sha256_fingerprint: digest_hex(&ring::digest::SHA256, der.as_ref()),
            der,
        })
    }

    /// Persist installation signing material only through the machine-scoped
    /// protected writer. Callers never receive plaintext key bytes.
    #[cfg(windows)]
    pub fn persist_installation_key(&self, path: &Path) -> Result<(), CertificateError> {
        let der = Zeroizing::new(self.key.serialize_der());
        crate::windows::acl::protect_and_write_machine(path, der.as_slice())
    }

    #[cfg(windows)]
    pub fn prepare_installation_directories(
        root: &Path,
        pki: &Path,
    ) -> Result<(), CertificateError> {
        if pki.parent() != Some(root) {
            return Err(CertificateError::new("installation-pki-path-invalid"));
        }
        crate::windows::acl::create_machine_public_directory(root)?;
        crate::windows::acl::create_machine_directory(pki)
    }

    /// Read and decrypt a protected installation signer after exact ACL checks.
    #[cfg(windows)]
    pub fn load_protected_installation(
        generation: u64,
        certificate_der: &[u8],
        path: &Path,
    ) -> Result<Self, CertificateError> {
        let private = crate::windows::acl::read_and_unprotect_machine(path)?;
        Self::load_installation(generation, certificate_der, &private)
    }

    /// Create a new CA private-key file with platform-owned encryption and private
    /// access control at creation. Existing files and aliases are never overwritten.
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
    peak_entries: usize,
    peak_bytes: usize,
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
            peak_entries: 0,
            peak_bytes: 0,
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
        self.peak_entries = self.peak_entries.max(self.entries.len());
        self.peak_bytes = self.peak_bytes.max(self.bytes);
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
    pub fn peak_entries(&self) -> usize {
        self.peak_entries
    }
    pub fn peak_bytes(&self) -> usize {
        self.peak_bytes
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
    params.not_after = (now + lifetime).min(authority.not_after).into();
    params.is_ca = IsCa::ExplicitNoCa;
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    params.use_authority_key_identifier_extension = true;
    let issuer = Issuer::from_ca_cert_der(&authority.der, &*authority.key).map_err(|error| {
        CertificateError::with_detail("installation-issuer-invalid", error.to_string())
    })?;
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

#[cfg(test)]
mod installation_tests {
    use super::*;

    #[test]
    fn reload_preserves_exact_root_and_leaf_issuer() {
        let now = SystemTime::now();
        let original =
            SessionCertificateAuthority::generate(1, now, Duration::from_secs(365 * 24 * 60 * 60))
                .unwrap();
        let private = Zeroizing::new(original.key.serialize_der());
        let restored =
            SessionCertificateAuthority::load_installation(1, original.der().as_ref(), &private)
                .unwrap();
        assert_eq!(original.der().as_ref(), restored.der().as_ref());
        assert_eq!(original.sha256_fingerprint(), restored.sha256_fingerprint());
        let leaf = issue_leaf(
            &restored,
            CertificateIdentity::parse("example.test").unwrap(),
            now,
            Duration::from_secs(60),
        )
        .unwrap();
        assert_eq!(leaf.generation, 1);
        // Separate HTTP CONNECT and QUIC/H3 runtime instances reload the same
        // installation signer. Neither instance creates a replacement root.
        let second =
            SessionCertificateAuthority::load_installation(1, original.der().as_ref(), &private)
                .unwrap();
        let h3_leaf = issue_leaf(
            &second,
            CertificateIdentity::parse("h3.example.test").unwrap(),
            now,
            Duration::from_secs(60),
        )
        .unwrap();
        let (_, http_certificate) = x509_parser::parse_x509_certificate(leaf.der.as_ref()).unwrap();
        let (_, h3_certificate) =
            x509_parser::parse_x509_certificate(h3_leaf.der.as_ref()).unwrap();
        let (_, root_certificate) =
            x509_parser::parse_x509_certificate(original.der().as_ref()).unwrap();
        assert_eq!(http_certificate.issuer(), root_certificate.subject());
        assert_eq!(h3_certificate.issuer(), root_certificate.subject());
        assert_eq!(second.sha256_fingerprint(), restored.sha256_fingerprint());
    }

    #[test]
    fn wrong_signer_is_refused() {
        let now = SystemTime::now();
        let original =
            SessionCertificateAuthority::generate(1, now, Duration::from_secs(3600)).unwrap();
        let other =
            SessionCertificateAuthority::generate(2, now, Duration::from_secs(3600)).unwrap();
        let private = Zeroizing::new(other.key.serialize_der());
        assert_eq!(
            SessionCertificateAuthority::load_installation(1, original.der().as_ref(), &private)
                .unwrap_err()
                .code,
            "installation-key-mismatch"
        );
    }

    #[test]
    fn modified_root_signature_is_refused_before_leaf_issuance() {
        let now = SystemTime::now();
        let original =
            SessionCertificateAuthority::generate(3, now, Duration::from_secs(3600)).unwrap();
        let private = Zeroizing::new(original.key.serialize_der());
        let mut der = original.der().as_ref().to_vec();
        let last = der.last_mut().unwrap();
        *last ^= 1;
        assert_eq!(
            SessionCertificateAuthority::load_installation(3, &der, &private)
                .unwrap_err()
                .code,
            "installation-ca-invalid"
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateError {
    pub code: &'static str,
    pub detail: String,
}
impl CertificateError {
    pub(crate) fn new(code: &'static str) -> Self {
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
