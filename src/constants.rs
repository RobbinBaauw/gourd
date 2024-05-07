use anstyle::Style;

/// `e_machine` id for a x86_64 executable.
pub const X86_64_E_MACHINE: u16 = 62;

/// The styling for the program name.
pub const PRIMARY_STYLE: Style = anstyle::Style::new()
    .bold()
    .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green)));

/// The styling for the secondary text.
pub const SECONDARY_STYLE: Style =
    anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::BrightGreen)));

/// The styling for the university name.
pub const UNDERLINE_STYLE: Style = anstyle::Style::new().bold();
