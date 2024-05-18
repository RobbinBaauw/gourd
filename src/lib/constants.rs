use std::convert::Into;
use std::path::PathBuf;

use anstyle::AnsiColor;
use anstyle::Color;
use anstyle::Style;

/// The default path to the wrapper, that is, we assume `gourd_wrapper` is in $PATH.
pub const WRAPPER_DEFAULT: fn() -> String = || "gourd_wrapper".to_string();

/// The default path to the afterscript.
pub const AFTERSCRIPT_DEFAULT: fn() -> PathBuf = || "".into();

/// The default path to the afterscript output.
pub const AFTERSCRIPT_OUTPUT_DEFAULT: fn() -> PathBuf = || "after".into();

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

/// Supported SLURM versions.
pub const SLURM_VERSIONS: [[u64; 2]; 1] = [[21, 8]];
