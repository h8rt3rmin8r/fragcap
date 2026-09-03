// SPDX-License-Identifier: Apache-2.0

//! Versioned Deep Capture bundle manifest parsing and validation.

use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

pub const MANIFEST_VERSION: u64 = 2;
pub const MANIFEST_SCHEMA: &str = "https://fragcap.dev/schema/deep-capture-manifest.v2.json";
pub const MANIFEST_PREFIX: &str = "manifest.prefix.json";

/// Stable artifact-omission reason owned by manifest version 2.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestOmissionReason {
    NotRequested,
    NotProduced,
    NotObservable,
    UnsupportedProtocol,
    NotRouted,
    NotReached,
    CertificatePinned,
    ClientAuthRequired,
    UnsupportedVersion,
    ParserFailed,
    Truncated,
    BackendUnavailable,
    NoHttpSemantics,
    ProjectionFailed,
    WriterFailed,
    SharingExcludedSensitiveArtifact,
}

impl ManifestOmissionReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotRequested => "not-requested",
            Self::NotProduced => "not-produced",
            Self::NotObservable => "not-observable",
            Self::UnsupportedProtocol => "unsupported-protocol",
            Self::NotRouted => "not-routed",
            Self::NotReached => "not-reached",
            Self::CertificatePinned => "certificate-pinned",
            Self::ClientAuthRequired => "client-auth-required",
            Self::UnsupportedVersion => "unsupported-version",
            Self::ParserFailed => "parser-failed",
            Self::Truncated => "truncated",
            Self::BackendUnavailable => "backend-unavailable",
            Self::NoHttpSemantics => "no-http-semantics",
            Self::ProjectionFailed => "projection-failed",
            Self::WriterFailed => "writer-failed",
            Self::SharingExcludedSensitiveArtifact => "sharing-excludes-sensitive-artifact",
        }
    }

    pub fn severity(self) -> &'static str {
        match self {
            Self::NotRequested
            | Self::NotObservable
            | Self::NoHttpSemantics
            | Self::SharingExcludedSensitiveArtifact => "info",
            Self::NotProduced
            | Self::UnsupportedProtocol
            | Self::NotRouted
            | Self::NotReached
            | Self::CertificatePinned
            | Self::ClientAuthRequired
            | Self::UnsupportedVersion
            | Self::Truncated
            | Self::BackendUnavailable => "warn",
            Self::ParserFailed | Self::ProjectionFailed | Self::WriterFailed => "error",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "not-requested" => Self::NotRequested,
            "not-produced" => Self::NotProduced,
            "not-observable" => Self::NotObservable,
            "unsupported-protocol" => Self::UnsupportedProtocol,
            "not-routed" => Self::NotRouted,
            "not-reached" => Self::NotReached,
            "certificate-pinned" => Self::CertificatePinned,
            "client-auth-required" => Self::ClientAuthRequired,
            "unsupported-version" => Self::UnsupportedVersion,
            "parser-failed" => Self::ParserFailed,
            "truncated" => Self::Truncated,
            "backend-unavailable" => Self::BackendUnavailable,
            "no-http-semantics" => Self::NoHttpSemantics,
            "projection-failed" => Self::ProjectionFailed,
            "writer-failed" => Self::WriterFailed,
            "sharing-excludes-sensitive-artifact" => Self::SharingExcludedSensitiveArtifact,
            _ => return None,
        })
    }
}

/// Construct the manifest omission index entry from one typed reason.
pub fn manifest_omission(role: &str, reason: ManifestOmissionReason) -> Value {
    serde_json::json!({
        "role": role,
        "reason": reason.as_str(),
        "severity": reason.severity(),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestVersion {
    LegacyV1,
    NativeV2,
}

#[derive(Clone, Debug)]
pub struct ManifestDocument {
    version: ManifestVersion,
    value: Value,
}

impl ManifestDocument {
    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        let value: Value = serde_json::from_slice(bytes).map_err(io::Error::other)?;
        let version = match value.get("manifest_version").and_then(Value::as_u64) {
            Some(1) => ManifestVersion::LegacyV1,
            Some(MANIFEST_VERSION) => {
                validate_v2(&value)?;
                ManifestVersion::NativeV2
            }
            Some(other) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unsupported manifest version {other}"),
                ));
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "manifest version missing",
                ))
            }
        };
        Ok(Self { version, value })
    }

    pub fn read(path: &Path) -> io::Result<Self> {
        Self::parse(&fs::read(path)?)
    }

    pub fn version(&self) -> ManifestVersion {
        self.version
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

pub fn validate_relative_path(value: &str) -> io::Result<PathBuf> {
    if value.is_empty()
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || value
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsafe manifest path",
        ));
    }
    let path = PathBuf::from(value);
    if path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsafe manifest path",
        ));
    }
    Ok(path)
}

