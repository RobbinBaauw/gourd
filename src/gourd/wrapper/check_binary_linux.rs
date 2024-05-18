#![cfg(target_os = "linux")]

use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use elf::abi;
use elf::endian::AnyEndian;
use elf::ElfBytes;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::file_system::read_bytes;

/// A mapping from architecture string to a ELF `e_machine` field.
const E_MACHINE_MAPPING: for<'a> fn(&'a str) -> u16 = |machine| match machine {
    "x86" => abi::EM_IA_64,
    "x86_64" => abi::EM_X86_64,
    "arm" => abi::EM_ARM,
    "aarch64" => abi::EM_AARCH64,
    "mips" => abi::EM_MIPS,
    "mips64" => abi::EM_MIPS_X,
    "powerpc" => abi::EM_PPC,
    "powerpc64" => abi::EM_PPC64,
    "riscv64" => abi::EM_RISCV,
    "s390x" => abi::EM_S390,
    "sparc64" => abi::EM_SPARC,
    _ => 0,
};

pub fn verify_arch(binary: &PathBuf, expected_arch: &str) -> anyhow::Result<()> {
    let expected_machine_type = E_MACHINE_MAPPING(expected_arch);
    let elf = read_bytes(binary)?;

    let elf = ElfBytes::<AnyEndian>::minimal_parse(elf.as_slice()).with_context(ctx!(
      "Could not parse the file as ELF {binary:?}", ;
      "Are you sure this file is executable and you are using linux?",
    ))?;

    if elf.ehdr.e_machine != expected_machine_type {
        Err(anyhow!(
            "The program architecture {} does not match the expected architecture {}",
            elf.ehdr.e_machine,
            expected_arch
        ))
        .with_context(ctx!(
          "The architecture does not match for program {binary:?}", ;
          "Ensure that the program is compiled for the correct target",
        ))
    } else {
        Ok(())
    }
}
