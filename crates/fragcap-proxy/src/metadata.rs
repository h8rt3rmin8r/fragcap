// SPDX-License-Identifier: Apache-2.0

//! Protocol-faithful HTTP metadata values.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolVersion {
    Http11,
    Http2,
    Http3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataKind {
    Request,
    InformationalResponse,
    Response,
    Trailers,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataOrdering {
    Wire,
    DecodedPerName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataField {
    pub name: Vec<u8>,
    pub value: Vec<u8>,
    pub original_index: u32,
    pub sensitive: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedMetadataValue {
    pub name: Vec<u8>,
    pub value: Vec<u8>,
    pub source_field_index: Option<u32>,
    pub source_component: &'static str,
    pub decode_valid: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataBlock {
    pub kind: MetadataKind,
    pub version: ProtocolVersion,
    pub ordering: MetadataOrdering,
    pub pseudo_fields: Vec<MetadataField>,
    pub fields: Vec<MetadataField>,
    pub unavailable: Vec<&'static str>,
    pub method: Option<Vec<u8>>,
    pub target: Option<Vec<u8>>,
    pub url: Option<Vec<u8>>,
    pub status: Option<u16>,
    pub reason: Option<Vec<u8>>,
    pub head_bytes: Option<u64>,
    pub query: Vec<DerivedMetadataValue>,
    pub cookies: Vec<DerivedMetadataValue>,
}

impl MetadataBlock {
    pub fn http1(kind: MetadataKind, fields: &[(String, Vec<u8>)]) -> Self {
        let fields: Vec<_> = fields
            .iter()
            .enumerate()
            .map(|(index, (name, value))| MetadataField {
                name: name.as_bytes().to_vec(),
                value: value.clone(),
                original_index: index.try_into().unwrap_or(u32::MAX),
                sensitive: sensitive_name(name.as_bytes()),
            })
            .collect();
        let cookies = cookie_pairs_from_fields(&fields);
        Self {
            kind,
            version: ProtocolVersion::Http11,
            ordering: MetadataOrdering::Wire,
            pseudo_fields: Vec::new(),
            fields,
            unavailable: Vec::new(),
            method: None,
            target: None,
            url: None,
            status: None,
            reason: None,
            head_bytes: None,
            query: Vec::new(),
            cookies,
        }
    }

    pub fn with_http1_request(mut self, method: &str, target: &str, url: &str) -> Self {
        self.method = Some(method.as_bytes().to_vec());
        self.target = Some(target.as_bytes().to_vec());
        self.url = Some(url.as_bytes().to_vec());
        self.query = query_pairs_bytes(target.as_bytes());
        self
    }

    pub fn with_http1_response(mut self, status: u16, reason: Option<&str>) -> Self {
        self.status = Some(status);
        self.reason = reason.map(|value| value.as_bytes().to_vec());
        self
    }

    pub fn with_head_bytes(mut self, bytes: usize) -> Self {
        self.head_bytes = Some(bytes.try_into().unwrap_or(u64::MAX));
        self
    }

    pub fn http2(
        kind: MetadataKind,
        pseudo_fields: Vec<MetadataField>,
        fields: Vec<MetadataField>,
    ) -> Self {
        let query = pseudo_fields
            .iter()
            .find(|field| field.name == b":path")
            .map_or_else(Vec::new, |field| query_pairs_bytes(&field.value));
        let cookies = cookie_pairs_from_fields(&fields);
        let url = absolute_http2_url(&pseudo_fields);
        let pseudo = |name: &[u8]| {
            pseudo_fields
                .iter()
                .find(|field| field.name == name)
                .map(|field| field.value.clone())
        };
        let method = pseudo(b":method");
        let target = pseudo(b":path");
        let status =
            pseudo(b":status").and_then(|value| std::str::from_utf8(&value).ok()?.parse().ok());
        Self {
            kind,
            version: ProtocolVersion::Http2,
            ordering: MetadataOrdering::DecodedPerName,
            pseudo_fields,
            fields,
            unavailable: vec!["hpack-wire-bytes", "compressed-cross-name-order"],
            method,
            target,
            url,
            status,
            reason: None,
            head_bytes: None,
            query,
            cookies,
        }
    }

    pub fn http3(
        kind: MetadataKind,
        pseudo_fields: Vec<MetadataField>,
        fields: Vec<MetadataField>,
    ) -> Self {
        let mut value = Self::http2(kind, pseudo_fields, fields);
        value.version = ProtocolVersion::Http3;
        value.unavailable = vec!["qpack-wire-bytes", "compressed-cross-name-order"];
        value
    }
}

fn absolute_http2_url(fields: &[MetadataField]) -> Option<Vec<u8>> {
    let value = |name: &[u8]| {
        fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.value.as_slice())
    };
    let (scheme, authority, path) = (value(b":scheme")?, value(b":authority")?, value(b":path")?);
    let mut url = Vec::with_capacity(scheme.len() + authority.len() + path.len() + 3);
    url.extend_from_slice(scheme);
    url.extend_from_slice(b"://");
    url.extend_from_slice(authority);
    url.extend_from_slice(path);
    Some(url)
}

pub fn fields_from_header_map(headers: &hyper::HeaderMap) -> Vec<MetadataField> {
    headers
        .iter()
        .enumerate()
        .map(|(index, (name, value))| MetadataField {
            name: name.as_str().as_bytes().to_vec(),
            value: value.as_bytes().to_vec(),
            original_index: index.try_into().unwrap_or(u32::MAX),
            sensitive: sensitive_name(name.as_str().as_bytes()),
        })
        .collect()
}

pub fn query_pairs(uri: &hyper::Uri) -> Vec<(Vec<u8>, Vec<u8>, bool)> {
    uri.query()
        .unwrap_or_default()
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (name, value) = part.split_once('=').unwrap_or((part, ""));
            let valid = percent_encoding_is_valid(name) && percent_encoding_is_valid(value);
            (name.as_bytes().to_vec(), value.as_bytes().to_vec(), valid)
        })
        .collect()
}

fn query_pairs_bytes(target: &[u8]) -> Vec<DerivedMetadataValue> {
    let Some(query) = target.splitn(2, |byte| *byte == b'?').nth(1) else {
        return Vec::new();
    };
    query
        .split(|byte| *byte == b'&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let split = part
                .iter()
                .position(|byte| *byte == b'=')
                .unwrap_or(part.len());
            let name = part[..split].to_vec();
            let value = if split < part.len() {
                part[split + 1..].to_vec()
            } else {
                Vec::new()
            };
            let decode_valid =
                percent_encoding_is_valid_bytes(&name) && percent_encoding_is_valid_bytes(&value);
            DerivedMetadataValue {
                name,
                value,
                source_field_index: None,
                source_component: "request-target",
                decode_valid,
            }
        })
        .collect()
}

