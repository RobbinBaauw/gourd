//! Gourd allows

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![allow(clippy::redundant_static_lifetimes)]
// for tarpaulin cfg
#![allow(unexpected_cfgs)]

/// The tests validating the behaviour of `gourd`.
#[cfg(test)]
pub mod tests;

/// The error type of `gourd`.
pub mod error;

/// A struct and related methods for global configuration,
/// declaratively specifying experiments.
pub mod config;

/// The binary wrapper around run programs.
pub mod wrapper;

/// Constant values.
pub mod constants;

/// The local runner module: `gourd run local`.
pub mod local;

/// Code shared between the wrapper and `gourd`.
pub mod measurement;

/// Accessing and managing resources.
pub mod resources;

/// The command line interface and relevant structures.
pub mod cli;

/// The main entrypoint.
///
/// This function is the main entrypoint of the program.
#[cfg(not(tarpaulin_include))]
fn main() {
    cli::parse_command();
}
