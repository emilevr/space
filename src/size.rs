//! Defines types that represent the size of a directory item.

#[cfg(test)]
#[path = "./size_test.rs"]
mod size_test;

#[cfg(feature = "cli")]
use clap::ValueEnum;

/// The format to use when a size value is displayed.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum SizeDisplayFormat {
    /// 1KB = 1000 bytes.
    Metric,
    /// 1KiB = 1024 bytes.
    Binary,
}

#[derive(Debug, Eq, PartialEq)]
struct SizeDisplayData {
    divisor: u64,
    unit: &'static str,
}

const BINARY_DISPLAY_DATA: [&SizeDisplayData; 3] = [
    &SizeDisplayData {
        divisor: 1024 * 1024 * 1024,
        unit: "GiB",
    },
    &SizeDisplayData {
        divisor: 1024 * 1024,
        unit: "MiB",
    },
    &SizeDisplayData {
        divisor: 1024,
        unit: "KiB",
    },
];

const METRIC_DISPLAY_DATA: [&SizeDisplayData; 3] = [
    &SizeDisplayData {
        divisor: 1000 * 1000 * 1000,
        unit: "GB",
    },
    &SizeDisplayData {
        divisor: 1000 * 1000,
        unit: "MB",
    },
    &SizeDisplayData {
        divisor: 1000,
        unit: "KB",
    },
];

/// A directory item size.
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Size {
    value: u64,
}

impl Size {
    /// Creates a new instance.
    #[inline(always)]
    pub fn new(value: u64) -> Self {
        Size { value }
    }

    /// Subtracts the specified number of bytes from this size.
    pub fn subtract(&mut self, value: u64) {
        if value <= self.value {
            self.value -= value;
        } else {
            self.value = 0;
        }
    }

    /// Gets the current size in bytes.
    #[inline(always)]
    pub fn get_value(&self) -> u64 {
        self.value
    }

    /// Given the total size in bytes, returns the fraction of that total that this size represents.
    pub fn get_fraction(&self, total_size_in_bytes: u64) -> f32 {
        if total_size_in_bytes == 0 {
            0f32
        } else {
            self.value as f32 / total_size_in_bytes as f32
        }
    }

    /// Converts the size to string, using the specified format.
    pub fn to_string(&self, format: SizeDisplayFormat) -> String {
        let best_format = Self::get_best_format(self.value, format);
        format!("{} {}", self.value / best_format.divisor, best_format.unit)
    }

    fn get_best_format(size_in_bytes: u64, format: SizeDisplayFormat) -> &'static SizeDisplayData {
        let config = match format {
            SizeDisplayFormat::Binary => BINARY_DISPLAY_DATA,
            SizeDisplayFormat::Metric => METRIC_DISPLAY_DATA,
        };
        for data in config {
            if size_in_bytes > data.divisor {
                return data;
            }
        }
        config[config.len() - 1]
    }
}