pub fn cookie_pairs(headers: &hyper::HeaderMap) -> Vec<(Vec<u8>, Vec<u8>, bool)> {
    headers
        .get_all(hyper::header::COOKIE)
        .iter()
        .flat_map(|header| header.as_bytes().split(|byte| *byte == b';'))
        .map(|part| part.trim_ascii())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let Some(separator) = part.iter().position(|byte| *byte == b'=') else {
                return (part.to_vec(), Vec::new(), false);
            };
            let name = part[..separator].trim_ascii().to_vec();
            let value = part[separator + 1..].trim_ascii().to_vec();
            let valid = !name.is_empty();
            (name, value, valid)
        })
        .collect()
}

fn cookie_pairs_from_fields(fields: &[MetadataField]) -> Vec<DerivedMetadataValue> {
    fields
        .iter()
        .filter(|field| field.name.eq_ignore_ascii_case(b"cookie"))
        .flat_map(|field| {
            field
                .value
                .split(|byte| *byte == b';')
                .map(|part| (field.original_index, part.trim_ascii()))
        })
        .filter(|(_, part)| !part.is_empty())
        .map(|(source_field_index, part)| {
            let separator = part.iter().position(|byte| *byte == b'=');
            let (name, value, decode_valid) = match separator {
                Some(index) => (
                    part[..index].trim_ascii().to_vec(),
                    part[index + 1..].trim_ascii().to_vec(),
                    !part[..index].trim_ascii().is_empty(),
                ),
                None => (part.to_vec(), Vec::new(), false),
            };
            DerivedMetadataValue {
                name,
                value,
                source_field_index: Some(source_field_index),
                source_component: "field",
                decode_valid,
            }
        })
        .collect()
}

