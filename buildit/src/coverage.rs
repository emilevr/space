use clap::{Args, ValueEnum};
use std::{
    env,
    fmt::Display,
    fs::{self, create_dir_all},
};
use xtaskops::ops::*;

use crate::command::BuildItCommand;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CoverageReportType {
    /// The Cobertura output format. Used to publish code coverage to GitHub.
    Cobertura,
    /// An HTML report. For local use during development.
    Html,
    /// An lcov format report that is used by the Coverage Gutters VSCode extension and may also be uploaded
    /// to codecov.io
    Lcov,
}

impl Display for CoverageReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoverageReportType::Cobertura => write!(f, "cobertura"),
            CoverageReportType::Html => write!(f, "html"),
            CoverageReportType::Lcov => write!(f, "lcov"),
        }
    }
}

#[derive(Args, Debug)]
pub struct CoverageCommandArgs {
    /// The coverage report types to generate.
    #[arg(value_enum, short = 'o', long = "output")]
    pub output_types: Option<Vec<CoverageReportType>>,

    /// An optional package to cover.
    #[arg(long = "package")]
    pub package: Option<String>,

    /// Optionally run only the ignored tests
    #[arg(long = "ignored", default_value_t = false)]
    pub ignored: bool,

    /// Optionally include the ignored tests
    #[arg(long = "include-ignored", default_value_t = false)]
    pub include_ignored: bool,
}

pub struct CoverageCommand {
    output_types: Option<Vec<CoverageReportType>>,
    package: Option<String>,
    ignored: bool,
    include_ignored: bool,
}

impl CoverageCommand {
    pub fn new(args: CoverageCommandArgs) -> Self {
        CoverageCommand {
            output_types: args.output_types,
            package: args.package,
            ignored: args.ignored,
            include_ignored: args.include_ignored,
        }
    }
}

impl BuildItCommand for CoverageCommand {
    fn run(&mut self) -> anyhow::Result<()> {
        let mut output_path = env::current_dir()?;
        output_path.push("coverage");

        if output_path.exists() {
            println!("👷 Deleting old reports ...");
            fs::remove_dir_all(&output_path)?;
        }

        println!("👷 Creating coverage directory ...");
        create_dir_all(&output_path)?;

        let target_coverage_dir = "./target_coverage";
        println!("👷 Setting target dir to {}", target_coverage_dir);
        env::set_var("CARGO_TARGET_DIR", target_coverage_dir);

        println!("👷 Running Tests with Code Coverage ...");
        let mut args = vec!["test"];
        if let Some(package) = &self.package {
            args.push("--package");
            args.push(package);
        }
        // Now the test params, i.e. after a '--' in the params.
        if self.include_ignored || self.ignored {
            args.push("--");
            if self.ignored {
                args.push("--ignored");
            } else if self.include_ignored {
                args.push("--include-ignored");
            }
        }
        cmd("cargo", args)
            .env("RUSTFLAGS", "-Cinstrument-coverage")
            .env("LLVM_PROFILE_FILE", "cargo-test-%p-%m.profraw")
            .run()?;
        println!("✔️ Testing and Code Coverage completed successfully.");

        println!("👷 Generating Reports ...");
        let output_types = match &self.output_types {
            Some(output_types) => output_types
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(","),
            _ => "html,lcov".to_string(),
        };

        // Call grcov, which has to be available on the path.
        cmd!(
            "grcov",
            ".",
            "--binary-path",
            format!("{}/debug/deps", target_coverage_dir),
            "-s",
            ".",
            "-t",
            output_types,
            "--branch",
            "--excl-line",
            // Exlude the following lines:
            //  ^\\s*(debug_)?assert(_eq|_ne)?!                                             => debug_assert and assert variants
            //  ^\\s*#\\[cfg\\s*\\(.*\\]\\s*$                                               => cfg attributes
            //  ^\\s*#\\[inline\\s*\\(.*\\]\\s*$                                            => inline attributes
            //  ^\\s*#\\[derive\\s*\\(.*\\]\\s*$                                            => derive attributes
            //  ^\\s*\\}\\s*else\\s*\\{\\s*$                                                => lines containing only "} else {"
            //  ^\\s*//.*$                                                                  => lines containing only a comment
            //  ^\\s*(pub|pub\\s*\\(\\s*crate\\s*\\)\\s*)?\\s*fn\\s+.*\\s*[(){}]*\\s*$      => lines containing only a fn definition
            //  ^\\s*(pub|pub\\s*\\(\\s*crate\\s*\\)\\s*)?\\s*[^ ]+\\s*:\\s*[^ ]+\\s*,\\s*$ => lines containing only a property declaration
            //  ^\\s*loop\\s*\\{\\s*$                                                       => lines containing only 'loop {'
            //  ^\\s*[^ ]+\\s*=>\\s*(\\{)?\\s*$                                             => lines only containing the expression part of a match clause
            //  ^\\s*\\)\\s*->\\s*[^ ]+\\s*\\{\\s*$                                         => lines only ') -> some_return_type {'
            //  ^\\s*[{}(),;\\[\\] ]*\\s*$                                                  => lines with only delimiters or whitespace, no logic
            //  ^\\s*impl\\s+.*\\s+for\\s+.*\\s+\\{\\s*$                                    => lines containing only an impl declaration
            "^\\s*(debug_)?assert(_eq|_ne)?!\
                |^\\s*#\\[cfg\\s*\\(.*\\]\\s*$\
                |^\\s*#\\[inline\\s*\\(.*\\]\\s*$\
                |^\\s*#\\[derive\\s*\\(.*\\]\\s*$\
                |^\\s*\\}\\s*else\\s*\\{\\s*$\
                |^\\s*//.*$\
                |^\\s*(pub|pub\\s*\\(\\s*crate\\s*\\)\\s*)?\\s*fn\\s+.*\\s*[(){}]*\\s*$\
                |^\\s*(pub|pub\\s*\\(\\s*crate\\s*\\)\\s*)?\\s*[^ ]+\\s*:\\s*[^ ]+\\s*,\\s*$\
                |^\\s*loop\\s*\\{\\s*$\
                |^\\s*[^ ]+\\s*=>\\s*(\\{)?\\s*$\
                |^\\s*\\)\\s*->\\s*[^ ]+\\s*\\{\\s*$\
                |^\\s[});({]*\\s*$\
                |^\\s*[{}(),;\\[\\] ]*\\s*$\
                |^\\s*impl\\s+.*\\s+for\\s+.*\\s+\\{\\s*$",
            "--ignore-not-existing",
            "--ignore",
            "buildit/*",
            "--ignore",
            "src/tests/*",
            "--ignore",
            "src/benches/*",
            "--ignore",
            "**/*_test.rs",
            "--ignore",
            "**/test_*.rs",
            "--ignore",
            "**/*_test_*.rs",
            "--ignore",
            "**/.cargo/registry/*",
            "-o",
            &output_path,
        )
        .run()?;
        output_path.push("html");
        output_path.push("index.html");
        println!(
            "✔️ Code Coverage report was created successfully: file:///{}",
            output_path.display()
        );

        println!("👷 Cleaning Up ...");
        clean_files("**/*.profraw")?;
        println!("✔️ Cleanup completed.");

        Ok(())
    }
}
