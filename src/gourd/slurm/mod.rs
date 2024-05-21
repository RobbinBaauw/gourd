use std::ops::Range;
use std::path::Path;

use anyhow::Result;
use gourd_lib::config::ResourceLimits;
use gourd_lib::config::SlurmConfig;

/// Some checks when running on slurm to improve error handling
pub mod checks;
mod chunk;
/// The core slurm functionality
pub mod handler;
/// Currently used implementation of interacting with SLURM through the CLI
pub mod interactor;

/// The interface for interacting with a SLURM cluster.
/// This can be via a version-specific CLI, via a REST API, or via a library.
pub trait SlurmInteractor {
    /// Check the version of slurm on the current environment.
    /// returns an error if the version is not supported, or if slurm is not present.
    fn get_version(&self) -> Result<[u64; 2]>;
    /// Check if the provided partition is valid.
    fn get_partitions(&self) -> Result<Vec<Vec<String>>>;

    /// actually running batch jobs. still not completely decided what this will do, more documentation soon™
    fn schedule_array(
        &self,
        range: Range<usize>,
        slurm_config: &SlurmConfig,
        resource_limits: &ResourceLimits,
        wrapper_path: &str,
        exp_path: &Path,
    ) -> Result<()>;

    /// Check if a version of SLURM is supported by this interactor.
    fn is_version_supported(&self, v: [u64; 2]) -> bool;

    /// Get the supported versions of SLURM for this interactor.
    fn get_supported_versions(&self) -> String;
}
