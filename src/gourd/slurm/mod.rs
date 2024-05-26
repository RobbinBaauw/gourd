use std::path::Path;

use anyhow::Result;
use gourd_lib::config::SlurmConfig;
use gourd_lib::experiment::Chunk;

/// Some checks when running on slurm to improve error handling
pub mod checks;
/// An implementation for allocating queued jobs to chunks
pub mod chunk;
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
    fn schedule_chunk(
        &self,
        slurm_config: &SlurmConfig,
        chunk: &mut Chunk,
        chunk_id: usize,
        wrapper_path: &str,
        exp_path: &Path,
    ) -> Result<()>;

    /// Check if a version of SLURM is supported by this interactor.
    fn is_version_supported(&self, v: [u64; 2]) -> bool;

    /// Get the supported versions of SLURM for this interactor.
    fn get_supported_versions(&self) -> String;
}
