#![forbid(unsafe_code)]

use clap::{ColorChoice, Parser};
use cli::cli_command::CliCommand;
use cli::environment::EnvServiceTrait;
use cli::view_command::ViewCommand;
use log::error;
use logging::configure_logger;
use space_rs::SizeDisplayFormat;
#[cfg(not(test))]
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, Arc};

#[cfg(test)]
#[path = "./main_test.rs"]
mod main_test;

#[cfg(test)]
mod test_directory_utils;

#[cfg(test)]
mod test_utils;

mod cli;
mod logging;

const DEFAULT_SIZE_THRESHOLD_PERCENTAGE: u8 = 1;

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
    $ space path/to/dir1,'path/to/dir 2'
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
    $ space path/to/dir1,'path/to/dir 2'

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
    /// will be used. Separate multiple paths using commas.
    #[arg(value_name = "TARGET PATH(S)", value_parser, num_args = 1.., value_delimiter = ',')]
    target_paths: Option<Vec<PathBuf>>,

    /// The size threshold as a percentage of the total. Only items with a relative size greater or equal
    /// to this percentage will be displayed.
    #[arg(value_name = "PERCENTAGE", short = 's', long, default_value_t = DEFAULT_SIZE_THRESHOLD_PERCENTAGE, value_parser = clap::value_parser!(u8).range(0..=100))]
    size_threshold_percentage: u8,

    /// The format to use when a size value is displayed.
    #[arg(short = 'f', long, value_enum, default_value_t = SizeDisplayFormat::Metric)]
    size_format: SizeDisplayFormat,

    /// If specified then only non-interactive output will be rendered.
    #[arg(short = 'n', long)]
    non_interactive: bool,
}

#[cfg(not(test))]
pub fn main() -> anyhow::Result<()> {
    use std::sync::atomic::Ordering;

    use cli::environment::DefaultEnvService;

    let should_exit = Arc::new(AtomicBool::new(false));
    let s = should_exit.clone();
    ctrlc::set_handler(move || {
        s.store(true, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl-C handler");

    run(
        &env::args().collect::<Vec<_>>(),
        &mut std::io::stdout(),
        dirs::home_dir(),
        Box::<DefaultEnvService>::default(),
        should_exit,
    )?;

    Ok(())
}

pub(crate) fn run<W: Write>(
    args: &[String],
    writer: &mut W,
    user_home_dir: Option<PathBuf>,
    env_service: Box<dyn EnvServiceTrait>,
    should_exit: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    configure_logger(user_home_dir, &env_service);
    if let Err(e) = run_command(args, writer, env_service, should_exit) {
        error!("{}", e);
        eprintln!("{}", e);
        Err(e)
    } else {
        Ok(())
    }
}

fn run_command<W: Write>(
    args: &[String],
    writer: &mut W,
    env_service: Box<dyn EnvServiceTrait>,
    should_exit: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    let args = parse_args(args)?;
    prepare_command(args, env_service, should_exit)?.run(writer)?;
    Ok(())
}

fn parse_args(args: &[String]) -> Result<CliArgs, anyhow::Error> {
    Ok(CliArgs::parse_from(args))
}

fn prepare_command(
    args: CliArgs,
    env_service: Box<dyn EnvServiceTrait>,
    should_exit: Arc<AtomicBool>,
) -> anyhow::Result<ViewCommand> {
    let mut command = ViewCommand::new(
        args.target_paths,
        Some(args.size_format),
        args.size_threshold_percentage,
        #[cfg(not(test))]
        args.non_interactive,
        env_service,
        should_exit,
    );
    command.prepare()?;
    Ok(command)
}
