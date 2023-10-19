use std::env::{self, VarError};

use log::LevelFilter;

use super::configure_logger;

#[test]
fn configure_logging_defaults_to_level_warn() {
    // Arrange
    let log_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());

    // Act
    let level = configure_logger(
        Some(log_dir),
        Some(|key| match key {
            super::SPACE_LOG_LEVEL_ENV_VAR_NAME => Err(VarError::NotPresent),
            _ => env::var(key),
        }),
    );

    // Assert
    assert_eq!(LevelFilter::Warn, level);
}

#[test]
fn configure_logging_with_env_level_set_uses_env_level() {
    // Arrange
    let log_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());

    // Act
    let level = configure_logger(
        Some(log_dir),
        Some(|key| match key {
            super::SPACE_LOG_LEVEL_ENV_VAR_NAME => Ok("error".to_string()),
            _ => env::var(key),
        }),
    );

    // Assert
    assert_eq!(LevelFilter::Error, level);
}

#[test]
fn configure_logging_with_env_level_set_to_invalid_value_defaults_to_warn() {
    // Arrange
    let log_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());

    // Act
    let level = configure_logger(
        Some(log_dir),
        Some(|key| match key {
            super::SPACE_LOG_LEVEL_ENV_VAR_NAME => Ok("some_invalid_level".to_string()),
            _ => env::var(key),
        }),
    );

    // Assert
    assert_eq!(LevelFilter::Warn, level);
}
