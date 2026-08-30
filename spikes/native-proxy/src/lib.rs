// SPDX-License-Identifier: Apache-2.0

pub mod baseline;
pub mod candidate;
pub mod evidence;
pub mod scenario;

use evidence::{BackendRun, Comparison};

pub async fn run_candidate() -> BackendRun {
    candidate::run().await
}

pub async fn run_baseline() -> BackendRun {
    baseline::run().await
}

pub async fn run_comparison() -> Comparison {
    let candidate = run_candidate().await;
    let baseline = run_baseline().await;
    Comparison::new(candidate, baseline)
}
