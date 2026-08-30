// SPDX-License-Identifier: Apache-2.0

use fragcap_native_proxy_spike::{run_baseline, run_candidate, run_comparison};

#[tokio::main]
async fn main() {
    let command = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "compare".to_string());
    let value = match command.as_str() {
        "candidate" => serde_json::to_value(run_candidate().await),
        "baseline" => serde_json::to_value(run_baseline().await),
        "compare" => serde_json::to_value(run_comparison().await),
        _ => {
            eprintln!("usage: fragcap-native-proxy-spike [candidate|baseline|compare]");
            std::process::exit(2);
        }
    };

    match value.and_then(|value| serde_json::to_string_pretty(&value)) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("cannot serialize authoritative evidence: {error}");
            std::process::exit(1);
        }
    }
}
