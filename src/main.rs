use clap::{ColorChoice, Parser};
use cli::cli_command::CliCommand;
use cli::view_command::ViewCommand;
use space_rs::SizeDisplayFormat;
use std::env;
use std::io::Write;
use std::path::PathBuf;

mod cli;

#[cfg(test)]
#[path = "./main_test.rs"]
mod main_test;

#[cfg(test)]
mod test_directory_utils;
#[cfg(test)]
mod test_utils;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Parser)]
#[clap(
    name = "space",
    version,
    long_about =
r#"Space, the final frontier! 🖖

Analyzes and displays the size of one or more directory trees.

License: MIT [https://github.com/emilevr/space/blob/main/LICENSE]
Copyright © 2023 Emile van Reenen [https://github.com/emilevr]"#,
    after_help =
r#"EXAMPLES:
    $ space
    $ space path/to/file/or/dir
    $ space path/to/dir1 path/to/dir2
    $ space --size-threshold-percentage 5
    $ space --size-format binary
    $ space --non-interactive"#,
    after_long_help =
r#"EXAMPLES:
    Analyze and display current working directory in a Text User Interface (TUI):
    $ space

    Analyze and display a single directory or file:
    $ space path/to/file/or/dir

    Analyze and display multiple directories:
    $ space path/to/dir1 path/to/dir2

    Set the relative size display filter to >= 5%:
    $ space --size-threshold-percentage 5

    Display file and directory sizes using binary units rather than the default metric units:
    $ space --size-format binary

    Display non-interactive output then exit:
    $ space --non-interactive"#,
    color = ColorChoice::Never,
)]
struct CliArgs {
    /// The path(s) to the target files or directories to view. If not supplied the current directory
    /// will be used. Separate multiple paths using spaces.
    #[arg(value_name = "TARGET PATH(S)", value_parser, num_args = 1.., value_delimiter = ' ')]
    target_paths: Option<Vec<PathBuf>>,

    /// The size threshold as a percentage of the total. Only items with a relative size greater or equal
    /// to this percentage will be included.
    #[arg(value_name = "PERCENTAGE", short = 's', long, default_value_t = DEFAULT_SIZE_THRESHOLD_PERCENTAGE, value_parser = clap::value_parser!(u8).range(0..=100))]
    size_threshold_percentage: u8,

    /// The format to use when a size value is displayed.
    #[arg(short = 'f', long, value_enum, default_value_t = SizeDisplayFormat::Metric)]
    size_format: SizeDisplayFormat,

    /// If specified then only non-interactive output will be rendered.
    #[arg(short = 'n', long)]
    non_interactive: bool,
}

pub(crate) const DEFAULT_SIZE_THRESHOLD_PERCENTAGE: u8 = 1;

#[derive(Debug, Parser)]
enum CommandArgs {
    /// Shows the apparent disk space usage of the specified directory tree(s). This is the default command.
    #[command()]
    View {},
}

pub fn main() -> anyhow::Result<()> {
    env_logger::init();
    run(env::args().collect(), &mut std::io::stdout())?;
    Ok(())
}

pub(crate) fn run<W: Write>(args: Vec<String>, writer: &mut W) -> anyhow::Result<()> {
    let mut command = resolve_command(args)?;
    command.prepare()?;
    command.run(writer)?;
    Ok(())
}

fn resolve_command(args: Vec<String>) -> Result<impl CliCommand, anyhow::Error> {
    let args = parse_args(args)?;
    Ok(ViewCommand::new(
        args.target_paths,
        Some(args.size_format),
        args.size_threshold_percentage,
        args.non_interactive,
    ))
}

fn parse_args(args: Vec<String>) -> Result<CliArgs, anyhow::Error> {
    Ok(CliArgs::parse_from(args))
}
