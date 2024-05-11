use std::process::Command;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;

use crate::constants::SLURM_VERSIONS;
use crate::error::ctx;
use crate::error::Ctx;

pub fn s_main() -> Result<()> {
    match get_version() {
        Ok(version) => {
            if !SLURM_VERSIONS.contains(&version) {
                return Err(anyhow!("SLURM Version assertion failed")).with_context(ctx!("Unsupported SLURM version: {:?}", version; "Supported versions are: {:?}", SLURM_VERSIONS.map(|x| x.iter().map(u64::to_string).collect::<Vec<String>>().join(".")).to_vec()));
            }
        }
        Err(e) => {
            return Err(anyhow!("SLURM versioning failed")).with_context(ctx!("Failed to get SLURM version: {:?}", e; "Please make sure that SLURM is installed and available in the PATH",));
        }
    }
    // version test passed
    Ok(())
}

pub fn get_version() -> Result<[u64; 2]> {
    let s_info_out = Command::new("sinfo").arg("--version").output()?;
    let version = String::from_utf8_lossy(&s_info_out.stdout)
        .to_string()
        .split_whitespace()
        .collect::<Vec<&str>>()[1]
        .split(|c: char| !c.is_numeric())
        .collect::<Vec<&str>>()
        .iter()
        .map(|x| x.parse::<u64>().unwrap())
        .collect::<Vec<u64>>();
    let mut buf = [0; 2];
    buf[0] = *version.first().ok_or(anyhow!("Invalid version received"))?;
    buf[1] = *version.get(1).ok_or(anyhow!("Invalid version received"))?;
    Ok(buf)
}
