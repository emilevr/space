## Describe your changes
-

## Issue ticket number(s) and link(s)
> :bulb: List any linked issues here by using the keywords and format described [here](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue)
> :construction_worker: DELETE THIS NOTE

-

> :construction_worker: Delete this whole section if this PR is not related to any open issues.
> :construction_worker: DELETE THIS NOTE

## PR requirements
> :exclamation: The following requirements must be met otherwise this PR will not be accepted:
> :construction_worker: Check each of the boxes below, by adding an `x` between the `[ ]` brackets (e.g. `[x]`), after meeting those requirements of course. :wink:
> :construction_worker: DELETE THIS NOTE

- [ ] I have followed the guidelines in the [Contribution Guide](../CONTRIBUTING.md#general-guidelines) document.
- [ ] I have checked that there aren't other open [Pull Requests](https://github.com/emilevr/space/pulls) for the same update/change.
- [ ] I have confirmed that all tests pass locally
  > :construction_worker: Run all tests locally via `cargo test -- --include-ignored`
  > :information_source: This will include the TUI interactive tests, so make sure you run this in a maximized terminal window. Other platforms will be tested on the build agent.
  > :construction_worker: DELETE THIS NOTE

- [ ] I have confirmed that my changes do not negatively impact performance by manually comparing before and
      after performance against a suitable local directory tree.
  > :construction_worker: Run the release build in non-interactive mode and with timing output via `cargo run --release --non-interactive --show-timing`
  > :information_source: A benchmark will be run on the build agent.
  > :construction_worker: DELETE THIS NOTE

- [ ] I have confirmed that code coverage has not regressed as a result of my changes.
  > Run code coverage locally via `cargo install --path ./buildit && buildit coverage --include-ignored` and then review the report.
  > :information_source: This will include the TUI interactive tests, so make sure you run this in a maximized terminal window. Other platforms will be tested on the build agent.
  > :construction_worker: DELETE THIS NOTE
