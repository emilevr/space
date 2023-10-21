use anyhow::bail;
use clap::{arg, ColorChoice, Parser};
use cli::{cli_command::CliCommand, view_command::ViewCommand};
use criterion::Criterion;
use space_rs::SizeDisplayFormat;
use std::{
    io::{self, Write},
    path::PathBuf,
    time::Duration,
};

#[cfg(test)]
#[path = "./space_bench_test.rs"]
mod space_bench_test;

#[cfg(test)]
mod test_directory_utils;

#[cfg(test)]
mod test_utils;

mod cli;

pub(crate) const DEFAULT_SIZE_THRESHOLD_PERCENTAGE: u8 = 1;

#[derive(Clone, Debug, Parser)]
#[clap(
    name = "space-bench",
    version,
    long_about =
r#"The time to analyze and display the specified tree(s) will be benchmarked. This is useful for project
contributors to quickly test and compare the performance impact of their changes against the previous release
version. You would typically run this in the root of the space repo. Results are stored to
./target/criterion and used for comparisons with previous runs."#,
    after_help =
r#"EXAMPLES:
    $ space-bench path/to/file/or/dir
    $ space-bench path/to/dir1 path/to/dir2
    $ space-bench --size-threshold-percentage 5"#,
    after_long_help =
r#"EXAMPLES:
    Benchmark using a single directory:
    $ space-bench path/to/dir

    Benchmark using multiple directories:
    $ space-bench path/to/dir1 path/to/dir2

    Benchmark using a single directory and with a relative size display filter of >= 5%:
    $ space-bench path/to/dir --size-threshold-percentage 5

    Benchmark using a single directory and with binary units rather than the default metric units:
    $ space-bench path/to/dir --size-format binary"#,
    color = ColorChoice::Never,
)]
struct CliArgs {
    /// The path(s) to the target files or directories to view. If not supplied the current directory
    /// will be used. Separate multiple paths using spaces.
    #[arg(value_name = "TARGET PATH(S)", value_parser, num_args = 1.., value_delimiter = ' ')]
    target_paths: Vec<PathBuf>,

    /// The size threshold as a percentage of the total. Only items with a relative size greater or equal
    /// to this percentage will be included.
    #[arg(value_name = "PERCENTAGE", short = 's', long, default_value_t = DEFAULT_SIZE_THRESHOLD_PERCENTAGE, value_parser = clap::value_parser!(u8).range(0..=100))]
    size_threshold_percentage: u8,

    /// The format to use when a size value is displayed.
    #[arg(short = 'f', long, value_enum, default_value_t = SizeDisplayFormat::Metric)]
    size_display_format: SizeDisplayFormat,

    /// The sample size for the benchmark.
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u16).range(10..=1000))]
    sample_size: u16,

    /// The warmup time in seconds.
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u16).range(1..=100))]
    warmup_seconds: u16,

    /// The measurement time in seconds.
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u16).range(1..=100))]
    measurement_seconds: u16,
}

pub struct BenchmarkCommand {
    target_paths: Vec<PathBuf>,
    size_display_format: SizeDisplayFormat,
    size_threshold_percentage: u8,
    sample_size: u16,
    warmup_seconds: u16,
    measurement_seconds: u16,
    buffer: Vec<u8>,
}

impl BenchmarkCommand {
    pub fn new(
        target_paths: Vec<PathBuf>,
        size_display_format: SizeDisplayFormat,
        size_threshold_percentage: u8,
        sample_size: u16,
        warmup_seconds: u16,
        measurement_seconds: u16,
    ) -> Self {
        BenchmarkCommand {
            target_paths,
            size_display_format,
            size_threshold_percentage,
            sample_size,
            warmup_seconds,
            measurement_seconds,
            buffer: Vec::new(),
        }
    }

    fn run(&mut self) -> anyhow::Result<()> {
        let mut c = Criterion::default()
            .significance_level(0.5)
            .sample_size(self.sample_size.into())
            .warm_up_time(Duration::from_secs(self.warmup_seconds.into()))
            .measurement_time(Duration::from_secs(self.measurement_seconds.into()));

        c.bench_function(&format!("{:?}", self.target_paths), |b| {
            b.iter(|| {
                ViewCommand::new(
                    Some(self.target_paths.clone()),
                    Some(self.size_display_format),
                    self.size_threshold_percentage,
                    true,
                )
                .prepare()
                .unwrap()
                .run(self)
                .unwrap();
            })
        });
        Ok(())
    }
}

impl Write for BenchmarkCommand {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(not(test))]
pub fn main() -> anyhow::Result<()> {
    use std::env;
    run(&env::args().collect::<Vec<_>>())?;
    Ok(())
}

fn run(args: &[String]) -> anyhow::Result<()> {
    let args = CliArgs::parse_from(args);
    if args.target_paths.is_empty() {
        bail!("Please specify at least one target path to use for benchmarking.");
    }

    BenchmarkCommand::new(
        args.target_paths,
        args.size_display_format,
        args.size_threshold_percentage,
        args.sample_size,
        args.warmup_seconds,
        args.measurement_seconds,
    )
    .run()?;

    Ok(())
}
