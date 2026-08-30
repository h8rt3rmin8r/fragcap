// SPDX-License-Identifier: Apache-2.0

pub mod candidate;
pub mod evidence;
pub mod scenario;

pub async fn run_candidate() -> evidence::BackendRun {
    candidate::run().await
}
