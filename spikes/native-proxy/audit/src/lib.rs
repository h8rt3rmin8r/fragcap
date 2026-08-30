// SPDX-License-Identifier: Apache-2.0

/// Keeps the exact candidate dependency reachable during audit builds.
pub fn candidate_type_name() -> &'static str {
    std::any::type_name::<hudsucker::Body>()
}