pub fn validate_v2(value: &Value) -> io::Result<()> {
    let object = value
        .as_object()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "manifest must be an object"))?;
    let product = object.get("product").and_then(Value::as_object);
    if object.get("manifest_version").and_then(Value::as_u64) != Some(MANIFEST_VERSION)
        || object.get("$schema").and_then(Value::as_str) != Some(MANIFEST_SCHEMA)
        || product
            .and_then(|value| value.get("name"))
            .and_then(Value::as_str)
            != Some("fragcap")
        || product
            .and_then(|value| value.get("version"))
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        || object
            .get("session_id")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        || object.get("artifacts").and_then(Value::as_array).is_none()
        || object.get("omissions").and_then(Value::as_array).is_none()
        || !matches!(
            object.get("state").and_then(Value::as_str),
            Some("complete" | "partial" | "failed" | "interrupted" | "crash-prefix")
        )
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid version 2 manifest",
        ));
    }
    let product = product.expect("validated product object");
    if !has_only_keys(product, &["name", "version"]) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "product contains an unknown field",
        ));
    }
    let mut roles = BTreeSet::new();
    let mut omitted_roles = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let session_complete = object.get("state").and_then(Value::as_str) == Some("complete");
    for artifact in object["artifacts"].as_array().expect("checked above") {
        let artifact = artifact.as_object().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "artifact must be an object")
        })?;
        let role = artifact
            .get("role")
            .and_then(Value::as_str)
            .filter(|role| !role.is_empty())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "artifact role missing"))?;
        if !has_only_keys(
            artifact,
            &[
                "role",
                "path",
                "omission_reason",
                "authority",
                "sensitivity",
                "content_type",
                "required",
                "finalization",
                "completeness",
                "loss",
                "correlation",
            ],
        ) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "artifact contains an unknown field",
            ));
        }
        if !roles.insert(role.to_string()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "duplicate artifact role",
            ));
        }
        for required in [
            "authority",
            "content_type",
            "sensitivity",
            "finalization",
            "completeness",
            "loss",
            "correlation",
        ] {
            if !artifact.contains_key(required) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("artifact {role} lacks {required}"),
                ));
            }
        }
        let authority = artifact
            .get("authority")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "artifact authority invalid")
            })?;
        if !has_only_keys(authority, &["kind", "owner", "source_role"])
            || !matches!(
                authority.get("kind").and_then(Value::as_str),
                Some(
                    "primary-evidence"
                        | "derived-projection"
                        | "bundle-index"
                        | "analyzer-aid"
                        | "operational-record"
                )
            )
            || authority
                .get("owner")
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            || authority
                .get("source_role")
                .is_some_and(|value| !value.is_null() && value.as_str().is_none_or(str::is_empty))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "artifact authority contract incomplete",
            ));
        }
        let authority_kind = authority.get("kind").and_then(Value::as_str);
        let source_role = authority.get("source_role").and_then(Value::as_str);
        if (authority_kind == Some("derived-projection")) != source_role.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "derived authority source contract invalid",
            ));
        }
        let completeness = artifact.get("completeness").and_then(Value::as_str);
        let finalization = artifact.get("finalization").and_then(Value::as_str);
        let required = artifact.get("required").and_then(Value::as_bool) == Some(true);
        if artifact.get("required").and_then(Value::as_bool).is_none()
            || artifact
                .get("content_type")
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            || !artifact.get("loss").is_some_and(Value::is_object)
            || !artifact.get("correlation").is_some_and(Value::is_object)
            || artifact
                .get("omission_reason")
                .is_some_and(|value| value.as_str().is_none_or(str::is_empty))
            || !matches!(
                artifact.get("sensitivity").and_then(Value::as_str),
                Some("ordinary" | "sensitive" | "secret-adjacent")
            )
            || !matches!(
                completeness,
                Some(
                    "complete"
                        | "partial"
                        | "truncated"
                        | "failed"
                        | "omitted"
                        | "pending"
                        | "unavailable"
                )
            )
            || !matches!(finalization, Some("complete" | "incomplete" | "failed"))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "artifact state contract invalid",
            ));
        }
        let path = artifact.get("path").and_then(Value::as_str);
        if artifact
            .get("path")
            .is_some_and(|value| value.as_str().is_none())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "artifact path has invalid type",
            ));
        }
        if completeness == Some("omitted") {
            omitted_roles.insert(role.to_string());
            if path.is_some()
                || artifact
                    .get("omission_reason")
                    .and_then(Value::as_str)
                    .and_then(ManifestOmissionReason::parse)
                    .is_none()
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid artifact omission",
                ));
            }
        } else if path.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "produced artifact path missing",
            ));
        }
        if completeness == Some("complete") && finalization != Some("complete") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "complete artifact was not finalized",
            ));
        }
        if session_complete && required && completeness != Some("complete") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "complete session has incomplete required artifact",
            ));
        }
        if let Some(path) = path {
            validate_relative_path(path)?;
            if !paths.insert(path.to_ascii_lowercase()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "duplicate artifact path",
                ));
            }
        }
    }
    let mut declared_omissions = BTreeSet::new();
    for omission in object["omissions"].as_array().expect("checked above") {
        let omission = omission.as_object().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "omission must be an object")
        })?;
        let role = omission
            .get("role")
            .and_then(Value::as_str)
            .filter(|role| !role.is_empty())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "omission role missing"))?;
        let reason = omission
            .get("reason")
            .and_then(Value::as_str)
            .and_then(ManifestOmissionReason::parse);
        if !has_only_keys(omission, &["role", "reason", "severity"])
            || reason.is_none()
            || !matches!(
                omission.get("severity").and_then(Value::as_str),
                Some("info" | "warn" | "error")
            )
            || reason.is_some_and(|reason| {
                omission.get("severity").and_then(Value::as_str) != Some(reason.severity())
            })
            || !declared_omissions.insert(role.to_string())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid or duplicate omission",
            ));
        }
    }
    if declared_omissions != omitted_roles {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "artifact omissions and omission index disagree",
        ));
    }
    Ok(())
}

