// SPDX-License-Identifier: Apache-2.0

use fragcap_http_mitm_proxy_spike::run_candidate;
use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

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
        Some(path) => {
            let mut writer = BufWriter::new(File::create(path)?);
            serde_json::to_writer_pretty(&mut writer, &run)?;
            writer.flush()?;
        }
        None => serde_json::to_writer_pretty(std::io::stdout().lock(), &run)?,
    }
    if let Some(error) = run.harness_error() {
        return Err(error.into());
    }
    Ok(())
}
