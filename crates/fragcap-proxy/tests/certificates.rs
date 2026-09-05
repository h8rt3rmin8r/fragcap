// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

#[cfg(windows)]
use std::{fs, io, path::PathBuf};

use fragcap_proxy::{CertificateIdentity, LeafCache, SessionCertificateAuthority};
use x509_parser::extensions::{GeneralName, ParsedExtension};
use x509_parser::parse_x509_certificate;

#[test]
fn ca_is_synthetic_bounded_and_fingerprinted() {
    let now = SystemTime::now();
    let ca = SessionCertificateAuthority::generate(7, now, Duration::from_secs(3600)).unwrap();
    assert_eq!(ca.sha1_thumbprint().len(), 40);
    assert_eq!(ca.sha256_fingerprint().len(), 64);
    let (_, parsed) = parse_x509_certificate(ca.der().as_ref()).unwrap();
    assert!(parsed.tbs_certificate.is_ca());
    assert!(parsed.subject().to_string().contains("Deep Capture"));
}

#[test]
fn leaves_use_exact_san_and_cache_bounds() {
    let now = SystemTime::now();
    let ca = SessionCertificateAuthority::generate(1, now, Duration::from_secs(3600)).unwrap();
    let mut cache = LeafCache::new(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(16_384).unwrap(),
        Duration::from_secs(600),
        1,
    )
    .unwrap();
    let first = cache
        .certificate_for(&ca, CertificateIdentity::parse("one.invalid").unwrap(), now)
        .unwrap();
    let (_, parsed) = parse_x509_certificate(first.der.as_ref()).unwrap();
    let san = parsed
        .extensions()
        .iter()
        .find_map(|extension| match extension.parsed_extension() {
            ParsedExtension::SubjectAlternativeName(value) => Some(value),
            _ => None,
        })
        .unwrap();
    assert!(san
        .general_names
        .iter()
        .any(|name| matches!(name, GeneralName::DNSName("one.invalid"))));
    cache
        .certificate_for(&ca, CertificateIdentity::parse("two.invalid").unwrap(), now)
        .unwrap();
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.evictions(), 1);
    assert!(cache.bytes() <= 16_384);
    cache.rotate_policy(2);
    assert!(cache.is_empty());
}

#[test]
fn malformed_leaf_identities_are_refused() {
    for bad in [
        "",
        "*.invalid",
        "é.invalid",
        "bad..invalid",
        "-bad.invalid",
        "bad.invalid.",
    ] {
        assert!(CertificateIdentity::parse(bad).is_err(), "{bad}");
    }
}

#[test]
fn concurrent_leaf_requests_share_one_bounded_entry() {
    let now = SystemTime::now();
    let ca =
        Arc::new(SessionCertificateAuthority::generate(9, now, Duration::from_secs(3600)).unwrap());
    let cache = Arc::new(Mutex::new(
        LeafCache::new(
            NonZeroUsize::new(2).unwrap(),
            NonZeroUsize::new(32_768).unwrap(),
            Duration::from_secs(600),
            1,
        )
        .unwrap(),
    ));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let ca = Arc::clone(&ca);
            let cache = Arc::clone(&cache);
            std::thread::spawn(move || {
                cache
                    .lock()
                    .unwrap()
                    .certificate_for(
                        &ca,
                        CertificateIdentity::parse("race.invalid").unwrap(),
                        now,
                    )
                    .unwrap()
            })
        })
        .collect();
    let leaves: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert!(leaves.iter().all(|leaf| Arc::ptr_eq(leaf, &leaves[0])));
    assert_eq!(cache.lock().unwrap().len(), 1);
}

#[cfg(windows)]
#[test]
fn private_material_is_dpapi_protected_in_owned_storage() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session-ca.private");
    let ca =
        SessionCertificateAuthority::generate(10, SystemTime::now(), Duration::from_secs(3600))
            .unwrap();
    ca.persist_private_key(&path).unwrap();
    let protected = std::fs::read(&path).unwrap();
    assert!(!protected.is_empty());
    assert!(!protected.windows(10).any(|value| value == b"PRIVATE KEY"));

    let impossible = directory.path().join("absent").join("key");
    assert!(ca.persist_private_key(&impossible).is_err());
    assert!(!impossible.exists());
}

