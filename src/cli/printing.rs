use anstyle::AnsiColor;
use clap::crate_authors;
use clap::crate_name;
use clap::crate_version;

use crate::constants::style_from_fg;
use crate::constants::ERROR_STYLE;
use crate::constants::HELP_STYLE;
use crate::constants::PRIMARY_STYLE;
use crate::constants::SECONDARY_STYLE;
use crate::constants::UNIVERSITY_STYLE;

/// Util function for getting the style for the CLI
pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(style_from_fg(AnsiColor::Yellow).bold())
        .header(style_from_fg(AnsiColor::Green).bold().underline())
        .literal(style_from_fg(AnsiColor::Cyan).bold())
        .invalid(style_from_fg(AnsiColor::Blue).bold())
        .error(ERROR_STYLE)
        .valid(HELP_STYLE)
        .placeholder(style_from_fg(AnsiColor::White))
}

/// Pretty print gourd's version
pub fn print_version() {
    println!(
        "{}{}{:#} at version {}{}{:#}\n\n",
        PRIMARY_STYLE,
        crate_name!(),
        PRIMARY_STYLE,
        SECONDARY_STYLE,
        crate_version!(),
        SECONDARY_STYLE
    );

    println!(
        "{}Technische Universiteit Delft 2024{:#}\n",
        UNIVERSITY_STYLE, UNIVERSITY_STYLE,
    );

    println!("Authored by:\n{}", crate_authors!("\n"));
}

/// Util function: formatting a table for printing
///
/// input: Vec of rows, each row is a Vec of strings (columns)
///
/// output: String
pub fn format_table(data: Vec<Vec<String>>) -> String {
    if data.is_empty() {
        return String::new();
    }
    let mut max_widths = vec![0; data[0].len()];
    for row in &data {
        for (i, item) in row.iter().enumerate() {
            max_widths[i] = max_widths[i].max(item.len());
        }
    }
    let mut result = String::new();
    for row in data {
        let formatted_row: Vec<String> = row
            .into_iter()
            .enumerate()
            .map(|(i, item)| format!("{:width$}", item, width = max_widths[i]))
            .collect();
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&formatted_row.join(" | "));
    }
    result
}