fn percent_encoding_is_valid(value: &str) -> bool {
    percent_encoding_is_valid_bytes(value.as_bytes())
}

fn percent_encoding_is_valid_bytes(bytes: &[u8]) -> bool {
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len()
                || !bytes[index + 1].is_ascii_hexdigit()
                || !bytes[index + 2].is_ascii_hexdigit()
            {
                return false;
            }
            index += 3;
        } else {
            index += 1;
        }
    }
    true
}

fn sensitive_name(name: &[u8]) -> bool {
    name.eq_ignore_ascii_case(b"authorization")
        || name.eq_ignore_ascii_case(b"proxy-authorization")
        || name.eq_ignore_ascii_case(b"cookie")
        || name.eq_ignore_ascii_case(b"set-cookie")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http1_preserves_order_case_duplicates_and_binary_values() {
        let source = vec![
            ("X-Test".to_string(), vec![0xff]),
            ("x-test".to_string(), b"two".to_vec()),
        ];
        let block = MetadataBlock::http1(MetadataKind::Request, &source);
        assert_eq!(block.ordering, MetadataOrdering::Wire);
        assert_eq!(block.fields[0].name, b"X-Test");
        assert_eq!(block.fields[0].value, vec![0xff]);
        assert_eq!(block.fields[1].name, b"x-test");
    }

    #[test]
    fn query_uncertainty_is_explicit() {
        let uri: hyper::Uri = "/?x=1&x=2&bad=%zz".parse().unwrap();
        let pairs = query_pairs(&uri);
        assert_eq!(pairs.len(), 3);
        assert!(pairs[0].2);
        assert!(!pairs[2].2);
    }

    #[test]
    fn cookies_preserve_repetition_and_invalid_members() {
        let mut headers = hyper::HeaderMap::new();
        headers.append(hyper::header::COOKIE, "a=1; a=2".parse().unwrap());
        headers.append(hyper::header::COOKIE, "flag; empty=".parse().unwrap());
        assert_eq!(
            cookie_pairs(&headers),
            vec![
                (b"a".to_vec(), b"1".to_vec(), true),
                (b"a".to_vec(), b"2".to_vec(), true),
                (b"flag".to_vec(), Vec::new(), false),
                (b"empty".to_vec(), Vec::new(), true),
            ]
        );
    }

    #[test]
    fn sensitive_fields_are_classified_without_rendering_values() {
        let block = MetadataBlock::http1(
            MetadataKind::Request,
            &[("Authorization".to_string(), b"secret".to_vec())],
        );
        assert!(block.fields[0].sensitive);
    }

    #[test]
    fn http1_lines_and_conveniences_trace_to_raw_components() {
        let block = MetadataBlock::http1(
            MetadataKind::Request,
            &[("Cookie".to_string(), b"sid=one; sid=two".to_vec())],
        )
        .with_http1_request(
            "POST",
            "/play?mode=a&mode=b&bad=%zz",
            "http://example.test/play?mode=a&mode=b&bad=%zz",
        );
        assert_eq!(block.method.as_deref(), Some(b"POST".as_slice()));
        assert_eq!(
            block.target.as_deref(),
            Some(b"/play?mode=a&mode=b&bad=%zz".as_slice())
        );
        assert_eq!(block.query.len(), 3);
        assert!(!block.query[2].decode_valid);
        assert_eq!(block.cookies.len(), 2);
        assert_eq!(block.cookies[1].source_field_index, Some(0));

        let response = MetadataBlock::http1(MetadataKind::Response, &[])
            .with_http1_response(418, Some("Teapot"));
        assert_eq!(response.status, Some(418));
        assert_eq!(response.reason.as_deref(), Some(b"Teapot".as_slice()));
    }
}
