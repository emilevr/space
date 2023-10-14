use anyhow::bail;
use clap::Parser;
use cli::cli_command::CliCommand;
use cli::view_command::ViewCommand;
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

#[derive(Debug, Parser)]
#[command(name = "space")]
#[command(author, version, propagate_version = true)]
#[command(about = r#"Space, the final frontier!

When run without any arguments 'space' will calculate the size of the current directory tree and display the results in the terminal.
Use the '--gui' option to show the results in a desktop graphical user interface instead.
\\//_"#, long_about = None)]
struct CliArgs {
    #[command(subcommand)]
    command: CommandArgs,
}

pub(crate) const DEFAULT_SIZE_THRESHOLD_PERCENTAGE: u8 = 1;

#[derive(Debug, Parser)]
enum CommandArgs {
    /// Shows the apparent disk usage of the specified directory tree(s). This is the default command.
    #[command()]
    View {
        /// The path(s) to the target files or directories to view. If not supplied the current directory
        /// will be used. Separate multiple paths using spaces.
        #[arg(value_name = "TARGET PATH(S)", value_parser, num_args = 1.., value_delimiter = ' ')]
        target_paths: Option<Vec<PathBuf>>,

        /// The size threshold as a percentage of the total. Only items with an apparent size greater or equal
        /// to this percentage will be included.
        #[arg(value_name = "PERCENTAGE", short = 's', long, default_value_t = DEFAULT_SIZE_THRESHOLD_PERCENTAGE, value_parser = clap::value_parser!(u8).range(0..=100))]
        size_threshold_percentage: u8,

        /// If specified then only non-interactive output will be rendered.
        #[arg(short = 'n', long)]
        non_interactive: bool,
    },
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
    match parse_args(args)?.command {
        CommandArgs::View {
            target_paths,
            size_threshold_percentage,
            non_interactive,
        } => Ok(ViewCommand::new(
            target_paths,
            None,
            size_threshold_percentage,
            non_interactive,
        )),
    }
}

fn parse_args(args: Vec<String>) -> Result<CliArgs, anyhow::Error> {
    Ok(CliArgs::parse_from(ensure_default_command(args)?))
}

fn ensure_default_command(mut args: Vec<String>) -> Result<Vec<String>, anyhow::Error> {
    match args.len() {
        // No arguments supplied so default to the View command.
        1 => args.push(ViewCommand::NAME.to_string()),

        // Some argument specified. Check if any of the supported commands were specified as the first
        // argument. If not, then insert it.
        2.. => match args[1].as_str() {
            ViewCommand::NAME => {}
            _ => args.insert(1, ViewCommand::NAME.to_string()),
        },

        _ => bail!("Arguments are invalid!"),
    }

    Ok(args)
}
