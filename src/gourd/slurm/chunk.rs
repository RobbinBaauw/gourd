use std::collections::BTreeSet;

use anyhow::Result;
use gourd_lib::config::ResourceLimits;
use gourd_lib::experiment::Chunk;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::Run;
use gourd_lib::experiment::SlurmExperiment;

use crate::slurm::checks::get_slurm_data_from_experiment;

/// A trait that applies to an Experiment and enables its constituent runs to be split into Chunks.
pub trait Chunkable {
    /// Gets Runs that have not yet been scheduled.
    ///
    /// Get a vector of `usize` IDs that correspond to the indices of `self.runs` that have not yet
    /// been scheduled on the SLURM cluster. Returns an error if this is not a SLURM experiment.
    fn get_unscheduled_runs(&self) -> Result<Vec<usize>>;

    /// Allocates the provided runs to new Chunks.
    ///
    /// Creates up to `num_chunks` Chunk objects of maximum length `chunk_length`
    /// from the provided `Run` IDs, such that each chunk contains Runs with
    /// equal resource limits (as provided by a mapping function). The IDs must
    /// be valid and should probably be retrieved using `get_unscheduled_runs`.
    fn create_chunks(
        &self,
        chunk_length: usize,
        num_chunks: usize,
        ids: impl Iterator<Item = usize>,
    ) -> Result<Vec<Chunk>>;

    /// Allocates the provided runs to new Chunks.
    ///
    /// Creates up to `num_chunks` Chunk objects of maximum length `chunk_length`
    /// from the provided `Run` IDs, such that each chunk contains Runs with
    /// equal resource limits (as provided by a mapping function). The IDs must
    /// be valid and should probably be retrieved using `get_unscheduled_runs`.
    #[allow(dead_code)]
    fn create_chunks_with_resource_limits(
        &self,
        chunk_length: usize,
        num_chunks: usize,
        resource_limit: fn(&Run) -> ResourceLimits,
        ids: impl Iterator<Item = usize>,
    ) -> Result<Vec<Chunk>>;
}

impl Chunkable for Experiment {
    fn get_unscheduled_runs(&self) -> Result<Vec<usize>> {
        let slurm = get_slurm_data_from_experiment(self)?;
        let mut unscheduled: BTreeSet<usize> = (0..self.runs.len()).collect();
        for chunk in &slurm.chunks {
            unscheduled.retain(|r| !chunk.runs.contains(r));
        }
        Ok(unscheduled.into_iter().collect())
    }

    fn create_chunks(
        &self,
        chunk_length: usize,
        num_chunks: usize,
        ids: impl Iterator<Item = usize>,
    ) -> Result<Vec<Chunk>> {
        let slurm = get_slurm_data_from_experiment(self)?;
        fn new_chunk(slurm_experiment: &SlurmExperiment, capacity: usize) -> Chunk {
            Chunk {
                runs: Vec::with_capacity(capacity),
                resource_limits: slurm_experiment.resource_limits.clone(),
            }
        }

        let mut chunks: Vec<Chunk> = vec![];
        let mut current_chunk = new_chunk(slurm, chunk_length);
        for id in ids {
            debug_assert!(id < self.runs.len(), "Run ID out of range");
            if chunks.len() == num_chunks {
                break;
            }
            if current_chunk.runs.len() == chunk_length {
                chunks.push(current_chunk);
                current_chunk = new_chunk(slurm, chunk_length);
            }
            if current_chunk.runs.len() < chunk_length {
                current_chunk.runs.push(id);
            }
        }
        Ok(chunks)
    }

    fn create_chunks_with_resource_limits(
        &self,
        chunk_length: usize,
        num_chunks: usize,
        resource_limit: fn(&Run) -> ResourceLimits,
        ids: impl Iterator<Item = usize>,
    ) -> Result<Vec<Chunk>> {
        let mut chunks: Vec<Chunk> = vec![];
        for id in ids {
            debug_assert!(id < self.runs.len(), "Run ID out of range");
            let run = &self.runs[id];
            let limit = resource_limit(run);
            match chunks.iter_mut().find(|c| c.resource_limits == limit) {
                Some(t) => {
                    if t.runs.len() < chunk_length {
                        t.runs.push(id)
                    } else {
                        chunks.push(Chunk {
                            runs: vec![id],
                            resource_limits: limit.clone(),
                        })
                    }
                }
                None => chunks.push(Chunk {
                    runs: vec![id],
                    resource_limits: limit.clone(),
                }),
            }
        }
        chunks.sort_by(|a, b| b.runs.len().partial_cmp(&a.runs.len()).unwrap());
        chunks.truncate(num_chunks);
        Ok(chunks)
    }
}

#[cfg(test)]
#[path = "tests/chunk.rs"]
mod tests;
