// SPDX-License-Identifier: Apache-2.0

//! Explicit, scoped, auditable Deep Capture sessions.
//!
//! The command line is one consumer of this API. A session is prepared without
//! effects, authorized against the exact prepared plan, and then driven through
//! checked lifecycle operations. Once effects begin, [`TerminalReport`] is the
//! authority: failures are accumulated, observations are retained, and every
//! owned resource receives one bounded cleanup attempt.
//!
//! Adapters must cooperatively honor the [`Budget`] passed to blocking calls.
//! Rust cannot safely preempt an arbitrary trait method. A late successful
//! return is still classified as a deadline failure by the coordinator.

mod adapters;
mod application;
mod model;
mod native;
mod policy;
mod session;

pub use adapters::*;
pub use application::*;
pub use model::*;
pub use native::{
    run_controlled_native_requests, CertificateStore, NativeCertificateStore,
    NativeObservationContext, NativeProxyAdapter, NativeProxyLimits, TrustController, TrustError,
    TrustMutation, TrustState, CURRENT_USER_ROOT, LOCAL_MACHINE_ROOT,
};
pub use policy::{
    calibration_outcome, calibration_outcome_reason, compatibility_fact_candidates,
    compatibility_owner_role, observation_is_correlated_to_final_client,
    observation_proves_final_client_ca_acceptance, terminal_calibration_outcome,
    validate_compatibility_prerequisites,
};
pub use session::{DeepCapture, DeepCaptureSession, PreparedSession};
