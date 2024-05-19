//! Gourd allows

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![allow(clippy::redundant_static_lifetimes)]
// for tarpaulin cfg
#![allow(unexpected_cfgs)]

/// The binary wrapper around run programs.
pub mod wrapper;

/// The local runner module: `gourd run local`.
pub mod local;

/// The SLURM runner module: `gourd run slurm`.
pub mod slurm;

/// Accessing and managing resources.
pub mod resources;

/// The command line interface and relevant structures.
pub mod cli;

/// The status of the jobs running or finished.
pub mod status;

/// The implementations for the experiment struct, used by the CLI
pub mod experiments;

/// All post-processing: afterscripts, sequential jobs, collecting their statuses.
pub mod post;

#[cfg(test)]
mod test_utils;

/// The main entrypoint.
///
/// This function is the main entrypoint of the program.
#[cfg(not(tarpaulin_include))]
fn main() {
    cli::process::parse_command();
}
