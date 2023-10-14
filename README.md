# Space

[![Lint](https://github.com/emilevr/space/actions/workflows/lint.yaml/badge.svg)](https://github.com/emilevr/space/actions/workflows/lint.yaml)
[![Test](https://github.com/emilevr/space/actions/workflows/test.yaml/badge.svg)](https://github.com/emilevr/space/actions/test.yaml)
[![Code Coverage](coverage/html/badges/flat.svg)](https://crates.io/crates/space)
[![Benchmark](https://github.com/emilevr/space/actions/workflows/benchmark.yaml/badge.svg)](https://github.com/emilevr/space/actions/workflows/benchmark.yaml)

A fast disk space analyzer and cleaner powered by Rust! :vulcan_salute:

This repository contains:
- A command line interface (CLI) utility that can be used to visualize and manage disk space.
- A Rust library that can be used to analyze disk space usage, in one or more directory trees. The library is not published to [crates.io](https://crates.io/), but may be at some point in the future.
- A CLI build tool that takes care of Code Coverage and Benchmarks.

## CLI

The CLI utility analyzes and displays the *apparent size* of files in one or more directory trees. Two modes are supported:

- A text user interface (TUI). This is the default mode.
  ![TUI on Windows](docs/cli/tui-windows.png)

- Non-interactive, read-only output to the terminal. This mode is used when the *--non-interactive* argument
  is specified.
  ![Non-Interactive](docs/cli/non-interactive-windows.png)

> :information_source: The *apparent size* of a file is the size of the file content, which is typically less
> than the actual space allocated as blocks on the disk. The larger the file, the less significant the
> difference.

Symbolic links are listed but not followed.

> **Yet another disk usage utility?**
>
> You may have noticed that there are a fair number of disk usage utilities out there, written in many
> languages. So why make another one? Well, this kind of project is a great way to learn a new language and a
> bunch of related stuff, like building and distributing applications for many platforms. It's also fun to write,
> with some challenges around performance that provides an opportunity to give Rust's "Fearless Concurrency" a
> spin. The implementation is straight forward and hasn't been optimized, yet it performs pretty well. Some
> future features will make much greater use of concurrency.
>
> :information_source: Optimization and benchmarks against similar tools are on the [backlog](https://github.com/users/emilevr/projects/1).
>
> **Thoughts on Rust**
>
> Given that this is my first Rust project, what are my thoughts on Rust so far? Well, it's great to work
> with a language that has no nulls, no exceptions, no garbage collection and the best tooling of any language I
> have come across so far.
>
> The current implementation of *space* is a little wasteful w.r.t. memory, which will be addressed with a
> little optimization when the time comes. That said, without a GC and a runtime that typically does work and
> allocations in the background, the memory and CPU usage is consistent and fairly low, considering the amount
> of data involved. For example:
>
> | Directory stats               | TUI Memory Usage |
> |-------------------------------|------------------|
> |   239 files,   66 directories |  ~2.8 MiB        |
> |  3937 files,  828 directories |  ~5.5 MiB        |
> | 55637 files, 6250 directories | ~37.8 MiB        |
>
> Also of note is that the CLI executable has no dependencies and is only ~1.4 MiB in size. Gotta ♥ Rust.
>
> **In conclusion**
>
> This has been such a good experience that I have decided to use Rust for all my future (suitable) side projects.

## Getting started with this repo

### Prerequisites

- The [Rust toolchain](https://www.rust-lang.org/tools/install) must be installed.
- [Visual Studio Code](https://code.visualstudio.com/) is the recommended IDE. The recommended extensions are
  listed in the [.vscode/extensions.json](.vscode/extensions.json) file and Visual Studio Code should prompt
  for installation.

### Enabling Git hooks (once off)

After the initial checkout please run the post-checkout hook manually via:
```git config core.hooksPath ./.git-hooks ; git hook run post-checkout```

This will setup the hooks directory, enabling the pre-commit hook to run linting, code coverage and benchmarks.

### Building and running the CLI

To build the library and CLI, simply run `cargo build` from the repo root directory. The binaries will be built
in the `./target/debug` directory. Alternatively use `cargo run` to build and run the CLI.

> :information_source: All the commands listed below should also be run from the repo root.

### Running Tests

Run `cargo test`.

### Running code format and lint tests

- Run `cargo fmt -- --check` to check code formatting.
- Clippy comes to the rescue for linting, via `cargo clippy`.

### Debugging

- Debugging profiles for Visual Studio Code have been created in the `.vscode` directory.
- Tests can be easily run or debugged by using the *Run Test* or *Debug* links above each test in the IDE:
  ![Run or Debug Test in VSCode](./docs/readme/run-or-debug-test-vscode.png)

### Code Coverage

This repo contains a simple Rust build project that takes care of versioning, running code coverage and benchmark tasks.

In order for code coverage to be generated, ensure you have installed *llvm-tools* via
`rustup component add llvm-tools`.

To generate a code coverage report, run `cargo buildit coverage`.

### Benchmarks

To run a benchmark test, use `cargo buildit benchmark`.

> :information_source: The plan is to make this a comparative benchmark that runs as part of each PR merge
> build, with the results automatically published to the README.

## Contributing

Please read [CONTRIBUTING.md](./CONTRIBUTING.md) for details on the code of conduct and how to submit pull
requests.

## Versioning

[SemVer](http://semver.org/) is used for versioning. For the versions available, see the
[tags](https://github.com/emilevr/space/tags) on this repository.

## Authors

- Emile van Reenen - initial work - [emilevr](https://github.com/emilevr)

Also see the list of [contributors](https://github.com/emilevr/space/contributors) to this project.

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details

## Acknowledgments

- Thanks to all the Rust library contributors out there, especially for those libs used by this project. Keep
  up the great work! See the relevant [Cargo.toml](./Cargo.toml) file dependencies.
- Thanks to the many disk usage tool authors out there, written in Rust and other languages.
  Challenge accepted! :relaxed: The future benchmark section will acknowledge a few of these.
- Thanks to [PurpleBooth](https://gist.github.com/PurpleBooth) for some
  [great repo templates](https://gist.github.com/PurpleBooth/109311bb0361f32d87a2).
