// SPDX-License-Identifier: Apache-2.0

use crate::ProtocolVersion;

pub(crate) fn negotiated_protocol(alpn: Option<&[u8]>) -> Result<ProtocolVersion, &'static str> {
    match alpn {
        Some(b"h2") => Ok(ProtocolVersion::Http2),
        Some(b"http/1.1") | None => Ok(ProtocolVersion::Http11),
        Some(_) => Err("tls-alpn-mismatch"),
    }
}
