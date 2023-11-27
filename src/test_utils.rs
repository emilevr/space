use anyhow::bail;
use regex::Regex;
use std::{
    env::VarError,
    fmt::Display,
    io::{self, Write},
};

use crate::cli::environment::MockEnvServiceTrait;

// Custom type for capturing output
pub(crate) struct TestOut {
    buffer: Vec<u8>,
    width: u16,
}

impl TestOut {
    pub(crate) fn new() -> Self {
        Self {
            buffer: Vec::new(),
            width: crossterm::terminal::size()
                .expect("Unable to get output width")
                .0,
        }
    }

    pub(crate) fn as_string(&self) -> String {
        String::from_utf8_lossy(&self.buffer).to_string()
    }

    pub(crate) fn contains(&self, str: &str) -> bool {
        strip_ansi_escape_sequences(self.as_string().as_str()).contains(str)
    }

    pub(crate) fn matches(&self, regex: Regex) -> anyhow::Result<()> {
        let haystack = strip_ansi_escape_sequences(self.as_string().as_str());
        if regex.is_match(haystack.as_str()) {
            Ok(())
        } else {
            bail!(
                "The specified pattern {} was expected in the output:\n{}",
                regex,
                haystack
            );
        }
    }

    pub(crate) fn expect(&self, str: &str) -> anyhow::Result<()> {
        if !self.contains(str) {
            bail!(
                "The expected string '{}' was not found in the output:\n{}",
                str,
                self
            );
        }
        Ok(())
    }
}

fn strip_ansi_escape_sequences(haystack: &str) -> String {
    let strip_ansi_regex = Regex::new("\x1B(?:[@-Z\\-_]|\\[[0-?]*[ -/]*[@-~])").unwrap();
    strip_ansi_regex.replace_all(haystack, " ").to_string()
}

impl Write for TestOut {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Display for TestOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut start_index: usize = 0;

        loop {
            let mut slice = self
                .buffer
                .get(start_index..(start_index + self.width as usize));

            if slice.is_none() {
                slice = self.buffer.get(start_index..);
                if slice.is_none() {
                    break;
                }
            }

            if let Some(slice) = slice {
                write!(f, "{}", String::from_utf8_lossy(&slice).to_string())?;
            } else {
                break;
            }

            start_index += self.width as usize;
        }

        Ok(())
    }
}

pub(crate) fn env_service_mock_without_env_vars() -> MockEnvServiceTrait {
    let mut env_service_mock = MockEnvServiceTrait::new();
    env_service_mock
        .expect_var()
        .returning(|_| Err(VarError::NotPresent));
    env_service_mock
}
