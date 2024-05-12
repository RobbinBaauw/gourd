// *********************************************************
// Helper functions, actually interacting with the Slurm CLI
use std::process::Command;

use anyhow::anyhow;

/// Get the SLURM version from CLI output.
pub fn get_version() -> anyhow::Result<[u64; 2]> {
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

/// Get available partitions on the cluster.
/// returns a (space and newline delimited) table of partition name and availability.
pub fn get_partitions() -> anyhow::Result<Vec<Vec<String>>> {
    let s_info_out = Command::new("sinfo")
        .arg("-o")
        .arg("\"%P")
        .arg("%a\"")
        .output()?;
    let partitions = String::from_utf8_lossy(&s_info_out.stdout)
        .split('\n')
        .collect::<Vec<&str>>()
        .iter()
        .map(|x| x.to_string())
        .map(|y| {
            y.split_whitespace()
                .collect::<Vec<&str>>()
                .iter()
                .map(|z| z.to_string())
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>();
    Ok(partitions)
}