fn has_only_keys(object: &serde_json::Map<String, Value>, allowed: &[&str]) -> bool {
    object.keys().all(|key| allowed.contains(&key.as_str()))
}

pub fn write_crash_prefix(bundle: &Path, session_id: &str) -> io::Result<()> {
    let value = serde_json::json!({
        "$schema": MANIFEST_SCHEMA,
        "manifest_version": MANIFEST_VERSION,
        "product": {"name":"fragcap","version":env!("CARGO_PKG_VERSION")},
        "session_id": session_id,
        "state": "crash-prefix",
        "artifacts": [],
        "omissions": [],
    });
    publish_new(
        &bundle.join(MANIFEST_PREFIX),
        &serde_json::to_vec_pretty(&value).map_err(io::Error::other)?,
    )
}

pub fn publish_final(bundle: &Path, bytes: &[u8]) -> io::Result<()> {
    if ManifestDocument::parse(bytes)?.version() != ManifestVersion::NativeV2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "new bundles require manifest version 2",
        ));
    }
    publish_new(&bundle.join("manifest.json"), bytes)?;
    match fs::remove_file(bundle.join(MANIFEST_PREFIX)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn publish_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temporary = path.with_extension("json.tmp");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::hard_link(&temporary, path)?;
        let _ = fs::remove_file(&temporary);
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn legacy_is_read_without_rewrite() {
        let bytes = br#"{"manifest_version":1,"artifacts":[]}"#;
        let before = bytes.to_vec();
        let parsed = ManifestDocument::parse(bytes).unwrap();
        assert_eq!(parsed.version(), ManifestVersion::LegacyV1);
        assert_eq!(bytes, before.as_slice());
        assert_eq!(parsed.value()["manifest_version"], 1);
    }

    #[test]
    fn omission_reason_owns_stable_text_and_severity() {
        let parser = manifest_omission("application-jsonl", ManifestOmissionReason::ParserFailed);
        assert_eq!(parser["reason"], "parser-failed");
        assert_eq!(parser["severity"], "error");
        let unrouted = manifest_omission("application-jsonl", ManifestOmissionReason::NotRouted);
        assert_eq!(unrouted["reason"], "not-routed");
        assert_eq!(unrouted["severity"], "warn");
        let unrequested = manifest_omission("har", ManifestOmissionReason::NotRequested);
        assert_eq!(unrequested["severity"], "info");
    }

    #[test]
    fn unsafe_and_alias_paths_are_rejected() {
        for path in ["", "../x", "a/./b", "a\\b", "/root"] {
            assert!(validate_relative_path(path).is_err(), "{path}");
        }
        let artifact = |role: &str, path: &str| {
            json!({
                "role": role, "path": path, "authority": {"kind":"primary-evidence","owner":"test"}, "content_type": "application/json",
                "sensitivity": "ordinary", "finalization": "complete", "completeness": "complete",
                "loss": {"state":"none"}, "correlation": {"state":"not-applicable"}
            })
        };
        let value = json!({"$schema":MANIFEST_SCHEMA,"manifest_version":2,"product":{"name":"fragcap","version":"test"},"state":"partial","artifacts":[artifact("a","X"),artifact("b","x")],"omissions":[]});
        assert!(validate_v2(&value).is_err());
    }

    #[test]
    fn published_schema_is_byte_identical_to_embedded_schema() {
        assert_eq!(
            include_bytes!("../../assets/deep-capture-manifest.v2.schema.json"),
            include_bytes!("../../../../docs/schema/deep-capture-manifest.v2.json")
        );
    }

    #[test]
    fn published_examples_match_the_versioned_reader() {
        for bytes in [
            include_bytes!(
                "../../../../docs/schema/examples/deep-capture-manifest-v2-complete.json"
            )
            .as_slice(),
            include_bytes!(
                "../../../../docs/schema/examples/deep-capture-manifest-v2-partial.json"
            )
            .as_slice(),
            include_bytes!(
                "../../../../docs/schema/examples/deep-capture-manifest-v2-crash-prefix.json"
            )
            .as_slice(),
        ] {
            assert_eq!(
                ManifestDocument::parse(bytes).unwrap().version(),
                ManifestVersion::NativeV2
            );
        }
        assert_eq!(
            ManifestDocument::parse(include_bytes!(
                "../../../../docs/schema/examples/deep-capture-manifest-v1.json"
            ))
            .unwrap()
            .version(),
            ManifestVersion::LegacyV1
        );
    }

    #[test]
    fn crash_prefix_is_replaced_only_by_a_valid_final_manifest() {
        let directory = tempfile::tempdir().unwrap();
        write_crash_prefix(directory.path(), "session").unwrap();
        assert!(directory.path().join(MANIFEST_PREFIX).is_file());
        assert!(publish_final(directory.path(), br#"{"manifest_version":9}"#).is_err());
        assert!(directory.path().join(MANIFEST_PREFIX).is_file());
        assert!(publish_final(
            directory.path(),
            include_bytes!("../../../../docs/schema/examples/deep-capture-manifest-v1.json")
        )
        .is_err());
        assert!(directory.path().join(MANIFEST_PREFIX).is_file());
        let final_value = json!({
            "$schema": MANIFEST_SCHEMA,
            "manifest_version": 2,
            "product": {"name":"fragcap","version":"test"},
            "session_id":"session",
            "state":"complete",
            "artifacts":[],
            "omissions":[]
        });
        publish_final(
            directory.path(),
            &serde_json::to_vec_pretty(&final_value).unwrap(),
        )
        .unwrap();
        assert!(directory.path().join("manifest.json").is_file());
        assert!(!directory.path().join(MANIFEST_PREFIX).exists());
    }

    #[test]
    fn version_two_requires_a_nonempty_session_identifier() {
        let mut value: Value = serde_json::from_slice(include_bytes!(
            "../../../../docs/schema/examples/deep-capture-manifest-v2-complete.json"
        ))
        .unwrap();
        value.as_object_mut().unwrap().remove("session_id");
        assert!(ManifestDocument::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        value["session_id"] = Value::String(String::new());
        assert!(ManifestDocument::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn version_two_enforces_artifact_field_types_and_enums() {
        let original: Value = serde_json::from_slice(include_bytes!(
            "../../../../docs/schema/examples/deep-capture-manifest-v2-complete.json"
        ))
        .unwrap();
        for (field, invalid) in [
            ("loss", Value::Null),
            ("correlation", Value::String("unknown".to_string())),
            ("content_type", Value::String(String::new())),
            ("sensitivity", Value::String("private".to_string())),
        ] {
            let mut value = original.clone();
            value["artifacts"][0][field] = invalid;
            assert!(
                ManifestDocument::parse(&serde_json::to_vec(&value).unwrap()).is_err(),
                "{field} must match the published schema"
            );
        }
        let mut value = original;
        value["artifacts"][0]["unexpected"] = Value::Bool(true);
        assert!(ManifestDocument::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn version_two_preserves_unavailable_artifact_completeness() {
        let mut value: Value = serde_json::from_slice(include_bytes!(
            "../../../../docs/schema/examples/deep-capture-manifest-v2-complete.json"
        ))
        .unwrap();
        value["state"] = Value::String("partial".to_string());
        value["artifacts"][0]["completeness"] = Value::String("unavailable".to_string());

        let document = ManifestDocument::parse(&serde_json::to_vec(&value).unwrap()).unwrap();

        assert_eq!(
            document.value()["artifacts"][0]["completeness"],
            "unavailable"
        );
    }
}
