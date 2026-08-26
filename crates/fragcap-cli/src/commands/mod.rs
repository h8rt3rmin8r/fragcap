// SPDX-License-Identifier: Apache-2.0

//! The command implementations.
//!
//! Each command is a function returning either the exit code it chose or a
//! [`crate::exit::CliError`] the library maps to the exit contract. A command
//! that completed and wants a non-zero code (a blocked `doctor`, an acquisition
//! timeout whose summary was printed) returns the code directly; an error means
//! "print this and stop".

pub mod capture;
pub mod catalog;
pub mod deep_capture;
pub mod doctor;
pub mod extcap;
pub mod schema;
pub mod steam;
pub mod stub;
pub mod target_resolve;
pub mod targets;
pub mod technologies;