#[cfg(windows)]
#[test]
#[ignore = "S129 matrix runner only"]
fn approved_current_user_trust_round_trip_restores_exact_state() {
    use fragcap_proxy::{CertificateStore, NativeCertificateStore, TrustState};

    assert_eq!(
        std::env::var("FRAGCAP_WINDOWS_PHYSICAL_EFFECTS").as_deref(),
        Ok("approved")
    );
    let ca = SessionCertificateAuthority::generate(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        SystemTime::now(),
        Duration::from_secs(300),
    )
    .unwrap();
    let store = NativeCertificateStore;
    let obligation = trust_obligation_path();
    assert_eq!(
        store
            .observe(ca.der().as_ref(), ca.sha1_thumbprint())
            .unwrap(),
        TrustState::Absent
    );
    write_trust_obligation(&obligation, ca.der().as_ref(), ca.sha1_thumbprint()).unwrap();
    store
        .add_exact(ca.der().as_ref(), ca.sha1_thumbprint())
        .unwrap();
    let observed = store.observe(ca.der().as_ref(), ca.sha1_thumbprint());
    let cleanup = store.remove_exact(ca.der().as_ref(), ca.sha1_thumbprint());
    let after = store.observe(ca.der().as_ref(), ca.sha1_thumbprint());
    if cleanup.is_ok() && matches!(after, Ok(TrustState::Absent)) {
        fs::remove_file(&obligation).unwrap();
    }
    cleanup.unwrap();
    assert_eq!(observed.unwrap(), TrustState::PresentExact);
    assert_eq!(after.unwrap(), TrustState::Absent);
}

#[cfg(windows)]
#[test]
#[ignore = "S129 matrix runner recovery only"]
fn reconcile_pending_current_user_trust() {
    use fragcap_proxy::{CertificateStore, NativeCertificateStore, TrustState};

    assert_eq!(
        std::env::var("FRAGCAP_WINDOWS_PHYSICAL_EFFECTS").as_deref(),
        Ok("approved")
    );
    let obligation = trust_obligation_path();
    if !obligation.is_file() {
        return;
    }
    let (der, thumbprint) = read_trust_obligation(&obligation).unwrap();
    let calculated = ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, &der)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(calculated, thumbprint);
    let store = NativeCertificateStore;
    store.remove_exact(&der, &thumbprint).unwrap();
    assert_eq!(
        store.observe(&der, &thumbprint).unwrap(),
        TrustState::Absent
    );
    fs::remove_file(obligation).unwrap();
}

#[cfg(windows)]
fn trust_obligation_path() -> PathBuf {
    PathBuf::from(std::env::var_os("FRAGCAP_WINDOWS_SCRATCH").expect("matrix scratch root"))
        .join("trust-cleanup.bin")
}

#[cfg(windows)]
fn write_trust_obligation(path: &std::path::Path, der: &[u8], thumbprint: &str) -> io::Result<()> {
    let mut bytes = b"fragcap-s129-trust-v1\n".to_vec();
    bytes.extend_from_slice(thumbprint.as_bytes());
    bytes.push(b'\n');
    bytes.extend_from_slice(der);
    let pending = path.with_extension("pending");
    fs::write(&pending, bytes)?;
    fs::rename(pending, path)
}

#[cfg(windows)]
fn read_trust_obligation(path: &std::path::Path) -> io::Result<(Vec<u8>, String)> {
    let bytes = fs::read(path)?;
    let first = bytes.iter().position(|byte| *byte == b'\n');
    let second = first.and_then(|index| {
        bytes[index + 1..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|next| index + 1 + next)
    });
    let (first, second) = first
        .zip(second)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid trust obligation"))?;
    if &bytes[..first] != b"fragcap-s129-trust-v1" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unknown trust obligation",
        ));
    }
    let thumbprint = std::str::from_utf8(&bytes[first + 1..second])
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        .to_owned();
    Ok((bytes[second + 1..].to_vec(), thumbprint))
}

#[cfg(windows)]
#[test]
fn trust_cleanup_obligation_round_trips_exact_identity() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("trust-cleanup.bin");
    let der = b"public certificate bytes";
    let thumbprint = "0123456789abcdef0123456789abcdef01234567";
    write_trust_obligation(&path, der, thumbprint).unwrap();
    let observed = read_trust_obligation(&path).unwrap();
    assert_eq!(observed, (der.to_vec(), thumbprint.to_owned()));
}
