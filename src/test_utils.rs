use anyhow::bail;
use regex::Regex;
use std::{
    fmt::Display,
    io::{self, Write},
};

// Custom type for capturing output
pub(crate) struct TestOut {
    buffer: Vec<u8>,
}

impl TestOut {
    pub(crate) fn new() -> Self {
        Self { buffer: Vec::new() }
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
    strip_ansi_regex.replace_all(haystack, "").to_string()
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
        write!(
            f,
            "{}",
            strip_ansi_escape_sequences(self.as_string().as_str())
        )?;
        Ok(())
    }
}
