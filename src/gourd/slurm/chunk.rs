use std::collections::BTreeSet;
use std::collections::HashMap;

use anyhow::Result;
use gourd_lib::config::ResourceLimits;
use gourd_lib::experiment::Chunk;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::Run;

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
        resource_limit: impl Fn(&Run, &Experiment) -> Result<ResourceLimits>,
        ids: impl Iterator<Item = usize>,
    ) -> Result<Vec<Chunk>>;
}

impl Chunkable for Experiment {
    fn get_unscheduled_runs(&self) -> Result<Vec<usize>> {
        let mut unscheduled: BTreeSet<usize> = (0..self.runs.len()).collect();

        for chunk in &self.chunks {
            for chunk_run in chunk.runs.clone() {
                unscheduled.remove(&chunk_run);
            }
        }

        Ok(unscheduled.into_iter().collect())
    }

    fn create_chunks(
        &self,
        chunk_length: usize,
        num_chunks: usize,
        ids: impl Iterator<Item = usize>,
    ) -> Result<Vec<Chunk>> {
        fn new_chunk(resource_limits: &Option<ResourceLimits>) -> Chunk {
            Chunk {
                runs: Vec::new(),
                resource_limits: resource_limits.clone(),
                slurm_id: None,
            }
        }

        let mut chunks: Vec<Chunk> = vec![];
        let mut current_chunk = new_chunk(&self.resource_limits);

        for id in ids {
            debug_assert!(id < self.runs.len(), "Run ID out of range");
            if chunks.len() == num_chunks {
                break;
            }

            if current_chunk.runs.len() == chunk_length {
                chunks.push(current_chunk);
                current_chunk = new_chunk(&self.resource_limits);
            }

            if current_chunk.runs.len() < chunk_length {
                current_chunk.runs.push(id);
            }
        }

        if !current_chunk.runs.is_empty() {
            chunks.push(current_chunk);
        }

        Ok(chunks)
    }

    fn create_chunks_with_resource_limits(
        &self,
        chunk_length: usize,
        num_chunks: usize,
        resource_limit: impl Fn(&Run, &Experiment) -> Result<ResourceLimits>,
        ids: impl Iterator<Item = usize>,
    ) -> Result<Vec<Chunk>> {
        let mut chunks_map: HashMap<ResourceLimits, Chunk> = HashMap::new();
        let mut final_chunks: Vec<Chunk> = vec![];

        for id in ids {
            debug_assert!(id < self.runs.len(), "Run ID out of range");
            let run = &self.runs[id];
            let limit = resource_limit(run, self)?;
            match chunks_map.get_mut(&limit) {
                Some(chunk) => {
                    if chunk.runs.len() < chunk_length {
                        chunk.runs.push(id)
                    } else {
                        final_chunks.push(chunk.clone());
                        chunks_map.insert(
                            limit.clone(),
                            Chunk {
                                runs: vec![id],
                                resource_limits: Some(limit),
                                slurm_id: None,
                            },
                        );
                    }
                }

                None => {
                    _ = chunks_map.insert(
                        limit.clone(),
                        Chunk {
                            runs: vec![id],
                            resource_limits: Some(limit),
                            slurm_id: None,
                        },
                    )
                }
            }
        }

        for chunk in chunks_map.values() {
            final_chunks.push(chunk.clone());
        }

        // Sort in descending order of chunk size
        final_chunks.sort_by(|a, b| b.runs.len().partial_cmp(&a.runs.len()).unwrap());
        final_chunks.truncate(num_chunks);

        Ok(final_chunks)
    }
}

#[cfg(test)]
#[path = "tests/chunk.rs"]
mod tests;
