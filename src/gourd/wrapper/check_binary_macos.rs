#![cfg(target_os = "macos")]

use std::path::PathBuf;
use std::process::Command;

use anyhow::anyhow;
use anyhow::Context;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::file_system::read_bytes;

const OSX_ARCH_MAPPING: for<'a> fn(&'a str) -> &'static str = |machine| match machine {
    "x86" => "i386",
    "x86_64" => "x86_64",
    "aarch64" => "arm64",
    "powerpc" => "ppc",
    _ => "unsupported",
};

pub(crate) fn verify_arch(binary: &PathBuf, expected_arch: &str) -> anyhow::Result<()> {
    let _ = read_bytes(binary).context("Could not read the binary file.");

    match Command::new("lipo")
        .arg("-archs")
        .arg(binary.as_path().as_os_str())
        .output()
    {
        Ok(out) => {
            let binary_archs = String::from_utf8(out.stdout.to_ascii_lowercase()).context(
                "Could not get the output of 'lipo' when checking the binary's architecture.",
            )?;
            if !binary_archs.contains(OSX_ARCH_MAPPING(expected_arch)) {
                return Err(anyhow!(
                    "The program architecture(s) {} do not match the expected architecture {}",
                    binary_archs,
                    expected_arch
                ))
                .with_context(ctx!(
                  "The architecture does not match for program {binary:?}", ;
                  "Ensure that the program is compiled for the correct target",
                ));
            }
            Ok(())
        }
        // If 'lipo' is not present, such as on PowerPC-only OS X installations with no software
        // updates since 2005, the architecture checking is skipped.
        Err(_) => Ok(()),
    }
}
