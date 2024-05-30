use std::path::PathBuf;
use std::time::Duration;

use anstyle::AnsiColor;
use anstyle::Color;
use anstyle::Style;

use crate::config::ProgramMap;
use crate::config::ResourceLimits;

/// The default path to the wrapper, that is, we assume `gourd_wrapper` is in
/// $PATH.
pub const WRAPPER_DEFAULT: fn() -> String = || "gourd_wrapper".to_string();

/// The default path to the afterscript.
pub const AFTERSCRIPT_DEFAULT: fn() -> Option<PathBuf> = || None;

/// The default path to the output of an afterscript.
pub const AFTERSCRIPT_OUTPUT_DEFAULT: fn() -> Option<PathBuf> = || None;

/// The default postprocess job name.
pub const POSTPROCESS_JOB_DEFAULT: fn() -> Option<String> = || None;

/// The default path to the output of a postprocess job.
pub const POSTPROCESS_JOB_OUTPUT_DEFAULT: fn() -> Option<PathBuf> = || None;

/// The default list of postprocess programs.
pub const POSTPROCESS_JOBS_DEFAULT: fn() -> Option<ProgramMap> = || None;

/// The default value of resource limits for a program.
pub const PROGRAM_RESOURCES_DEFAULT: fn() -> Option<ResourceLimits> = || None;

/// The default arguments for an input.
pub const EMPTY_ARGS: fn() -> Vec<String> = Vec::new;

/// The prefix which will cause an argument to be interpreted as a glob.
pub const GLOB_ESCAPE: &str = "glob|";

/// The internal representation of globbed inputs.
pub const INTERNAL_HATCH: &str = "_hatch_";

/// The internal representation of globbed inputs.
pub const INTERNAL_GLOB: &str = "_glob_";

// /// The internal representation of postprocess runs.
pub const INTERNAL_POST: &str = "_postprocess_";

/// Internal representation for names parsed from config
pub const INTERNAL_PREFIX: &str = "_internal_";

/// The amount between refreshes of the status screen, in ms.
pub const STATUS_REFRESH_PERIOD: Duration = Duration::from_millis(50);

/// Create a style with a defined foreground color.
pub const fn style_from_fg(color: AnsiColor) -> Style {
    Style::new().fg_color(Some(Color::Ansi(color)))
}

/// The styling for the program name.
pub const PRIMARY_STYLE: Style = style_from_fg(AnsiColor::Green).bold();

/// The styling for the secondary text.
pub const SECONDARY_STYLE: Style = style_from_fg(AnsiColor::BrightGreen);

/// The styling for the university name.
pub const NAME_STYLE: Style = Style::new().bold();

/// The styling for error messages.
pub const ERROR_STYLE: Style = style_from_fg(AnsiColor::Red).bold().blink();

/// The styling for help messages.
pub const HELP_STYLE: Style = style_from_fg(AnsiColor::Green).bold().underline();

/// Supported SLURM versions.
pub const SLURM_VERSIONS: [[u64; 2]; 1] = [[21, 8]];

/// Possible values for Mail Type in slurm configuration
pub const MAIL_TYPE_VALID_OPTIONS: [&str; 13] = [
    "NONE",
    "BEGIN",
    "END",
    "FAIL",
    "REQUEUE",
    "ALL",
    "INVALID_DEPEND",
    "STAGE_OUT",
    "TIME_LIMIT",
    "TIME_LIMIT_90",
    "TIME_LIMIT_80",
    "TIME_LIMIT_50",
    "ARRAY_TASKS",
];

/// The maximal amount of runs before status only shows a short summary.
pub const SHORTEN_STATUS_CUTOFF: usize = 100;

/// Do we assume by default that runs with custom labels are failed runs?
pub const RERUN_LABEL_BY_DEFAULT: fn() -> bool = || true;

/// The logo of the application.
pub const LOGO: &str = include_str!("../resources/logo.ascii");
