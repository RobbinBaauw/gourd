//! Gourd allows

/// An interface for communicating with `gourd-wrapper`, a separately
/// packaged binary that encapsulates the user's programs.
pub mod wrapper;

/// A framework for running experiments on the local machine using a
/// thread-pool executor.
pub mod local;

/// A framework for running supercomputer experiments by interfacing
/// with a local installation of SLURM.
pub mod slurm;

/// Functionality for user-friendly initialisation of new experimental
/// setups and examples.
pub mod init;

/// Functionality for retrieving resources (binaries and test cases)
/// from files, remote servers, and source code.
pub mod resources;

/// The command line interface and relevant structures.
#[cfg(not(tarpaulin_include))]
pub mod cli;

/// Functionality for checking and displaying the status of already
/// running experiments.
pub mod status;

/// Extensions to the `Experiment` struct defined in `gourd-lib`,
/// allowing for operations on runtime data.
pub mod experiments;

/// Functionality for post-processing jobs including after-scripts,
/// pipeline jobs, and retrieval of their status.
pub mod post;

/// Rerun subcommand helper functions
pub mod rerun;

/// Analysing runs - collecting metrics, exporting, plotting.
pub mod analyse;

/// Convenience functions for unit tests.
#[cfg(test)]
pub mod test_utils;

/// The main CLI entry-point of the `gourd` utility.
///
/// This function parses command-line arguments and executes
/// sub-commands as specified by the user.
#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() {
    cli::process::parse_command().await;
}
