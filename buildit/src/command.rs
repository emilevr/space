use clap::{Parser, Subcommand};

use crate::{coverage::CoverageCommandArgs, version::VersionCommandArgs};

#[cfg(feature = "benchmark")]
use crate::benchmark::BenchmarkCommandArgs;

pub trait BuildItCommand {
    fn run(&mut self) -> anyhow::Result<()>;
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct BuildItCommandArgs {
    #[command(subcommand)]
    pub command_args: CommandArgs,
}

#[derive(Debug, Subcommand)]
pub enum CommandArgs {
    /// Builds, runs, measures and creates code coverage reports.
    Coverage(CoverageCommandArgs),

    /// Runs a benchmark using a known directory structure.
    #[cfg(feature = "benchmark")]
    #[command()]
    Benchmark(BenchmarkCommandArgs),

    /// Determines the previous and next version numbers based on repository history.
    #[command()]
    Version(VersionCommandArgs),
}
