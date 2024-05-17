use anyhow::anyhow;
use anyhow::Context;
use gourd_lib::config::Config;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;

use crate::cli::printing::format_table;
use crate::slurm::handler::SlurmHandler;
use crate::slurm::SlurmConfig;
use crate::slurm::SlurmInteractor;

/// Check the config that it has the necessary fields
pub fn get_slurm_options_from_config(config: &Config) -> anyhow::Result<&SlurmConfig> {
    config.slurm.as_ref()
        .ok_or_else(|| anyhow!("No SLURM configuration found"))
        .with_context(ctx!(
              "Tried to execute on Slurm but the configuration field for the Slurm options in gourd.toml was empty", ;
              "Make sure that your gourd.toml includes the required fields under [slurm]",
            ))
}

impl<T> SlurmHandler<T>
where
    T: SlurmInteractor,
{
    /// Check if the SLURM version is supported.
    pub(crate) fn check_version(&self) -> anyhow::Result<()>
    where
        T: SlurmInteractor,
    {
        match self.internal.get_version() {
            Ok(version) => {
                if !self.internal.is_version_supported(version) {
                    Err(anyhow!("SLURM Version assertion failed")).with_context(
                        ctx!("Unsupported SLURM version: {:?}",
                          version.iter().map(u64::to_string).collect::<Vec<String>>().join(".");
                          "Supported versions are: {}",
                          self.internal.get_supported_versions()
                        ),
                    )
                } else {
                    Ok(())
                }
            }

            Err(e) => Err(anyhow!("SLURM versioning failed")).with_context(ctx!(
              "Failed to get SLURM version: {}", e;
              "Please make sure that SLURM is installed and available in the PATH",
            )),
        }
    }

    /// Check if the provided partition is valid.
    pub fn check_partition(&self, partition: &str) -> anyhow::Result<()>
    where
        T: SlurmInteractor,
    {
        let partitions = self.internal.get_partitions()?;
        if partitions.iter().map(|x| x.first()).any(|x| {
            if let Some(y) = x {
                y == partition
            } else {
                false
            }
        }) {
            Ok(())
        } else {
            Err(anyhow!("Invalid partition provided")).with_context(ctx!(
              "Partition `{}` is not available on this cluster. ", partition;
              "Present partitions are:\n{}", format_table(partitions),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::*;
    #[test]
    fn versioning_test() {
        struct X {}
        impl SlurmInteractor for X {
            fn get_version(&self) -> anyhow::Result<[u64; 2]> {
                Ok([21, 8])
            }
            fn get_partitions(&self) -> anyhow::Result<Vec<Vec<String>>> {
                Ok(vec![])
            }

            fn schedule_array(
                &self,
                _range: Range<usize>,
                _slurm_config: &SlurmConfig,
                _wrapper_path: &str,
            ) -> anyhow::Result<()> {
                Ok(())
            }

            fn is_version_supported(&self, _v: [u64; 2]) -> bool {
                true
            }

            fn get_supported_versions(&self) -> String {
                "groff".to_string()
            }
        }
        let y = SlurmHandler { internal: X {} };
        assert!(y.check_version().is_ok());
    }

    #[test]
    fn versioning_un_test() {
        struct X {}
        impl SlurmInteractor for X {
            fn get_version(&self) -> anyhow::Result<[u64; 2]> {
                Ok([21, 8])
            }
            fn get_partitions(&self) -> anyhow::Result<Vec<Vec<String>>> {
                Ok(vec![])
            }

            fn schedule_array(
                &self,
                _range: Range<usize>,
                _slurm_config: &SlurmConfig,
                _wrapper_path: &str,
            ) -> anyhow::Result<()> {
                Ok(())
            }

            fn is_version_supported(&self, _v: [u64; 2]) -> bool {
                false
            }

            fn get_supported_versions(&self) -> String {
                "your dad".to_string()
            }
        }
        let y = SlurmHandler { internal: X {} };
        assert!(y.check_version().is_err());
    }

    #[test]
    fn get_slurm_options_from_config_test() {
        let config = Config {
            slurm: Some(SlurmConfig {
                partition: "test".to_string(),
                time_limit: "1:00:00".to_string(),
                cpus: 1,
                mem_per_cpu: 420,
                out: None,
                experiment_name: "test".to_string(),
            }),
            ..Default::default()
        };
        assert!(get_slurm_options_from_config(&config).is_ok());
    }

    #[test]
    fn get_slurm_options_from_config_un_test() {
        let config = Config {
            slurm: None,
            ..Default::default()
        };
        assert!(get_slurm_options_from_config(&config).is_err());
    }
}
