use std::cmp::Ordering;
use std::collections::HashSet;

use anyhow::Context;
use anyhow::Result;
use gourd_lib::bailc;
use gourd_lib::config::slurm::ResourceLimits;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::Run;
use log::debug;
use serde::Deserialize;
use serde::Serialize;

use crate::status::ExperimentStatus;

/// A job is scheduled and can have multiple runs
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Job(pub Vec<usize>);

/// Describes one chunk: a Slurm array of scheduled runs with common resource
/// limits. Chunks are created at runtime; a run is in one chunk iff it has
/// been scheduled.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// The runs that belong to this chunk (by RunID)
    pub runs: Vec<Job>,

    /// The resource limits of this chunk.
    ///
    /// This field is immutable.
    resource_limits: ResourceLimits,
}

impl Chunk {
    /// Get the slurm id of the chunk if it is scheduled.
    ///
    /// Returns None if it is running locally or not ran yet.
    pub fn limits(&self) -> ResourceLimits {
        self.resource_limits
    }
}

impl PartialOrd for Chunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Chunk {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.runs.len().cmp(&other.runs.len()) != Ordering::Equal {
            self.runs.len().cmp(&other.runs.len())
        } else {
            self.resource_limits.cmp(&other.resource_limits)
        }
    }
}

/// Split an [`Experiment`]'s [`Run`]s into [`Chunk`]s of common
/// [`ResourceLimits`].
pub trait Chunkable {
    /// Next available [`Chunk`]s for scheduling,
    fn next_chunks(
        &mut self,
        chunk_length: usize,
        tasks_per_run: usize,
        how_many: usize,
        status: ExperimentStatus,
    ) -> Result<Vec<Chunk>>;

    /// Add the runs to the experiment so the wrapper can find them,
    ///
    /// returns the chunk index that was just created.
    fn register_runs(&mut self, runs: &Vec<Job>) -> usize;

    /// Once a chunk has been scheduled, mark all of its runs as scheduled with
    /// their slurm ids
    fn mark_chunk_scheduled(&mut self, chunk: &Chunk, batch_id: String);

    /// Get the still pending runs of this experiment.
    fn unscheduled(&self, status: &ExperimentStatus) -> Vec<(usize, &Run)>;

    /// Get the still pending runs of this experiment.
    fn scheduled_nodep(&self) -> usize;
}

impl Chunkable for Experiment {
    fn next_chunks(
        &mut self,
        chunk_length: usize,
        runs_per_job: usize,
        how_many: usize,
        status: ExperimentStatus,
    ) -> Result<Vec<Chunk>> {
        let mut chunks = vec![];

        let runs: Vec<(usize, &Run)> = self.unscheduled(&status);

        if runs.is_empty() {
            bailc!(
                "No runs left to schedule!",;
                "All available runs have already been scheduled.",;
                "You can rerun, wait for dependent runs to complete, or start a new experiment.",
            );
        }

        let separated = runs
            .chunk_by(|a, b| a.1.limits == b.1.limits)
            .collect::<Vec<&[(usize, &Run)]>>();

        for c in separated {
            let job_chunks = c.chunks_exact(runs_per_job);
            let job_chunks_remainder = job_chunks.remainder();

            let jobs: Vec<Job> = job_chunks.map(|c| {
                Job(c.iter().map(|(i, _)| *i).collect())
            }).collect();

            for chunk_jobs in jobs.chunks(chunk_length) {
                chunks.push(Chunk {
                    runs: chunk_jobs.to_vec(),
                    resource_limits: c[0].1.limits,
                });
            }

            chunks.push(Chunk {
                runs: vec![Job(job_chunks_remainder.iter().map(|(i, _)| *i).collect())],
                resource_limits: c[0].1.limits,
            });
        }

        chunks.sort_unstable();
        chunks.reverse();
        // Decreasing order of size, such that we schedule as much as possible first

        Ok(chunks.into_iter().take(how_many).collect())
    }

    fn register_runs(&mut self, runs: &Vec<Job>) -> usize {
        self.chunks.push(runs.iter().map(|j| j.0.clone()).collect());

        debug!(
            "creating chunks: {:?}, latest = {:?}",
            self.chunks,
            &self.chunks[self.chunks.len() - 1]
        );

        self.chunks.len() - 1
    }

    fn mark_chunk_scheduled(&mut self, chunk: &Chunk, batch_id: String) {
        for (task_idx, run_ids) in chunk.runs.iter().enumerate() {
            for (run_idx, run_id) in run_ids.0.iter().enumerate() {
                // because we schedule an array by specifying the run_id(s) in a list,
                // the sub id should be == run_id.
                self.runs[*run_id].slurm_id = Some(format!("{}_{}.{}", batch_id, task_idx, run_idx));
            }
        }
    }

    fn unscheduled(&self, status: &ExperimentStatus) -> Vec<(usize, &Run)> {
        self.runs
            .iter()
            .enumerate()
            .filter(|(r_idx, r)| {
                !status[r_idx].is_scheduled()
                    && !status[r_idx].is_completed()
                    && r.slurm_id.is_none()
            })
            .filter(|(_, r)| !r.parent.is_some_and(|d| !status[&d].is_completed()))
            .collect()
    }

    fn scheduled_nodep(&self) -> usize {
        let mut set: HashSet<usize> = HashSet::new();

        for chunk in &self.chunks {
            for run in chunk {
                set.extend(run);
            }
        }

        set.len()
    }
}
