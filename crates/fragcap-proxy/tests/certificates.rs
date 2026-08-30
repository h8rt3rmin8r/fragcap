// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

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
    assert!(parsed.subject().to_string().contains("TEST ONLY"));
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
