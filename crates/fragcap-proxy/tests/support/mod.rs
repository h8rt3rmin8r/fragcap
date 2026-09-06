// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

pub fn isolated_tls_client_config() -> Arc<rustls::ClientConfig> {
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(generated.cert.der().clone()).unwrap();
    fragcap_proxy::tls_client_config_with_roots(roots).unwrap()
}
