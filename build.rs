use std::env;
use std::fs;
use std::io::Error;

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::Bash;
use clap_complete::shells::Fish;
use clap_complete::shells::PowerShell;
use clap_complete::shells::Zsh;

include!("src/gourd/cli/def.rs");

fn main() -> Result<(), Error> {
    let outdir: PathBuf = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    }
    .into();

    let target_dir = outdir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("completions/");

    let _ = fs::create_dir(&target_dir);

    let mut cmd = CLI::command();

    generate_to(Bash, &mut cmd, "gourd", &target_dir)?;
    generate_to(Fish, &mut cmd, "gourd", &target_dir)?;
    generate_to(PowerShell, &mut cmd, "gourd", &target_dir)?;
    generate_to(Zsh, &mut cmd, "gourd", &target_dir)?;

    Ok(())
}
