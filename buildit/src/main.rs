use clap::Parser;
use command::*;
use coverage::CoverageCommand;
use version::VersionCommand;

#[cfg(feature = "benchmark")]
use benchmark::BenchmarkCommand;

#[cfg(feature = "benchmark")]
mod benchmark;

mod command;
mod coverage;
mod version;

pub fn main() -> std::process::ExitCode {
    env_logger::init();
    if let Err(error) = run() {
        eprintln!("❌ {error}");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}

fn run() -> anyhow::Result<()> {
    let mut command = resolve_command();
    command.run()?;
    Ok(())
}

fn resolve_command() -> Box<dyn BuildItCommand> {
    match BuildItCommandArgs::parse().command_args {
        CommandArgs::Coverage(args) => Box::new(CoverageCommand::new(args)),
        #[cfg(feature = "benchmark")]
        CommandArgs::Benchmark(args) => Box::new(BenchmarkCommand::new(args)),
        CommandArgs::Version(args) => Box::new(VersionCommand::new(args)),
    }
}
