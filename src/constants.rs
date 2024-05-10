use anstyle::AnsiColor;
use anstyle::Color;
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

/// Create a style with a defined foreground color.
pub const fn style_from_fg(color: AnsiColor) -> Style {
    Style::new().fg_color(Some(Color::Ansi(color)))
}

/// The styling for the program name.
pub const PRIMARY_STYLE: Style = style_from_fg(AnsiColor::Green).bold();

/// The styling for the secondary text.
pub const SECONDARY_STYLE: Style = style_from_fg(AnsiColor::BrightGreen);

/// The styling for the university name.
pub const UNIVERSITY_STYLE: Style = Style::new().bold();

/// The styling for error messages.
pub const ERROR_STYLE: Style = style_from_fg(AnsiColor::Red).bold().blink();

/// The styling for help messages.
pub const HELP_STYLE: Style = style_from_fg(AnsiColor::Green).bold().underline();
