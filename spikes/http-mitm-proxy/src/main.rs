// SPDX-License-Identifier: Apache-2.0

use fragcap_http_mitm_proxy_spike::run_candidate;
use std::{env, fs::File, io::BufWriter, path::PathBuf};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "candidate".to_string());
    if command != "candidate" {
        return Err(format!("unknown command: {command}").into());
    }
    let mut output = None;
    while let Some(argument) = args.next() {
        if argument == "--output" {
            output = args.next().map(PathBuf::from);
        }
    }
    let run = run_candidate().await;
    match output {
        Some(path) => serde_json::to_writer_pretty(BufWriter::new(File::create(path)?), &run)?,
        None => serde_json::to_writer_pretty(std::io::stdout().lock(), &run)?,
    }
    Ok(())
}
