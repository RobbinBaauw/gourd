use anstyle::Style;
use elf::abi;

/// A mapping from architecture string to a ELF `e_machine` field.
pub const E_MACHINE_MAPPING: for<'a> fn(&'a str) -> u16 = |machine| match machine {
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

/// The default path to the wrapper, that is, we assume `gourd-wrapper` is in $PATH.
pub const WRAPPER_DEFAULT: fn() -> String = || "gourd-wrapper".to_string();

/// The styling for the program name.
pub const PRIMARY_STYLE: Style = anstyle::Style::new()
    .bold()
    .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green)));

/// The styling for the secondary text.
pub const SECONDARY_STYLE: Style =
    anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::BrightGreen)));

/// The styling for the university name.
pub const UNDERLINE_STYLE: Style = anstyle::Style::new().bold();
