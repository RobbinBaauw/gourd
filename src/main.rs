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

/// Common file operations
pub mod file_system;

/// The local runner module: `gourd run local`.
pub mod local;

/// The SLURM runner module: `gourd run slurm`.
pub mod slurm;

/// Code shared between the wrapper and `gourd`.
pub mod measurement;

/// Accessing and managing resources.
pub mod resources;

/// The command line interface and relevant structures.
pub mod cli;

/// The status of the jobs running or finished.
pub mod status;

/// The setup of an experiment.
pub mod experiment;

/// The main entrypoint.
///
/// This function is the main entrypoint of the program.
#[cfg(not(tarpaulin_include))]
fn main() {
    cli::process::parse_command();
}
